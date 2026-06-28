pub mod sessions;
pub mod setup;
pub mod sync;

// Re-export the items used by lib.rs
pub use setup::sync_model;
pub use sync::sync_to_codex;

// Tauri command wrappers at the codex module root level.
// Tauri's generate_handler! needs the __cmd__ helpers in the same module
// as the command path, so we define wrappers here rather than re-exporting.
#[tauri::command]
pub async fn get_sessions(offset: u32, limit: u32) -> Result<sessions::SessionsResponse, String> {
    sessions::get_sessions(offset, limit).await
}

#[tauri::command]
pub async fn get_session_content(id: String) -> Result<Vec<sessions::SessionEntry>, String> {
    sessions::get_session_content(id).await
}
