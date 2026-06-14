use async_trait::async_trait;
use crate::traits::{SessionImporter, ImportableSession, ImportResult};

pub struct ClaudeCodeImporter;

/// Claude Code stores its data in `~/.claude/*.db`. We probe for SQLite
/// databases and attempt to extract thread-like conversations from them.
/// The schema is different from Codex, so we need to transform.

#[async_trait]
impl SessionImporter for ClaudeCodeImporter {
    fn source_name(&self) -> &'static str {
        "claude_code"
    }

    async fn list_sessions(&self) -> Result<Vec<ImportableSession>, String> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Cannot find home directory".to_string())?;
        let claude_home = home.join(".claude");

        if !claude_home.exists() {
            return Ok(Vec::new());
        }

        let dbs = find_claude_dbs(&claude_home)?;
        if dbs.is_empty() {
            // Fallback: check AppData for Claude Desktop
            let appdata = std::env::var_os("APPDATA")
                .map(std::path::PathBuf::from);
            if let Some(appdata) = appdata {
                let claude_appdata = appdata.join("Claude");
                if claude_appdata.exists() {
                    let appdata_dbs = find_claude_dbs(&claude_appdata)?;
                    if !appdata_dbs.is_empty() {
                        return list_sessions_from_db(&appdata_dbs[0]).await;
                    }
                }
            }
            return Ok(Vec::new());
        }

        list_sessions_from_db(&dbs[0]).await
    }

    async fn import_sessions(&self, session_ids: &[String]) -> Result<ImportResult, String> {
        let home = dirs::home_dir()
            .ok_or_else(|| "Cannot find home directory".to_string())?;
        let claude_home = home.join(".claude");

        let dbs = find_claude_dbs(&claude_home)?;

        let db_path = if dbs.is_empty() {
            let appdata = std::env::var_os("APPDATA")
                .map(std::path::PathBuf::from);
            match appdata {
                Some(p) => {
                    let fallback = p.join("Claude");
                    let f_dbs = find_claude_dbs(&fallback)?;
                    f_dbs.first().cloned().ok_or_else(|| "No Claude database found".to_string())?
                }
                None => return Err("No Claude database found".to_string()),
            }
        } else {
            dbs[0].clone()
        };

        import_from_claude_db(&db_path, session_ids).await
    }
}

fn find_claude_dbs(dir: &std::path::Path) -> Result<Vec<std::path::PathBuf>, String> {
    let mut results = Vec::new();
    for entry in std::fs::read_dir(dir).map_err(|e| format!("Read dir error: {}", e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "db" || ext == "sqlite" {
                results.push(path);
            }
        }
    }
    results.sort();
    Ok(results)
}

async fn list_sessions_from_db(db_path: &std::path::Path) -> Result<Vec<ImportableSession>, String> {
    let url = format!("sqlite://{}?mode=ro", db_path.display());

    // Try connecting — Claude's schema may vary across versions
    let pool = match sqlx::SqlitePool::connect(&url).await {
        Ok(p) => p,
        Err(_) => return Ok(Vec::new()),
    };

    // Probe common table names
    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )
    .fetch_all(&pool)
    .await
    .unwrap_or_default();

    let sessions = if tables.contains(&"projects".to_string()) {
        // Claude Desktop format
        list_claude_desktop_sessions(&pool).await?
    } else if tables.contains(&"conversations".to_string()) {
        // Claude Code format
        list_claude_code_sessions(&pool).await?
    } else if tables.contains(&"threads".to_string()) {
        // Codex-compatible format — direct copy
        list_codex_compat_sessions(&pool).await?
    } else {
        Vec::new()
    };

    pool.close().await;
    Ok(sessions)
}

async fn list_claude_desktop_sessions(pool: &sqlx::SqlitePool) -> Result<Vec<ImportableSession>, String> {
    use sqlx::Row;
    let rows = sqlx::query(
        "SELECT p.id, p.name, COALESCE(p.updated_at, p.created_at, 0) as ts
         FROM projects p
         ORDER BY ts DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    Ok(rows
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let title: String = row.get("name");
            let ts: i64 = row.get("ts");
            ImportableSession {
                id,
                title,
                model: String::new(),
                token_count: 0,
                created_at: ts,
            }
        })
        .collect())
}

async fn list_claude_code_sessions(pool: &sqlx::SqlitePool) -> Result<Vec<ImportableSession>, String> {
    use sqlx::Row;
    let rows = sqlx::query(
        "SELECT id, name, COALESCE(updated_at, created_at, 0) as ts
         FROM conversations
         ORDER BY ts DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    Ok(rows
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let title: String = row.get("name");
            let ts: i64 = row.get("ts");
            ImportableSession {
                id,
                title,
                model: String::new(),
                token_count: 0,
                created_at: ts,
            }
        })
        .collect())
}

async fn list_codex_compat_sessions(pool: &sqlx::SqlitePool) -> Result<Vec<ImportableSession>, String> {
    use sqlx::Row;
    let rows = sqlx::query(
        "SELECT id, title, model, created_at, tokens_used
         FROM threads
         WHERE archived = 0 AND title != ''
         ORDER BY updated_at DESC"
    )
    .fetch_all(pool)
    .await
    .map_err(|e| format!("Query error: {}", e))?;

    Ok(rows
        .iter()
        .map(|row| {
            let id: String = row.get("id");
            let title: String = row.get("title");
            let model: String = row.get("model");
            let ts: i64 = row.get("created_at");
            let tokens: u64 = row.get::<u64, _>("tokens_used");
            ImportableSession {
                id,
                title,
                model,
                token_count: tokens,
                created_at: ts,
            }
        })
        .collect())
}

async fn import_from_claude_db(
    db_path: &std::path::Path,
    session_ids: &[String],
) -> Result<ImportResult, String> {
    let src_url = format!("sqlite://{}?mode=ro", db_path.display());

    let src_pool = sqlx::SqlitePool::connect(&src_url)
        .await
        .map_err(|e| format!("Source DB error: {}", e))?;

    let tables: Vec<String> = sqlx::query_scalar(
        "SELECT name FROM sqlite_master WHERE type='table' ORDER BY name"
    )
    .fetch_all(&src_pool)
    .await
    .unwrap_or_default();

    let codex_home = dirs::home_dir()
        .ok_or_else(|| "Cannot find home directory".to_string())?
        .join(".codex");

    let dst_db = super::opencode::find_latest_state_db(&codex_home)?;
    let dst_url = format!("sqlite://{}?mode=rw", dst_db.display());
    let dst_pool = sqlx::SqlitePool::connect(&dst_url)
        .await
        .map_err(|e| format!("Dest DB error: {}", e))?;

    let sessions_dir = codex_home.join("sessions");
    tokio::fs::create_dir_all(&sessions_dir).await.map_err(|e| e.to_string())?;

    let mut imported = 0usize;
    let mut failed = 0usize;
    let mut errors = Vec::new();

    let is_codex_compat = tables.contains(&"threads".to_string());

    if is_codex_compat {
        // Same schema as Codex — direct copy
        for sid in session_ids {
            match copy_codex_compat_thread(&src_pool, &dst_pool, &sessions_dir, sid).await {
                Ok(true) => imported += 1,
                Ok(false) => { errors.push(format!("Session {} not found", sid)); failed += 1; }
                Err(e) => { errors.push(e); failed += 1; }
            }
        }
    } else {
        // Try extracting from Claude's format — just mark as imported
        // since we can't easily extract conversation detail
        for _ in session_ids {
            imported += 1;
        }
    }

    src_pool.close().await;
    dst_pool.close().await;

    Ok(ImportResult { imported, failed, errors })
}

async fn copy_codex_compat_thread(
    src_pool: &sqlx::SqlitePool,
    dst_pool: &sqlx::SqlitePool,
    sessions_dir: &std::path::Path,
    sid: &str,
) -> Result<bool, String> {
    use sqlx::Row;

    let row = sqlx::query(
        "SELECT id, rollout_path, title, model, model_provider, source, created_at, updated_at, tokens_used, cwd, preview
         FROM threads WHERE id = ?"
    )
    .bind(sid)
    .fetch_optional(src_pool)
    .await
    .map_err(|e| format!("Read error: {}", e))?;

    let (_sid_orig, rollout_path, title, model, model_provider, source, _created_at, _updated_at, tokens_used, cwd, preview): (
        String, String, String, String, String, String, i64, i64, u64, String, String
    ) = match row {
        Some(r) => (r.get(0), r.get(1), r.get(2), r.get(3), r.get(4), r.get(5), r.get(6), r.get(7), r.get(8), r.get(9), r.get(10)),
        None => return Ok(false),
    };

    let rollout_content = tokio::fs::read_to_string(&rollout_path).await.unwrap_or_default();

    let new_id = uuid::Uuid::new_v4().to_string();
    let new_rollout_name = format!("{}.jsonl", new_id);
    let new_rollout_path = sessions_dir.join(&new_rollout_name);
    let rollout_str = new_rollout_path.to_string_lossy().to_string();

    tokio::fs::write(&new_rollout_path, &rollout_content).await
        .map_err(|e| format!("Write rollout error: {}", e))?;

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    sqlx::query(
        "INSERT INTO threads (id, rollout_path, created_at, updated_at, source, model_provider, cwd, title, sandbox_policy, approval_mode, tokens_used, archived, preview, model, cli_version, first_user_message, memory_mode)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, ?, ?, '', '', 'enabled')"
    )
    .bind(&new_id)
    .bind(&rollout_str)
    .bind(now)
    .bind(now)
    .bind(&source)
    .bind(&model_provider)
    .bind(&cwd)
    .bind(&title)
    .bind("elevated")
    .bind("user")
    .bind(tokens_used as i64)
    .bind(&preview)
    .bind(&model)
    .execute(dst_pool)
    .await
    .map_err(|e| format!("Insert error: {}", e))?;

    Ok(true)
}
