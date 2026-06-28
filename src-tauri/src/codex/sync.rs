use std::path::Path;

/// Full Codex sync: setup home, then inject base_instructions into models_catalog.json.
/// The `get_prompt_source` closure determines what base_instructions text to use.
pub fn sync_to_codex<F>(get_prompt_source: F) -> Result<(), String>
where
    F: FnOnce() -> String,
{
    let config = evocode_config::load_config().map_err(|e| e.to_string())?;
    let codex_home = super::setup::default_home()?;
    super::setup::setup_home(&config, &codex_home)?;

    let prompt_source = get_prompt_source();
    inject_base_instructions(&codex_home, &prompt_source);
    Ok(())
}

fn inject_base_instructions(codex_home: &Path, prompt_source: &str) {
    let catalog_path = codex_home.join("models_catalog.json");
    if !catalog_path.exists() {
        return;
    }
    let catalog_str = match std::fs::read_to_string(&catalog_path) {
        Ok(s) => s,
        Err(_) => return,
    };
    let mut catalog: serde_json::Value = match serde_json::from_str(&catalog_str) {
        Ok(v) => v,
        Err(_) => return,
    };
    if let Some(models) = catalog.get_mut("models").and_then(|m| m.as_array_mut()) {
        for model in models.iter_mut() {
            if let Some(obj) = model.as_object_mut() {
                obj.insert("base_instructions".to_string(), serde_json::Value::String(prompt_source.to_string()));
            }
        }
    }
    if let Ok(updated) = serde_json::to_string_pretty(&catalog) {
        let _ = std::fs::write(&catalog_path, &updated);
        tracing::info!("[codex] injected base_instructions into models_catalog");
    }
}
