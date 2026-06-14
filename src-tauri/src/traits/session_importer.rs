use serde::Serialize;
use async_trait::async_trait;

#[derive(Debug, Clone, Serialize)]
pub struct ImportableSession {
    pub id: String,
    pub title: String,
    pub model: String,
    pub token_count: u64,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub failed: usize,
    pub errors: Vec<String>,
}

#[async_trait]
pub trait SessionImporter: Send + Sync {
    fn source_name(&self) -> &'static str;
    async fn list_sessions(&self) -> Result<Vec<ImportableSession>, String>;
    async fn import_sessions(&self, session_ids: &[String]) -> Result<ImportResult, String>;
}
