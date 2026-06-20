use serde::{Serialize};
use std::path::PathBuf;

/// Core system prompt compiled from prompt.md
pub const CORE_PROMPT: &str = include_str!("../prompt.md");

/// Directory where prompt files are stored (~/.evocode/prompts/)
fn get_prompts_dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    home.join(".evocode").join("prompts")
}

/// Returns the path to .codex/AGENTS.md for the current project.
/// Tries current directory first, then falls back to home directory.
/// Returns true if there are any prompt files in ~/.evocode/prompts/
pub fn has_prompt_files() -> bool {
    let dir = get_prompts_dir();
    if !dir.exists() {
        return false;
    }
    std::fs::read_dir(&dir)
        .ok()
        .map(|entries| {
            entries.filter_map(|e| e.ok()).any(|e| {
                e.path().extension().and_then(|s| s.to_str()) == Some("md")
            })
        })
        .unwrap_or(false)
}

pub fn agents_file_path() -> PathBuf {
    get_codex_agents_path()
}

fn get_codex_agents_path() -> PathBuf {
    // Strategy 1: current working directory (works in `tauri dev`)
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let cwd_path = home.join(".codex").join("AGENTS.md");
    return cwd_path;
}

#[derive(Serialize, Clone)]
pub struct PromptFile {
    pub name: String,
    pub updated_at: String,
}

/// Helper: convert a system time to ISO string
fn system_time_to_iso(t: std::time::SystemTime) -> String {
    let duration = t.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
    let secs = duration.as_secs();
    // Simple conversion to ISO-like string
    let naive = chrono::DateTime::from_timestamp(secs as i64, 0)
        .map(|dt| dt.format("%Y-%m-%dT%H:%M:%S%.3fZ").to_string())
        .unwrap_or_else(|| "unknown".to_string());
    naive
}

/// Get the file path for a prompt name.
fn prompt_file_path(name: &str) -> PathBuf {
    get_prompts_dir().join(format!("{}.md", name))
}

/// List all prompt files in ~/.evocode/prompts/
#[tauri::command]
pub async fn list_prompt_files() -> Result<Vec<PromptFile>, String> {
    let dir = get_prompts_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = match std::fs::read_dir(&dir) {
        Ok(e) => e,
        Err(e) => return Err(format!("读取提示词目录失败: {}", e)),
    };

    let mut files = Vec::new();
    while let Some(entry) = entries.next().transpose().map_err(|e| format!("读取目录项失败: {}", e))? {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("md") {
            let name = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("")
                .to_string();
            let updated_at = entry
                .metadata()
                .ok()
                .and_then(|m| m.modified().ok())
                .map(system_time_to_iso)
                .unwrap_or_else(|| "".to_string());
            files.push(PromptFile { name, updated_at });
        }
    }

    // Sort by updated_at descending
    files.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Ok(files)
}

/// Read the content of a prompt file by name (filename without .md)
#[tauri::command]
pub async fn read_prompt_file(name: String) -> Result<String, String> {
    let path = prompt_file_path(&name);
    if !path.exists() {
        return Err(format!("提示词「{}」不存在", name));
    }
    std::fs::read_to_string(&path).map_err(|e| format!("读取提示词文件失败: {}", e))
}

/// Write content to a prompt file (creates or overwrites)
#[tauri::command]
pub async fn write_prompt_file(name: String, content: String) -> Result<(), String> {
    let dir = get_prompts_dir();
    std::fs::create_dir_all(&dir).map_err(|e| format!("创建提示词目录失败: {}", e))?;
    let path = prompt_file_path(&name);
    std::fs::write(&path, &content).map_err(|e| format!("写入提示词文件失败: {}", e))?;
    Ok(())
}

/// Return the bundled core system prompt content
#[tauri::command]
pub async fn read_core_prompt() -> Result<String, String> {
    Ok(CORE_PROMPT.to_string())
}

/// Delete a prompt file by name
#[tauri::command]
pub async fn delete_prompt_file(name: String) -> Result<(), String> {
    let path = prompt_file_path(&name);
    if !path.exists() {
        return Err(format!("提示词「{}」不存在", name));
    }
    std::fs::remove_file(&path).map_err(|e| format!("删除提示词文件失败: {}", e))
}

/// Read the current .codex/AGENTS.md content
#[tauri::command]
pub async fn read_agents_file() -> Result<String, String> {
    let path = get_codex_agents_path();
    if !path.exists() {
        return Ok(String::new());
    }
    std::fs::read_to_string(&path).map_err(|e| format!("读取 AGENTS.md 失败: {}", e))
}

/// Write content to .codex/AGENTS.md (activate a prompt)
#[tauri::command]
pub async fn write_agents_file(content: String) -> Result<(), String> {
    let path = get_codex_agents_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    std::fs::write(&path, &content).map_err(|e| format!("写入 AGENTS.md 失败: {}", e))?;
    Ok(())
}
