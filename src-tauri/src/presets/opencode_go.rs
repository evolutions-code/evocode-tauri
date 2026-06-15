use crate::traits::{ProviderPreset, ProviderConfig};

pub struct OpenCodeGoPreset;

impl ProviderPreset for OpenCodeGoPreset {
    fn id(&self) -> &'static str {
        "opencode-go"
    }

    fn title(&self) -> &'static str {
        "OpenCode Go 订阅"
    }

    fn description(&self) -> &'static str {
        "填入 API Key 即可使用 DeepSeek V4、GLM-5、Kimi K2.7 等模型"
    }

    fn build_config(&self) -> ProviderConfig {
        ProviderConfig {
            id: "opencode-go".to_string(),
            title: "OpenCode Go 订阅".to_string(),
            description: "填入 API Key 即可使用 DeepSeek V4、GLM-5、Kimi K2.7 等模型".to_string(),
            wire_api: "chat_completions".to_string(),
            base_url: "https://opencode.ai/zen/go/v1".to_string(),
            model: "deepseek-v4-pro".to_string(),
            api_key_header: "Authorization".to_string(),
            context_window: 128_000,
            compact_limit: 100_000,
        }
    }
}
