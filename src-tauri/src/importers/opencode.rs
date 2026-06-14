use async_trait::async_trait;
use crate::traits::{SessionImporter, ImportableSession, ImportResult};

pub struct OpenCodeImporter;

#[async_trait]
impl SessionImporter for OpenCodeImporter {
    fn source_name(&self) -> &'static str {
        "opencode"
    }

    async fn list_sessions(&self) -> Result<Vec<ImportableSession>, String> {
        let opencode_home = dirs::home_dir()
            .ok_or_else(|| "Cannot find home directory".to_string())?
            .join(".opencode");

        if !opencode_home.exists() {
            return Ok(Vec::new());
        }

        let db_path = find_latest_state_db(&opencode_home)?;
        let url = format!("sqlite://{}?mode=ro", db_path.display());

        let pool = sqlx::SqlitePool::connect(&url)
            .await
            .map_err(|e| format!("DB error: {}", e))?;

        let rows = sqlx::query_as::<_, (String, String, String, i64, u64)>(
            "SELECT id, title, model, created_at, tokens_used
             FROM threads
             WHERE archived = 0 AND title != '' AND tokens_used > 0
             ORDER BY updated_at DESC"
        )
        .fetch_all(&pool)
            .await
            .map_err(|e| format!("Query error: {}", e))?;

        pool.close().await;

        Ok(rows
            .into_iter()
            .map(|(id, title, model, created_at, tokens_used)| ImportableSession {
                id,
                title,
                model,
                token_count: tokens_used,
                created_at,
            })
            .collect())
    }

    async fn import_sessions(&self, session_ids: &[String]) -> Result<ImportResult, String> {
        let opencode_home = dirs::home_dir()
            .ok_or_else(|| "Cannot find home directory".to_string())?
            .join(".opencode");

        let src_db = find_latest_state_db(&opencode_home)?;
        let src_url = format!("sqlite://{}?mode=ro", src_db.display());
        let src_pool = sqlx::SqlitePool::connect(&src_url)
            .await
            .map_err(|e| format!("Source DB error: {}", e))?;

        let codex_home = dirs::home_dir()
            .ok_or_else(|| "Cannot find home directory".to_string())?
            .join(".codex");

        let dst_db = find_latest_state_db(&codex_home)?;

        // Connect to codex state DB in read-write mode
        let dst_url = format!("sqlite://{}?mode=rw", dst_db.display());
        let dst_pool = sqlx::SqlitePool::connect(&dst_url)
            .await
            .map_err(|e| format!("Dest DB error: {}", e))?;

        let mut imported = 0usize;
        let mut failed = 0usize;
        let mut errors = Vec::new();

        let sessions_dir = codex_home.join("sessions");
        tokio::fs::create_dir_all(&sessions_dir).await.map_err(|e| e.to_string())?;

        for sid in session_ids {
            let src_row = sqlx::query_as::<_, (String, String, String, String, String, String, i64, i64, u64, String, String)>(
                "SELECT id, rollout_path, title, model, model_provider, source, created_at, updated_at, tokens_used, cwd, preview
                 FROM threads WHERE id = ?"
            )
            .bind(sid)
            .fetch_optional(&src_pool)
            .await
            .map_err(|e| format!("Read error: {}", e));

            let src_row = match src_row {
                Ok(Some(r)) => r,
                Ok(None) => { errors.push(format!("Session {} not found", sid)); failed += 1; continue; }
                Err(e) => { errors.push(format!("Session {} error: {}", sid, e)); failed += 1; continue; }
            };

            let (_sid_orig, rollout_path, title, model, model_provider, source, _created_at, _updated_at, tokens_used, cwd, preview) = src_row;

            // Read the source rollout .jsonl
            let rollout_content = match tokio::fs::read_to_string(&rollout_path).await {
                Ok(c) => c,
                Err(_) => String::new(),
            };

            // Generate a fresh UUID to avoid collisions
            let new_id = uuid::Uuid::new_v4().to_string();
            let new_rollout_name = format!("{}.jsonl", new_id);
            let new_rollout_path = sessions_dir.join(&new_rollout_name);

            // Write the rollout file
            if let Err(e) = tokio::fs::write(&new_rollout_path, &rollout_content).await {
                errors.push(format!("Failed to write rollout for {}: {}", sid, e));
                failed += 1;
                continue;
            }

            // Insert into threads table
            let rollout_str = new_rollout_path.to_string_lossy().to_string();
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs() as i64;

            let result = sqlx::query(
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
            .execute(&dst_pool)
            .await;

            match result {
                Ok(_) => imported += 1,
                Err(e) => {
                    errors.push(format!("Insert session {} failed: {}", sid, e));
                    let _ = tokio::fs::remove_file(&new_rollout_path).await;
                    failed += 1;
                }
            }
        }

        src_pool.close().await;
        dst_pool.close().await;

        Ok(ImportResult { imported, failed, errors })
    }
}

pub fn find_latest_state_db(home: &std::path::Path) -> Result<std::path::PathBuf, String> {
    let mut best: Option<(u32, std::path::PathBuf)> = None;
    for entry in std::fs::read_dir(home).map_err(|e| format!("Read dir error: {}", e))? {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name.starts_with("state_") && name.ends_with(".sqlite") {
                let num = name
                    .trim_start_matches("state_")
                    .trim_end_matches(".sqlite")
                    .parse::<u32>()
                    .unwrap_or(0);
                let replace = match &best {
                    None => true,
                    Some((best_num, _)) => num > *best_num,
                };
                if replace {
                    best = Some((num, path));
                }
            }
        }
    }
    best.map(|(_, p)| p)
        .ok_or_else(|| "No state database found".to_string())
}
