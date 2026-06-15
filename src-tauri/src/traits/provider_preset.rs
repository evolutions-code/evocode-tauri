use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct ProviderConfig {
    pub id: String,
    pub title: String,
    pub description: String,
    pub wire_api: String,
    pub base_url: String,
    pub model: String,
    pub api_key_header: String,
    pub context_window: u32,
    pub compact_limit: u32,
}

#[allow(dead_code)]
pub trait ProviderPreset: Send + Sync {
    fn id(&self) -> &'static str;
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn build_config(&self) -> ProviderConfig;
}

pub fn all_presets() -> Vec<Box<dyn ProviderPreset>> {
    vec![Box::new(crate::presets::opencode_go::OpenCodeGoPreset)]
}
