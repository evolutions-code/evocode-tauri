use async_trait::async_trait;
use crate::traits::{ModelFetcher, ModelInfo};

pub struct AnthropicModelFetcher;

#[async_trait]
impl ModelFetcher for AnthropicModelFetcher {
    fn wire_api(&self) -> &'static str {
        "anthropic"
    }

    async fn fetch_models(
        &self,
        base_url: &str,
        api_key: &str,
        api_key_header: Option<&str>,
    ) -> Result<Vec<ModelInfo>, String> {
        let base = base_url.trim_end_matches('/');
        let url = format!("{}/v1/models", base);

        let header_name = api_key_header
            .filter(|s| !s.trim().is_empty())
            .unwrap_or("X-Api-Key");

        let header_value = if header_name.eq_ignore_ascii_case("Authorization") {
            format!("Bearer {}", api_key)
        } else {
            api_key.to_string()
        };

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|e| e.to_string())?;

        let resp = client
            .get(&url)
            .header(header_name, &header_value)
            .header("Content-Type", "application/json")
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }

        let body: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Parse error: {}", e))?;

        // Anthropic returns { "data": [{ "id": "claude-3-5-sonnet-20241022", ... }] }
        let models = body
            .get("data")
            .and_then(|d| d.as_array())
            .ok_or_else(|| "Unexpected response format".to_string())?;

        let infos: Vec<ModelInfo> = models
            .iter()
            .filter_map(|m| {
                let id = m.get("id")?.as_str()?.to_string();
                let name = m.get("name").and_then(|n| n.as_str()).map(String::from);
                Some(ModelInfo { id, name })
            })
            .collect();

        Ok(infos)
    }
}
