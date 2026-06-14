use async_trait::async_trait;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ModelInfo {
    pub id: String,
    pub name: Option<String>,
}

#[async_trait]
pub trait ModelFetcher: Send + Sync {
    #[allow(dead_code)]
    fn wire_api(&self) -> &'static str;
    async fn fetch_models(
        &self,
        base_url: &str,
        api_key: &str,
        api_key_header: Option<&str>,
    ) -> Result<Vec<ModelInfo>, String>;
}

pub fn model_fetcher_for_wire_api(wire_api: &str) -> Box<dyn ModelFetcher> {
    match wire_api {
        "anthropic" => Box::new(crate::fetchers::anthropic::AnthropicModelFetcher),
        _ => Box::new(crate::fetchers::openai::OpenAIModelFetcher),
    }
}
