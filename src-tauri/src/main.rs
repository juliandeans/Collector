#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unexpected_cfgs)]

mod capture;
mod edge_detect;
mod image_handler;
mod log_safety;
mod selected_text;
mod settings;
mod shortcuts;
mod vault_index;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, LogicalPosition, LogicalSize, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::RwLock;

#[cfg(target_os = "macos")]
use cocoa::base::{id, BOOL, YES};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

use crate::edge_detect::EdgeDetector;
use crate::image_handler::ProcessedImage;
use crate::settings::Settings;
use crate::shortcuts::ShortcutManager;
use std::fs;
use std::path::Component;

const SAVE_AS_NOTE_HIDE_DELAY_MS: u64 = 50;
const WINDOW_INIT_DELAY_MS: u64 = 100;
const SETTINGS_WINDOW_WIDTH: f64 = 980.0;
const SETTINGS_WINDOW_HEIGHT: f64 = 720.0;
const MAX_IMAGE_PAYLOAD_BYTES: usize = 20 * 1024 * 1024;

struct AppState {
    settings: Arc<RwLock<Settings>>,
    edge_detector: Arc<EdgeDetector>,
    shortcut_manager: Arc<ShortcutManager>,
    reader_shortcut_manager: Arc<ShortcutManager>,
    capture_text_shortcut_manager: Arc<ShortcutManager>,
    save_as_note_shortcut_manager: Arc<ShortcutManager>,
    vault_index: Arc<RwLock<Option<vault_index::VaultIndex>>>,
}

fn warn_if_failed<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    if let Err(error) = result {
        log::warn!("{}: {}", context, error);
    }
}

fn normalize_path(path: &Path) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("Path traversal is not allowed".to_string());
                }
            }
        }
    }

    Ok(normalized)
}

fn canonical_vault_root(settings: &Settings) -> Result<PathBuf, String> {
    let vault_path = PathBuf::from(&settings.vault_path);
    fs::canonicalize(&vault_path).map_err(|e| format!("Failed to resolve vault path: {}", e))
}

fn ensure_markdown_path(path: &Path) -> Result<(), String> {
    if path.extension().map(|ext| ext == "md").unwrap_or(false) {
        Ok(())
    } else {
        Err("Only Markdown files inside the vault are allowed".to_string())
    }
}

fn resolve_vault_read_path(settings: &Settings, requested_path: &str) -> Result<PathBuf, String> {
    let vault_root = canonical_vault_root(settings)?;
    let normalized = normalize_path(Path::new(requested_path))?;
    let candidate = if normalized.is_absolute() {
        normalized
    } else {
        vault_root.join(normalized)
    };
    let canonical = fs::canonicalize(&candidate)
        .map_err(|e| format!("Failed to resolve requested file path: {}", e))?;

    ensure_markdown_path(&canonical)?;

    if canonical.starts_with(&vault_root) {
        Ok(canonical)
    } else {
        Err("Requested file is outside the vault".to_string())
    }
}

fn resolve_vault_write_path(settings: &Settings, requested_path: &str) -> Result<PathBuf, String> {
    let vault_root = canonical_vault_root(settings)?;
    let normalized = normalize_path(Path::new(requested_path))?;
    let candidate = if normalized.is_absolute() {
        normalized
    } else {
        vault_root.join(normalized)
    };

    ensure_markdown_path(&candidate)?;

    if !candidate.starts_with(&vault_root) {
        return Err("Requested file is outside the vault".to_string());
    }

    let parent = candidate
        .parent()
        .ok_or_else(|| "Invalid note path".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("Failed to create note directory: {}", e))?;

    let canonical_parent =
        fs::canonicalize(parent).map_err(|e| format!("Failed to resolve note directory: {}", e))?;
    if !canonical_parent.starts_with(&vault_root) {
        return Err("Requested file is outside the vault".to_string());
    }

    let filename = candidate
        .file_name()
        .ok_or_else(|| "Invalid note filename".to_string())?;

    Ok(canonical_parent.join(filename))
}

pub(crate) fn build_image_data_url(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|e| format!("Failed to read image: {}", e))?;
    let mime = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };

    use base64::{engine::general_purpose, Engine as _};
    Ok(format!(
        "data:{};base64,{}",
        mime,
        general_purpose::STANDARD.encode(bytes)
    ))
}

async fn get_or_build_index(state: &tauri::State<'_, AppState>) -> Result<(), String> {
    let vault_path = {
        let settings = state.settings.read().await;
        settings.vault_path.clone()
    };
    let canonical_vault_path = fs::canonicalize(&vault_path)
        .unwrap_or_else(|_| PathBuf::from(&vault_path))
        .to_string_lossy()
        .to_string();

    {
        let index = state.vault_index.read().await;
        if let Some(ref idx) = *index {
            if idx.vault_path == canonical_vault_path {
                return Ok(());
            }
        }
    }

    log::info!(
        "Building vault index for: {}",
        crate::log_safety::redact_path_str(&vault_path)
    );
    let new_index = vault_index::VaultIndex::build(&vault_path)?;
    let build_duration_ms = new_index.built_at.elapsed().as_millis();
    let _ = new_index.resolve_note("");
    log::info!(
        "Vault index built: {} files in {} ms",
        new_index.file_count,
        build_duration_ms
    );

    *state.vault_index.write().await = Some(new_index);
    Ok(())
}

#[tauri::command]
async fn load_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    let settings = Settings::load()?;

    *state.settings.write().await = settings.clone();

    Ok(settings)
}

#[tauri::command]
async fn save_settings(
    new_settings: Settings,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    new_settings.validate().map_err(|e| {
        log::error!("Settings validation failed: {}", e);
        e
    })?;

    new_settings.save().map_err(|e| {
        log::error!("Failed to save settings to file: {}", e);
        e
    })?;

    match Settings::load() {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to verify saved settings: {}", e);
        }
    }

    let old_vault_path = {
        let old = state.settings.read().await;
        old.vault_path.clone()
    };

    if old_vault_path != new_settings.vault_path {
        *state.vault_index.write().await = None;
        log::info!("Vault index invalidated: vault path changed");
    }

    *state.settings.write().await = new_settings.clone();

    state
        .edge_detector
        .update_settings(new_settings.clone())
        .await;

    if let Some(window) = app.get_webview_window("capture") {
        if let Err(e) = position_window_logical(&window, &new_settings) {
            log::warn!("Failed to update window position: {}", e);
        }

        configure_macos_window(&window, new_settings.border_radius as f64);
    }

    if let Some(reader_window) = app.get_webview_window("reader") {
        if let Err(e) = position_reader_window_logical(&reader_window, &new_settings) {
            log::warn!("Failed to update reader window position: {}", e);
        }

        configure_macos_window(&reader_window, new_settings.border_radius as f64);
    }

    // Emit settings_changed before shortcuts so visual changes apply even if shortcuts fail.
    if let Some(capture_window) = app.get_webview_window("capture") {
        if let Err(e) = capture_window.emit("settings_changed", &new_settings) {
            log::warn!("Failed to emit settings_changed to capture window: {}", e);
        }
    }

    if let Some(settings_window) = app.get_webview_window("settings") {
        if let Err(e) = settings_window.emit("settings_changed", &new_settings) {
            log::warn!("Failed to emit settings_changed to settings window: {}", e);
        }
    }

    if let Some(reader_window) = app.get_webview_window("reader") {
        if let Err(e) = reader_window.emit("settings_changed", &new_settings) {
            log::warn!("Failed to emit settings_changed to reader window: {}", e);
        }
    }

    if let Err(e) = state.shortcut_manager.update(&app, &new_settings).await {
        log::warn!("Failed to update global shortcut (non-fatal): {}", e);
    }

    if let Err(e) = state
        .reader_shortcut_manager
        .register_reader(&app, &new_settings)
        .await
    {
        log::warn!("Failed to update reader shortcut (non-fatal): {}", e);
    }

    if let Err(e) = state
        .capture_text_shortcut_manager
        .register_capture_text(&app, &new_settings)
        .await
    {
        log::warn!("Failed to update capture_text shortcut (non-fatal): {}", e);
    }

    if let Err(e) = state
        .save_as_note_shortcut_manager
        .register_save_as_note(&app, &new_settings)
        .await
    {
        log::warn!("Failed to update save_as_note shortcut (non-fatal): {}", e);
    }

    Ok(())
}

#[tauri::command]
async fn save_as_note(
    content: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;
    let relative_path = capture::build_note_relative_path(&settings);
    let resolved = resolve_vault_write_path(&settings, &relative_path)?;
    let filename = resolved
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid note filename".to_string())?
        .to_string();

    state.edge_detector.set_capture_open(false).await;

    if let Some(window) = app.get_webview_window("capture") {
        warn_if_failed(window.hide(), "Failed to hide capture window");
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(
        SAVE_AS_NOTE_HIDE_DELAY_MS,
    ))
    .await;

    let result = capture::save_note_at_path(&content.trim(), &resolved, &filename, &settings)?;

    Ok(result.message)
}

#[tauri::command]
async fn append_to_daily_note(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;
    let daily_path = capture::build_daily_note_path(&settings);
    let resolved = resolve_vault_write_path(&settings, &daily_path)?;

    capture::append_to_daily_note(&text, &resolved, &settings)?;

    Ok(())
}

#[tauri::command]
async fn read_note_file(path: String, state: tauri::State<'_, AppState>) -> Result<String, String> {
    let settings = state.settings.read().await.clone();
    let resolved = resolve_vault_read_path(&settings, &path)?;
    fs::read_to_string(&resolved).map_err(|e| format!("Failed to read file: {}", e))
}

#[tauri::command]
async fn write_note_file(
    path: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    let resolved = resolve_vault_write_path(&settings, &path)?;
    fs::write(&resolved, content).map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
async fn open_external_url(url: String) -> Result<(), String> {
    let allowed = ["http://", "https://", "obsidian://"];
    if !allowed
        .iter()
        .any(|prefix| url.to_ascii_lowercase().starts_with(prefix))
    {
        return Err("Only http, https and obsidian links are allowed".to_string());
    }

    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

#[tauri::command]
async fn get_running_apps() -> Result<Vec<String>, String> {
    use std::process::Command;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(
            r#"tell application "System Events" to get name of every process where background only is false"#,
        )
        .output()
        .map_err(|e| format!("Failed to get running apps: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let apps = stdout
        .split(',')
        .map(|app| app.trim().to_string())
        .filter(|app| !app.is_empty())
        .collect();

    Ok(apps)
}

#[tauri::command]
async fn load_image_data_url(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    get_or_build_index(&state).await?;

    let resolved = {
        let index = state.vault_index.read().await;
        let idx = index
            .as_ref()
            .ok_or_else(|| "Vault index not available".to_string())?;

        idx.resolve_image(&path)
            .cloned()
            .ok_or_else(|| format!("Image not found: {}", path))?
    };

    build_image_data_url(&resolved)
}

#[tauri::command]
async fn load_images_batch(
    paths: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(String, String)>, String> {
    get_or_build_index(&state).await?;

    let index = state.vault_index.read().await;
    let idx = index
        .as_ref()
        .ok_or_else(|| "Vault index not available".to_string())?;

    let mut results = Vec::new();

    for path in paths {
        let clean = path.split('|').next().unwrap_or("").trim().to_string();
        if clean.is_empty() {
            continue;
        }

        match idx.resolve_image(&clean) {
            Some(resolved) => {
                if let Ok(data_url) = build_image_data_url(resolved) {
                    results.push((clean, data_url));
                }
            }
            None => {}
        }
    }

    Ok(results)
}

#[tauri::command]
async fn list_vault_notes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<vault_index::NoteEntry>, String> {
    get_or_build_index(&state).await?;

    let index = state.vault_index.read().await;
    let idx = index
        .as_ref()
        .ok_or_else(|| "Vault index not available".to_string())?;

    Ok(idx.all_notes())
}

#[tauri::command]
async fn get_daily_note_path(state: tauri::State<'_, AppState>) -> Result<String, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;
    let daily_path = capture::build_daily_note_path(&settings);
    let file_path = resolve_vault_write_path(&settings, &daily_path)?;
    Ok(file_path.to_string_lossy().to_string())
}

#[tauri::command]
async fn reindex_vault(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let vault_path = {
        let settings = state.settings.read().await;
        settings.vault_path.clone()
    };

    log::info!("Manual vault reindex requested");
    let new_index = vault_index::VaultIndex::build(&vault_path)?;
    let count = new_index.file_count;
    let build_duration_ms = new_index.built_at.elapsed().as_millis();
    let _ = new_index.resolve_note("");
    *state.vault_index.write().await = Some(new_index);
    log::info!(
        "Vault reindex complete: {} files in {} ms",
        count,
        build_duration_ms
    );

    Ok(count)
}

#[tauri::command]
async fn save_image(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessedImage, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;

    let result = image_handler::process_dropped_file(&file_path, &settings)?;

    Ok(result)
}

#[tauri::command]
async fn save_image_from_bytes(
    bytes_base64: String,
    filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessedImage, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;

    if bytes_base64.len() > MAX_IMAGE_PAYLOAD_BYTES {
        return Err("Image payload too large".to_string());
    }

    use base64::{engine::general_purpose, Engine as _};
    let bytes = general_purpose::STANDARD
        .decode(&bytes_base64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let result = image_handler::process_dropped_file_from_bytes(bytes, &filename, &settings)?;

    Ok(result)
}

#[tauri::command]
async fn toggle_edge_detection(
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.edge_detector.set_enabled(enabled).await;

    let mut settings = state.settings.write().await;
    settings.edge_detection_enabled = enabled;
    settings.save()?;

    Ok(())
}

#[tauri::command]
async fn set_autostart(enabled: bool, app: AppHandle) -> Result<(), String> {
    let manager = app.autolaunch();

    if enabled {
        manager
            .enable()
            .map_err(|e| format!("Failed to enable autostart: {}", e))?;
    } else {
        manager
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
async fn is_autostart_enabled(app: AppHandle) -> Result<bool, String> {
    let manager = app.autolaunch();
    manager
        .is_enabled()
        .map_err(|e| format!("Failed to check autostart status: {}", e))
}

#[tauri::command]
async fn hide_capture(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    state.edge_detector.set_capture_open(false).await;

    if let Some(window) = app.get_webview_window("capture") {
        warn_if_failed(window.hide(), "Failed to hide capture window");
    }

    Ok(())
}

#[tauri::command]
async fn show_capture(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("capture") {
        let settings = state.settings.read().await;
        position_window_logical(&window, &settings)?;
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
    }
    state.edge_detector.set_capture_open(true).await;
    Ok(())
}

async fn show_capture_internal(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(window) = app.get_webview_window("capture") {
        let settings = state.settings.read().await;
        position_window_logical(&window, &settings)?;
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
    }
    state.edge_detector.set_capture_open(true).await;
    Ok(())
}

async fn show_reader_internal(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(window) = app.get_webview_window("reader") {
        let settings = state.settings.read().await;
        position_reader_window_logical(&window, &settings)?;
        configure_macos_window(&window, settings.border_radius as f64);
        warn_if_failed(window.show(), "Failed to show reader window");
        warn_if_failed(window.set_focus(), "Failed to focus reader window");
    }
    state.edge_detector.set_reader_open(true).await;
    Ok(())
}

#[tauri::command]
async fn show_reader(app: AppHandle, _state: tauri::State<'_, AppState>) -> Result<(), String> {
    show_reader_internal(&app).await
}

#[tauri::command]
async fn hide_reader(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("reader") {
        warn_if_failed(window.hide(), "Failed to hide reader window");
    }
    state.edge_detector.set_reader_open(false).await;
    Ok(())
}

#[tauri::command]
async fn get_window_info(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let settings = state.settings.read().await;
    Ok(serde_json::json!({
        "width": settings.window_width,
        "height": settings.window_height,
        "readerWidth": settings.reader_width,
        "readerHeight": settings.reader_height,
        "borderRadius": settings.border_radius,
        "backgroundColor": settings.background_color,
        "fontFamily": settings.font_family,
        "fontSize": settings.font_size,
        "edgeSide": settings.edge_side,
    }))
}

#[tauri::command]
async fn open_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        warn_if_failed(window.show(), "Failed to show settings window");
        warn_if_failed(window.center(), "Failed to center settings window");
        warn_if_failed(window.set_focus(), "Failed to focus settings window");
        warn_if_failed(window.unminimize(), "Failed to unminimize settings window");
    } else {
        use tauri::WebviewUrl;
        use tauri::WebviewWindowBuilder;

        let settings_window =
            WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
                .title("Collector - Settings")
                .inner_size(SETTINGS_WINDOW_WIDTH, SETTINGS_WINDOW_HEIGHT)
                .resizable(true)
                .center()
                .build()
                .map_err(|e| format!("Failed to create settings window: {}", e))?;

        warn_if_failed(settings_window.show(), "Failed to show settings window");
        warn_if_failed(
            settings_window.set_focus(),
            "Failed to focus settings window",
        );
    }
    Ok(())
}

#[tauri::command]
async fn close_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        warn_if_failed(window.hide(), "Failed to hide settings window");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_macos_window(window: &tauri::WebviewWindow, corner_radius: f64) {
    let ns_window_ptr = match window.ns_window() {
        Ok(ptr) => ptr,
        Err(e) => {
            log::warn!(
                "Failed to get ns_window: {} - window might not be fully initialized yet",
                e
            );
            return;
        }
    };

    unsafe {
        let ns_window = ns_window_ptr as id;

        if ns_window.is_null() {
            log::warn!("NSWindow pointer is null - cannot configure window");
            return;
        }

        let content_view: id = msg_send![ns_window, contentView];
        if content_view.is_null() {
            log::warn!("Content view is null - cannot configure window");
            return;
        }

        let _: () = msg_send![content_view, setWantsLayer: YES];

        let layer: id = msg_send![content_view, layer];
        if layer.is_null() {
            log::warn!("Layer is null - cannot configure window");
            return;
        }

        let _: () = msg_send![layer, setCornerRadius: corner_radius];
        let _: () = msg_send![layer, setMasksToBounds: YES];

        let sublayers: id = msg_send![layer, sublayers];
        if !sublayers.is_null() {
            let count: usize = msg_send![sublayers, count];
            for i in 0..count {
                let sublayer: id = msg_send![sublayers, objectAtIndex: i];
                if !sublayer.is_null() {
                    let _: () = msg_send![sublayer, setCornerRadius: corner_radius];
                    let _: () = msg_send![sublayer, setMasksToBounds: YES];
                }
            }
        }

        let _: () = msg_send![ns_window, setOpaque: BOOL::from(false)];

        let clear_color: id = msg_send![class!(NSColor), clearColor];
        let _: () = msg_send![ns_window, setBackgroundColor: clear_color];

        let _: () = msg_send![ns_window, setHasShadow: YES];

        let _: () = msg_send![ns_window, invalidateShadow];
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_macos_window(_window: &tauri::WebviewWindow, _corner_radius: f64) {}

fn position_window_logical(
    window: &tauri::WebviewWindow,
    settings: &Settings,
) -> Result<(), String> {
    let (screen_width, screen_height) = edge_detect::get_screen_bounds();

    let width = settings.window_width as f64;
    let height = settings.window_height as f64;

    let y = (screen_height as f64 - height) / 2.0;

    let x = match settings.edge_side.as_str() {
        "left" => 0.0,
        "right" | _ => screen_width as f64 - width,
    };

    window
        .set_size(LogicalSize { width, height })
        .map_err(|e| e.to_string())?;

    window
        .set_position(LogicalPosition { x, y })
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn position_reader_window_logical(
    window: &tauri::WebviewWindow,
    settings: &Settings,
) -> Result<(), String> {
    let (_, screen_height) = edge_detect::get_screen_bounds();
    let width = settings.reader_width as f64;
    let height = settings.reader_height as f64;
    let y = ((screen_height as f64 - height) / 2.0).max(0.0);

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;

    window
        .set_position(LogicalPosition::new(0.0, y))
        .map_err(|e| e.to_string())?;

    Ok(())
}

fn create_tray_menu(app: &AppHandle) -> Menu<tauri::Wry> {
    let quick_capture = MenuItem::with_id(
        app,
        "quick_capture",
        "Quick Capture (⌘⇧N)",
        true,
        None::<String>,
    )
    .unwrap();
    let settings = MenuItem::with_id(app, "settings", "Settings...", true, None::<String>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<String>).unwrap();

    Menu::with_items(
        app,
        &[
            &quick_capture,
            &settings,
            &PredefinedMenuItem::separator(app).unwrap(),
            &quit,
        ],
    )
    .unwrap()
}

fn handle_tray_event(app: &AppHandle, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button,
        button_state,
        ..
    } = event
    {
        if button == MouseButton::Left && button_state == MouseButtonState::Up {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle.state::<AppState>();
                let settings = state.settings.read().await.clone();
                show_capture_window(&app_handle, &settings);
            });
        }
    }
}

fn show_capture_window(app: &AppHandle, settings: &Settings) {
    if let Some(window) = app.get_webview_window("capture") {
        warn_if_failed(
            position_window_logical(&window, settings),
            "Failed to position capture window",
        );
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
        warn_if_failed(app.emit("show_capture", ()), "Failed to emit show_capture");
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Quick Capture...");

    let settings = Settings::load().unwrap_or_else(|e| {
        log::error!("Failed to load settings: {}, using defaults", e);
        Settings::default()
    });

    let edge_detector = Arc::new(EdgeDetector::new(settings.clone()));
    let shortcut_manager = Arc::new(ShortcutManager::new());
    let reader_shortcut_manager = Arc::new(ShortcutManager::new());
    let capture_text_shortcut_manager = Arc::new(ShortcutManager::new());
    let save_as_note_shortcut_manager = Arc::new(ShortcutManager::new());

    let app_state = AppState {
        settings: Arc::new(RwLock::new(settings.clone())),
        edge_detector: edge_detector.clone(),
        shortcut_manager: shortcut_manager.clone(),
        reader_shortcut_manager: reader_shortcut_manager.clone(),
        capture_text_shortcut_manager: capture_text_shortcut_manager.clone(),
        save_as_note_shortcut_manager: save_as_note_shortcut_manager.clone(),
        vault_index: Arc::new(RwLock::new(None)),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--hidden"]),
        ))
        .setup(move |app| {
            let app_handle = app.handle().clone();
            app_handle.listen("show_capture", {
                let app_handle = app_handle.clone();
                move |_| {
                    let app = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(error) = show_capture_internal(&app).await {
                            log::warn!("Failed to show capture window: {}", error);
                        }
                    });
                }
            });
            app_handle.listen("show_reader", {
                let app_handle = app_handle.clone();
                move |_| {
                    let app = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(error) = show_reader_internal(&app).await {
                            log::warn!("Failed to show reader window: {}", error);
                        }
                    });
                }
            });

            let menu = create_tray_menu(&app_handle);
            let settings_for_tray = settings.clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quick_capture" => {
                        show_capture_window(app, &settings_for_tray);
                    }
                    "settings" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(error) = open_settings(app_clone).await {
                                log::warn!("Failed to open settings window: {}", error);
                            }
                        });
                    }
                    "quit" => std::process::exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    handle_tray_event(_tray.app_handle(), event);
                })
                .build(app)?;

            let detector = edge_detector.clone();
            let app_handle_edge = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                detector.start(app_handle_edge).await;
            });

            let shortcut_mgr = shortcut_manager.clone();
            let app_handle_shortcut = app_handle.clone();
            let settings_for_shortcut = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = shortcut_mgr
                    .register(&app_handle_shortcut, &settings_for_shortcut)
                    .await
                {
                    log::error!("Failed to register shortcut: {}", e);
                }
            });

            let capture_text_mgr = capture_text_shortcut_manager.clone();
            let app_handle_capture_text = app_handle.clone();
            let settings_for_capture_text = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = capture_text_mgr
                    .register_capture_text(&app_handle_capture_text, &settings_for_capture_text)
                    .await
                {
                    log::error!("Failed to register capture_text shortcut: {}", e);
                }
            });

            let reader_shortcut_mgr = reader_shortcut_manager.clone();
            let app_handle_reader_shortcut = app_handle.clone();
            let settings_for_reader_shortcut = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = reader_shortcut_mgr
                    .register_reader(&app_handle_reader_shortcut, &settings_for_reader_shortcut)
                    .await
                {
                    log::error!("Failed to register reader shortcut: {}", e);
                }
            });

            let save_as_note_mgr = save_as_note_shortcut_manager.clone();
            let app_handle_save_as_note = app_handle.clone();
            let settings_for_save_as_note = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = save_as_note_mgr
                    .register_save_as_note(&app_handle_save_as_note, &settings_for_save_as_note)
                    .await
                {
                    log::error!("Failed to register save_as_note shortcut: {}", e);
                }
            });

            let app_handle_index = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                let state = app_handle_index.state::<AppState>();
                let vault_path = {
                    let settings = state.settings.read().await;
                    settings.vault_path.clone()
                };

                if !vault_path.is_empty() {
                    match vault_index::VaultIndex::build(&vault_path) {
                        Ok(index) => {
                            *state.vault_index.write().await = Some(index);
                            log::info!("Vault index ready at startup");
                        }
                        Err(error) => {
                            log::warn!("Startup vault index failed: {}", error);
                        }
                    }
                }
            });

            if let Some(window) = app.get_webview_window("capture") {
                warn_if_failed(
                    position_window_logical(&window, &settings),
                    "Failed to position capture window",
                );

                let window_clone = window.clone();
                let border_radius = settings.border_radius;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_INIT_DELAY_MS))
                        .await;
                    configure_macos_window(&window_clone, border_radius as f64);
                });
            }

            if let Some(window) = app.get_webview_window("reader") {
                warn_if_failed(
                    position_reader_window_logical(&window, &settings),
                    "Failed to position reader window",
                );

                let window_clone = window.clone();
                let border_radius = settings.border_radius;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_INIT_DELAY_MS))
                        .await;
                    configure_macos_window(&window_clone, border_radius as f64);
                });
            }

            log::info!("Quick Capture setup complete");
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            load_settings,
            save_settings,
            save_as_note,
            append_to_daily_note,
            read_note_file,
            write_note_file,
            open_external_url,
            get_running_apps,
            load_image_data_url,
            load_images_batch,
            list_vault_notes,
            get_daily_note_path,
            reindex_vault,
            save_image,
            save_image_from_bytes,
            toggle_edge_detection,
            set_autostart,
            is_autostart_enabled,
            hide_capture,
            show_capture,
            show_reader,
            hide_reader,
            get_window_info,
            open_settings,
            close_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault_dir() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("collector-test-{}", suffix));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn normalize_path_rejects_escape() {
        assert!(normalize_path(Path::new("../../secret.md")).is_err());
    }

    #[test]
    fn resolve_vault_read_path_rejects_outside_file() {
        let vault_dir = temp_vault_dir();
        let note_path = vault_dir.join("safe.md");
        fs::write(&note_path, "# Safe").unwrap();

        let settings = Settings {
            vault_path: vault_dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let result = resolve_vault_read_path(&settings, "../safe.md");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(vault_dir);
    }

    #[test]
    fn resolve_vault_write_path_rejects_outside_file() {
        let vault_dir = temp_vault_dir();
        let settings = Settings {
            vault_path: vault_dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let result = resolve_vault_write_path(&settings, "../outside.md");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(vault_dir);
    }
}
