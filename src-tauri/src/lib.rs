mod connectivity;
mod traits;
mod fetchers;
mod prompts;
mod codex;

use serde::Serialize;
use std::path::PathBuf;
use std::io::BufRead;
use std::sync::Arc;

use evocode_config::load_config;
use evocode_proto::ServerConfig;
use tauri::image::Image;
use tauri::menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, State};
use tokio::sync::Mutex as AsyncMutex;


pub struct BridgeState {
    handle: Arc<AsyncMutex<Option<tokio::task::JoinHandle<()>>>>,
    port: std::sync::atomic::AtomicU16,
}

#[derive(Serialize)]
pub struct LogTailResult {
    pub lines: Vec<String>,
    pub total_lines: usize,
}

#[derive(Serialize)]
pub struct LogRangeResult {
    pub lines: Vec<String>,
    pub total_lines: usize,
}

impl BridgeState {
    fn new(port: u16) -> Self {
        Self {
            handle: Arc::new(AsyncMutex::new(None)),
            port: std::sync::atomic::AtomicU16::new(port),
        }
    }

    fn load_port_from_config() -> u16 {
        load_config()
            .ok()
            .and_then(|c| c.port)
            .unwrap_or(17761)
    }

    fn set_port(&self, port: u16) {
        self.port.store(port, std::sync::atomic::Ordering::Relaxed);
    }
}

fn get_config_path() -> Option<PathBuf> {
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)?;
    let primary = home.join(".evocode/config.json");
    if primary.exists() {
        return Some(primary);
    }
    let typo_compat = home.join(".evocde/config.json");
    if typo_compat.exists() {
        return Some(typo_compat);
    }
    Some(primary)
}

    // Custom timer that prints timestamps in the host's local timezone,
    // so log lines are no longer fixed to UTC.
    struct LocalTimer;
    impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
        fn format_time(
            &self,
            w: &mut tracing_subscriber::fmt::format::Writer<'_>,
        ) -> std::fmt::Result {
            write!(
                w,
                "{}",
                chrono::Local::now().format("%Y-%m-%dT%H:%M:%S%.3f%z")
            )
        }
    }

#[tauri::command]
async fn start_bridge(state: State<'_, BridgeState>) -> Result<String, String> {
    let mut handle_guard = state.handle.lock().await;
    if handle_guard.is_some() {
        return Err("Bridge is already running".into());
    }

    let config = evocode_config::EvocodeConfig::load().unwrap_or_default();
    let port = state.port.load(std::sync::atomic::Ordering::Relaxed);
    let listen_addr: std::net::SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| format!("Invalid port: {}", e))?;
    let cfg = ServerConfig {
        listen: listen_addr,
        providers: config.provider_routes(),
        upstream_url: config
            .base_url()
            .unwrap_or("http://127.0.0.1:17761")
            .to_string(),
        api_key: config.api_key().unwrap_or("").to_string(),
        api_key_header: config.api_key_header().to_string(),
        protocol: config.protocol(),
        provider: config.provider.clone().unwrap_or_default(),
        ..Default::default()
    };

    // Clear log file so each bridge start shows fresh logs
    let _ = std::fs::write(bridge_log_path(), "");

    let handle = tokio::spawn(async move {
        if let Err(e) = evocode_proto::serve(cfg).await {
            let msg = format!("[ERROR] {}", e);
            eprintln!("{}", msg);
            tracing::error!("bridge error: {}", e);
        }
    });

    *handle_guard = Some(handle);
    Ok(format!("Bridge starting on port {}...", port))
}

#[tauri::command]
async fn bridge_is_running(state: State<'_, BridgeState>) -> Result<bool, String> {
    Ok(state.handle.lock().await.is_some())
}

#[tauri::command]
async fn stop_bridge(state: State<'_, BridgeState>) -> Result<String, String> {
    let mut handle_guard = state.handle.lock().await;
    if let Some(handle) = handle_guard.take() {
        handle.abort();
        return Ok("Bridge stopped".into());
    }
    Ok("Bridge was not running".into())
}

#[tauri::command]
async fn bridge_status(state: State<'_, BridgeState>) -> Result<String, String> {
    let handle_guard = state.handle.lock().await;
    Ok(if handle_guard.is_some() {
        "running"
    } else {
        "stopped"
    }
    .into())
}

#[tauri::command]
async fn get_bridge_url(state: State<'_, BridgeState>) -> Result<String, String> {
    let port = state.port.load(std::sync::atomic::Ordering::Relaxed);
    Ok(format!("http://127.0.0.1:{}", port))
}

#[tauri::command]
async fn get_bridge_port(state: State<'_, BridgeState>) -> Result<u16, String> {
    let port = state.port.load(std::sync::atomic::Ordering::Relaxed);
    Ok(port)
}

#[tauri::command]
async fn set_bridge_port(state: State<'_, BridgeState>, port: u16) -> Result<(), String> {
    if port < 1024 {
        return Err("Port must be between 1024 and 65535".into());
    }
    if state.handle.lock().await.is_some() {
        return Err("Cannot change port while bridge is running. Please stop the bridge first.".into());
    }
    // Ensure ~/.evocode/ directory exists before writing
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::create_dir_all(home.join(".evocode"));
    }
    let mut config = evocode_config::EvocodeConfig::load().unwrap_or_default();
    config.port = Some(port);
    config.save().map_err(|e| e.to_string())?;
    state.set_port(port);
    Ok(())
}

#[tauri::command]
async fn set_max_request_body_size(size: Option<u64>) -> Result<(), String> {
    // Ensure ~/.evocode/ directory exists before writing
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::create_dir_all(home.join(".evocode"));
    }
    let mut config = evocode_config::EvocodeConfig::load().unwrap_or_default();
    config.max_request_body_size = size;
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_bridge_logs() -> Result<Vec<String>, String> {
    let log_path = bridge_log_path();
    let bytes = std::fs::read(&log_path).unwrap_or_default();
    let text = String::from_utf8_lossy(&bytes);
    Ok(text.lines().map(|l| l.to_string()).collect())
}

#[tauri::command]
async fn get_bridge_logs_tail(count: usize) -> Result<LogTailResult, String> {
    let log_path = bridge_log_path();
    let log_path = match std::fs::canonicalize(&log_path) {
        Ok(p) => p,
        Err(_) => return Ok(LogTailResult { lines: vec![], total_lines: 0 }),
    };
    let file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return Ok(LogTailResult { lines: vec![], total_lines: 0 }),
    };
    let reader = std::io::BufReader::new(file);
    let mut total: usize = 0;
    let mut circular: Vec<String> = Vec::with_capacity(count + 1);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                total += 1;
                if circular.len() >= count {
                    circular.remove(0);
                }
                circular.push(l);
            }
            Err(_) => continue,
        }
    }
    Ok(LogTailResult { lines: circular, total_lines: total })
}

#[tauri::command]
async fn get_bridge_logs_range(start: usize, count: usize) -> Result<LogRangeResult, String> {
    let log_path = bridge_log_path();
    let log_path = match std::fs::canonicalize(&log_path) {
        Ok(p) => p,
        Err(_) => return Ok(LogRangeResult { lines: vec![], total_lines: 0 }),
    };
    let file = match std::fs::File::open(&log_path) {
        Ok(f) => f,
        Err(_) => return Ok(LogRangeResult { lines: vec![], total_lines: 0 }),
    };
    let reader = std::io::BufReader::new(file);
    let mut total: usize = 0;
    let mut collected: Vec<String> = Vec::with_capacity(count);
    for line in reader.lines() {
        match line {
            Ok(l) => {
                if total >= start && collected.len() < count {
                    collected.push(l);
                }
                total += 1;
            }
            Err(_) => continue,
        }
    }
    Ok(LogRangeResult { lines: collected, total_lines: total })
}

#[tauri::command]
async fn read_config() -> Result<String, String> {
    let config_path =
        get_config_path().ok_or_else(|| "Cannot find config directory".to_string())?;

    if !config_path.exists() {
        return Ok(String::new());
    }

    tokio::fs::read_to_string(&config_path)
        .await
        .map_err(|e| e.to_string())
}
#[tauri::command]
async fn read_config_json() -> Result<evocode_config::EvocodeConfig, String> {
    match evocode_config::load_config() {
        Ok(cfg) => Ok(cfg),
        Err(_) => Ok(evocode_config::EvocodeConfig::default()),
    }
}



#[tauri::command]
async fn save_config(config: evocode_config::EvocodeConfig) -> Result<(), String> {
    tracing::info!("save_config received: model={:?}, provider={:?}", config.model, config.provider);
    // Ensure ~/.evocode/ directory exists before writing
    if let Some(home) = dirs::home_dir() {
        let _ = std::fs::create_dir_all(home.join(".evocode"));
    }
    let result = config.save().map_err(|e| e.to_string());
    tracing::info!("save_config result: {:?}", result);
    if let Ok(ref config_path) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        let path = std::path::Path::new(&config_path).join(".evocode/config.json");
        if let Ok(content) = std::fs::read_to_string(&path) {
        tracing::info!("config.json after save: {}", content);
        }
    }
    result
}



#[tauri::command]
async fn sync_to_codex() -> Result<(), String> {
    crate::codex::sync_to_codex(|| {
        let has_prompt_files = crate::prompts::has_prompt_files();
        let agents_path = crate::prompts::agents_file_path();
        if has_prompt_files && agents_path.exists() {
            if let Ok(user_content) = std::fs::read_to_string(&agents_path) {
                let trimmed = user_content.trim();
                if !trimmed.is_empty() {
                    return trimmed.to_string();
                }
            }
        }
        crate::prompts::CORE_PROMPT.trim().to_string()
    })
}

#[tauri::command]
async fn sync_model_to_codex(model: String) -> Result<(), String> {
    crate::codex::sync_model(&model)
}
#[tauri::command]
async fn test_provider_connectivity(
    base_url: String,
    api_key: String,
    wire_api: String,
    api_key_header: Option<String>,
    model: Option<String>,
) -> Result<connectivity::ConnectivityResult, String> {
    Ok(connectivity::test_provider(
        &base_url,
        &api_key,
        &wire_api,
        api_key_header.as_deref(),
        model.as_deref(),
    )
    .await)
}

#[tauri::command]
async fn open_config_dir(app: AppHandle) -> Result<String, String> {
    // Ensure ~/.evocode exists, then open it with the system file manager.
    let home = std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .map(PathBuf::from)
        .ok_or_else(|| "Cannot find home directory".to_string())?;
    let dir = home.join(".evocode");
    tokio::fs::create_dir_all(&dir)
        .await
        .map_err(|e| format!("Failed to create {}: {}", dir.display(), e))?;

    let path_str = dir.to_string_lossy().to_string();
    use tauri_plugin_opener::OpenerExt;
    app.opener()
        .open_path(path_str.clone(), None::<&str>)
        .map_err(|e| e.to_string())?;
    Ok(path_str)
}

#[tauri::command]
async fn get_app_version() -> Result<String, String> {
    Ok(env!("CARGO_PKG_VERSION").into())
}

#[tauri::command]
async fn check_update() -> Result<String, String> {
    let current = env!("CARGO_PKG_VERSION").to_string();

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let html = client
        .get("https://github.com/evolutions-code/evocode-tauri/releases")
        .header(
            "User-Agent",
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
        )
        .send()
        .await
        .map_err(|_| "Network error: cannot reach GitHub".to_string())?
        .text()
        .await
        .map_err(|_| "Failed to read response".to_string())?;

    // Find all "evocode-vX.Y.Z" patterns in the HTML
    let mut latest = String::new();
    let mut pos = 0;
    let bytes = html.as_bytes();
    let prefix = b"evocode-v";

    while pos + prefix.len() < bytes.len() {
        if bytes[pos..].starts_with(prefix) {
            let start = pos + prefix.len();
            let mut end = start;
            while end < bytes.len()
                && bytes[end] != b'<'
                && bytes[end] != b'"'
                && bytes[end] != b' '
                && bytes[end] != b'>'
            {
                end += 1;
            }
            let ver_str = std::str::from_utf8(&bytes[start..end]).unwrap_or("");
            let parts: Vec<u64> = ver_str.split('.').filter_map(|s| s.parse().ok()).collect();
            if parts.len() == 3 {
                let latest_parts: Vec<u64> =
                    latest.split('.').filter_map(|s| s.parse().ok()).collect();
                if parts > latest_parts {
                    latest = ver_str.to_string();
                }
            }
            pos = end;
        } else {
            pos += 1;
        }
    }

    if latest.is_empty() {
        return Err("No releases found".to_string());
    }

    let semver_latest: Vec<u64> = latest.split('.').filter_map(|s| s.parse().ok()).collect();
    let semver_current: Vec<u64> = current.split('.').filter_map(|s| s.parse().ok()).collect();

    let release_url = format!(
        "https://github.com/evolutions-code/evocode-tauri/releases/tag/v{}",
        latest
    );

    if semver_latest > semver_current {
        Ok(format!(
            "update_available__{}__{}__{}",
            latest, current, release_url
        ))
    } else {
        Ok(format!("no_update__{}", current))
    }
}

#[tauri::command]
async fn fetch_models(
    base_url: String,
    api_key: String,
    wire_api: String,
    api_key_header: Option<String>,
) -> Result<Vec<String>, String> {
    let fetcher = traits::model_fetcher_for_wire_api(&wire_api);
    let models = fetcher
        .fetch_models(&base_url, &api_key, api_key_header.as_deref())
        .await?;
    Ok(models.into_iter().map(|m| m.id).collect())
}









const MAIN_WINDOW_LABEL: &str = "main";
const TRAY_ID: &str = "evocode-tray";
const MENU_SHOW: &str = "tray_show";
const MENU_START: &str = "tray_start";
const MENU_STOP: &str = "tray_stop";
const MENU_QUIT: &str = "tray_quit";

fn load_tray_icon() -> Image<'static> {
    Image::from_bytes(include_bytes!("../icons/32x32.png")).expect("valid tray icon")
}

fn show_main_window(app: &AppHandle) {
    if let Some(win) = app.get_webview_window(MAIN_WINDOW_LABEL) {
        #[cfg(target_os = "macos")]
        let _ = app.set_dock_visibility(true);
        let _ = win.show();
        let _ = win.unminimize();
        let _ = win.set_focus();
    }
}

fn build_tray_menu(app: &AppHandle) -> tauri::Result<Menu<tauri::Wry>> {
    let show = MenuItem::with_id(app, MENU_SHOW, "Show Window", true, None::<&str>)?;
    let start = MenuItem::with_id(app, MENU_START, "Start Bridge", true, None::<&str>)?;
    let stop = MenuItem::with_id(app, MENU_STOP, "Stop Bridge", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, MENU_QUIT, "Quit", true, None::<&str>)?;
    let sep = PredefinedMenuItem::separator(app)?;
    Menu::with_items(app, &[&show, &sep, &start, &stop, &sep, &quit])
}

fn handle_tray_menu_event(app: &AppHandle, event: MenuEvent) {
    match event.id().as_ref() {
        MENU_SHOW => show_main_window(app),
        MENU_QUIT => {
            app.exit(0);
        }
        MENU_START | MENU_STOP => {
            show_main_window(app);
            let action = if event.id().as_ref() == MENU_START {
                "start"
            } else {
                "stop"
            };
            let _ = app.emit("tray-bridge-action", action);
        }
        _ => {}
    }
}

fn handle_tray_icon_event(app: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        show_main_window(app);
    }
}

fn handle_window_event(window: &tauri::Window, event: &tauri::WindowEvent) {
    if let tauri::WindowEvent::CloseRequested { api, .. } = event
        && window.label() == MAIN_WINDOW_LABEL
    {
        api.prevent_close();
        #[cfg(target_os = "macos")]
        let _ = window.app_handle().set_dock_visibility(false);
        let _ = window.hide();
    }
}

/// Absolute path to the bridge log file (~/.evocode/logs/temp.log).
/// Check whether the bridge is reachable by attempting a TCP connect
/// to the configured port. Returns latency in milliseconds on success.
#[tauri::command]
async fn bridge_ping(state: State<'_, BridgeState>) -> Result<u64, String> {
    let port = state.port.load(std::sync::atomic::Ordering::Relaxed);
    let addr: std::net::SocketAddr = format!("127.0.0.1:{}", port)
        .parse()
        .map_err(|e| format!("Invalid address: {}", e))?;
    let start = std::time::Instant::now();
    tokio::task::spawn_blocking(move || {
        match std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_secs(2)) {
            Ok(_) => Ok(start.elapsed().as_millis() as u64),
            Err(e) => Err(format!("Bridge not reachable: {}", e)),
        }
    })
    .await
    .map_err(|e| format!("Ping task failed: {}", e))?
}
fn bridge_log_path() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let dir = home.join(".evocode").join("logs");
    let _ = std::fs::create_dir_all(&dir);
    dir.join("temp.log")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Init tracing-based logging to ~/.evocode/logs/temp.log
    let log_path = bridge_log_path();
    println!("Logging to {}", log_path.display());
    // Remove old log file before this run
    let _ = std::fs::remove_file(&log_path);
    // Initialize tracing subscriber with file output
    let _ = tracing_subscriber::fmt()
        .with_timer(LocalTimer)
        .with_ansi(false)
        .with_writer({
            let log_path = log_path.clone();
            move || {
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(&log_path)
                    .expect("failed to open log file")
            }
        })
        .with_env_filter(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy()
        )
        .try_init();
    let mut builder = tauri::Builder::default();

    #[cfg(desktop)]
    {
        builder = builder.plugin(tauri_plugin_single_instance::init(|app, _argv, _cwd| {
            // A second instance was launched: focus the existing main window.
            show_main_window(app);
        }));
    }

    builder
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .manage(BridgeState::new(BridgeState::load_port_from_config()))
        .invoke_handler(tauri::generate_handler![
            start_bridge,
            bridge_ping,
            stop_bridge,
            bridge_is_running,
            bridge_status,
            get_bridge_url,
            get_bridge_port,
            set_bridge_port,
            set_max_request_body_size,
            read_config,
            read_config_json,
            sync_to_codex,
            sync_model_to_codex,
            get_bridge_logs,
            get_bridge_logs_tail,
            get_bridge_logs_range,
            get_app_version,
            check_update,
            codex::get_sessions,
            codex::get_session_content,
            test_provider_connectivity,
            open_config_dir,
            fetch_models,
            save_config,
            prompts::read_agents_file,
            prompts::read_core_prompt,
            prompts::write_agents_file,
            prompts::list_prompt_files,
            prompts::write_prompt_file,
            prompts::read_prompt_file,
            prompts::delete_prompt_file,
            
])
        .on_window_event(handle_window_event)
        .setup(|app| {
            let handle = app.handle().clone();
            let icon = load_tray_icon();
            let menu = build_tray_menu(&handle)?;
            let _tray = TrayIconBuilder::with_id(TRAY_ID)
                .icon(icon)
                .tooltip("evocode")
                .menu(&menu)
                .show_menu_on_left_click(false)
                .on_menu_event(handle_tray_menu_event)
                .on_tray_icon_event(|tray, event| {
                    let app = tray.app_handle();
                    handle_tray_icon_event(app, event);
                })
                .build(app)?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

