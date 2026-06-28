use std::path::{Path, PathBuf};

use evocode_config::EvocodeConfig;

/// Default Codex home directory (~/.codex)
pub fn default_home() -> Result<PathBuf, String> {
    dirs::home_dir()
        .map(|h| h.join(".codex"))
        .ok_or_else(|| "Cannot find home directory".to_string())
}

/// Setup Codex home: write models_catalog.json, config.toml, auth.json
pub fn setup_home(config: &EvocodeConfig, codex_home: &Path) -> Result<(), String> {
    std::fs::create_dir_all(codex_home).map_err(|e| e.to_string())?;
    write_models_catalog(config, codex_home)?;
    write_config_toml(config, codex_home)?;
    write_auth_json(config, codex_home)?;
    Ok(())
}

/// Update the model field in Codex config.toml
pub fn sync_model(model: &str) -> Result<(), String> {
    let codex_home = default_home()?;
    let config_path = codex_home.join("config.toml");
    let toml_str = std::fs::read_to_string(&config_path).unwrap_or_default();
    let mut cfg: toml::Value = toml::from_str(&toml_str)
        .unwrap_or(toml::Value::Table(toml::Table::new()));
    if let Some(table) = cfg.as_table_mut() {
        table.insert("model".to_string(), toml::Value::String(model.to_string()));
        // Remove global overrides so per-model catalog values take effect
        table.remove("model_context_window");
        table.remove("model_auto_compact_token_limit");
    }
    let out = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, &out).map_err(|e| e.to_string())?;
    Ok(())
}

// ── helpers ──────────────────────────────────────────────────────────

fn write_models_catalog(config: &EvocodeConfig, codex_home: &Path) -> Result<(), String> {
    let mut models_json: Vec<serde_json::Value> = Vec::new();
    for (id, provider) in &config.providers {
        if provider.base_url.is_none() {
            continue;
        }
        for entry in &provider.models {
            if entry.name.is_empty() || entry.name == "codex" {
                continue;
            }
            let slug = format!("{}:{}", id, entry.name);
            let ctx = entry.context_window.unwrap_or(256_000);
            let ac = entry.auto_compact_token_limit.unwrap_or(ctx * 8 / 10);
            let mods: Vec<String> = if entry.input_modalities.is_empty() {
                vec!["text".to_string()]
            } else {
                entry.input_modalities.clone()
            };
            models_json.push(build_model_entry(&slug, ctx, ac, &mods));
        }
    }
    let catalog = serde_json::json!({ "models": models_json });
    let catalog_path = codex_home.join("models_catalog.json");
    let json = serde_json::to_string_pretty(&catalog).map_err(|e| e.to_string())?;
    std::fs::write(&catalog_path, &json).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_config_toml(config: &EvocodeConfig, codex_home: &Path) -> Result<(), String> {
    let model = config.model.clone().unwrap_or_default();
    let model = if model.is_empty() {
        config.provider.as_deref()
            .and_then(|pid| config.providers.get(pid))
            .and_then(|p| p.models.first())
            .map(|m| format!("{}:{}", config.provider.as_deref().unwrap_or("evocode"), m.name))
            .unwrap_or_else(|| "codex".to_string())
    } else {
        model
    };
    let base_url = format!("http://127.0.0.1:{}", config.port());
    let provider_name = config.provider.clone().unwrap_or_else(|| "evocode".to_string());
    let config_path = codex_home.join("config.toml");
    let catalog_path = codex_home.join("models_catalog.json");

    let existing_toml = std::fs::read_to_string(&config_path).ok();
    let mut cfg: toml::Value = existing_toml
        .and_then(|c| toml::from_str(&c).ok())
        .unwrap_or_else(|| toml::Value::Table(toml::Table::new()));
    if !cfg.is_table() {
        cfg = toml::Value::Table(toml::Table::new());
    }

    let ctx_win = config.provider.as_deref()
        .and_then(|pid| config.providers.get(pid))
        .and_then(|p| p.models.first())
        .and_then(|m| m.context_window);
    let compact_limit = config.provider.as_deref()
        .and_then(|pid| config.providers.get(pid))
        .and_then(|p| p.models.first())
        .and_then(|m| m.auto_compact_token_limit);

    set_toml_value(&mut cfg, &["model_provider".to_string()], toml::Value::String("evocode".to_string()));
    set_toml_value(&mut cfg, &["model".to_string()], toml::Value::String(model));
    set_toml_value(&mut cfg, &provider_path(&provider_name, "name"), toml::Value::String("evocode".to_string()));
    set_toml_value(&mut cfg, &provider_path(&provider_name, "requires_openai_auth"), toml::Value::Boolean(false));
    set_toml_value(&mut cfg, &provider_path(&provider_name, "base_url"), toml::Value::String(base_url));
    set_toml_value(&mut cfg, &["model_context_window".to_string()], toml::Value::Integer(ctx_win.unwrap_or(256_000)));
    set_toml_value(&mut cfg, &["model_auto_compact_token_limit".to_string()], toml::Value::Integer(compact_limit.unwrap_or(200_000)));
    set_toml_value(&mut cfg, &["model_catalog_json".to_string()], toml::Value::String(catalog_path.to_string_lossy().to_string()));

    // Remove global overrides so Codex uses per-model catalog values
    if let Some(table) = cfg.as_table_mut() {
        table.remove("model_context_window");
        table.remove("model_auto_compact_token_limit");
    }

    let out = toml::to_string_pretty(&cfg).map_err(|e| e.to_string())?;
    std::fs::write(&config_path, &out).map_err(|e| e.to_string())?;
    Ok(())
}

fn write_auth_json(config: &EvocodeConfig, codex_home: &Path) -> Result<(), String> {
    let auth_path = codex_home.join("auth.json");
    if let Some(api_key) = config.provider.as_deref()
        .and_then(|pid| config.providers.get(pid))
        .and_then(|p| p.api_key.as_deref())
        .or(config.api_key.as_deref())
    {
        let auth_json = format!(r#"{{"OPENAI_API_KEY": "{}"}}"#, api_key);
        std::fs::write(&auth_path, auth_json).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn build_model_entry(slug: &str, ctx: i64, ac: i64, mods: &[String]) -> serde_json::Value {
    serde_json::json!({
        "slug": slug,
        "display_name": slug,
        "description": null,
        "default_reasoning_level": "medium",
        "supported_reasoning_levels": [
            {"effort": "low", "description": "Fast responses with lighter reasoning"},
            {"effort": "medium", "description": "Balances speed and reasoning depth for everyday tasks"},
            {"effort": "high", "description": "Greater reasoning depth for complex problems"},
            {"effort": "xhigh", "description": "Extra high reasoning depth for complex problems"}
        ],
        "shell_type": "default",
        "visibility": "list",
        "supported_in_api": true,
        "priority": 0,
        "additional_speed_tiers": [],
        "service_tiers": [],
        "availability_nux": null,
        "upgrade": null,
        "base_instructions": "",
        "model_messages": null,
        "supports_reasoning_summaries": false,
        "default_reasoning_summary": "auto",
        "support_verbosity": false,
        "default_verbosity": null,
        "apply_patch_tool_type": "freeform",
        "web_search_tool_type": "text",
        "truncation_policy": {"mode": "bytes", "limit": 10000},
        "supports_parallel_tool_calls": false,
        "supports_image_detail_original": true,
        "context_window": ctx,
        "max_context_window": ctx,
        "auto_compact_token_limit": ac,
        "effective_context_window_percent": 95,
        "experimental_supported_tools": [],
        "input_modalities": mods,
        "supports_search_tool": false
    })
}

fn set_toml_value(config: &mut toml::Value, path: &[String], value: toml::Value) {
    if path.is_empty() { return; }
    let mut current = config;
    for key in &path[..path.len() - 1] {
        let table = current.as_table_mut().expect("toml root is a table");
        current = table.entry(key.clone()).or_insert_with(|| toml::Value::Table(toml::Table::new()));
        if !current.is_table() {
            *current = toml::Value::Table(toml::Table::new());
        }
    }
    let table = current.as_table_mut().expect("toml parent is a table");
    table.insert(path[path.len() - 1].clone(), value);
}

fn provider_path(provider_name: &str, key: &str) -> Vec<String> {
    vec!["model_providers".to_string(), provider_name.to_string(), key.to_string()]
}
