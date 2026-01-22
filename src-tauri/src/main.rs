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

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager,
};
use tauri_plugin_autostart::ManagerExt;
use tokio::sync::RwLock;

#[cfg(target_os = "macos")]
use cocoa::base::{id, BOOL, YES};
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};

use crate::edge_detect::EdgeDetector;
use crate::image_handler::ProcessedImage;
use crate::log_safety::{redact_path_str, summarize_bytes};
use crate::settings::Settings;
use crate::shortcuts::ShortcutManager;

struct AppState {
    settings: Arc<RwLock<Settings>>,
    edge_detector: Arc<EdgeDetector>,
    shortcut_manager: Arc<ShortcutManager>,
    capture_text_shortcut_manager: Arc<ShortcutManager>,
    save_as_note_shortcut_manager: Arc<ShortcutManager>,
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
    log::info!("save_settings received settings payload");

    new_settings.validate().map_err(|e| {
        log::error!("Settings validation failed: {}", e);
        e
    })?;

    new_settings.save().map_err(|e| {
        log::error!("Failed to save settings to file: {}", e);
        e
    })?;

    match Settings::load() {
        Ok(loaded) => {
            log::info!(
                "Settings verified (window_blur={}, window_transparency={}, window_saturation={})",
                loaded.window_blur,
                loaded.window_transparency,
                loaded.window_saturation
            );
        }
        Err(e) => {
            log::error!("Failed to verify saved settings: {}", e);
        }
    }

    *state.settings.write().await = new_settings.clone();

    state
        .edge_detector
        .update_settings(new_settings.clone())
        .await;

    if let Some(window) = app.get_webview_window("capture") {
        if let Err(e) = position_window_logical(&window, &new_settings) {
            log::warn!("Failed to update window position: {}", e);
        } else {
            log::info!("Window position/size updated");
        }

        configure_macos_window(&window, new_settings.border_radius as f64);
    }

    // Emit settings_changed before shortcuts so visual changes apply even if shortcuts fail.
    if let Some(capture_window) = app.get_webview_window("capture") {
        if let Err(e) = capture_window.emit("settings_changed", &new_settings) {
            log::warn!("Failed to emit settings_changed to capture window: {}", e);
        } else {
            log::info!("Settings changed event emitted to capture window");
        }
    }

    if let Some(settings_window) = app.get_webview_window("settings") {
        if let Err(e) = settings_window.emit("settings_changed", &new_settings) {
            log::warn!("Failed to emit settings_changed to settings window: {}", e);
        } else {
            log::info!("Settings changed event emitted to settings window");
        }
    }

    match state.shortcut_manager.update(&app, &new_settings).await {
        Ok(_) => log::info!("Global shortcut updated"),
        Err(e) => log::warn!("Failed to update global shortcut (non-fatal): {}", e),
    }

    match state
        .capture_text_shortcut_manager
        .register_capture_text(&app, &new_settings)
        .await
    {
        Ok(_) => log::info!("Capture text shortcut updated"),
        Err(e) => log::warn!("Failed to update capture_text shortcut (non-fatal): {}", e),
    }

    match state
        .save_as_note_shortcut_manager
        .register_save_as_note(&app, &new_settings)
        .await
    {
        Ok(_) => log::info!("Save as note shortcut updated"),
        Err(e) => log::warn!("Failed to update save_as_note shortcut (non-fatal): {}", e),
    }

    log::info!("Settings updated successfully");
    Ok(())
}

#[tauri::command]
async fn save_as_note(
    content: String,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await.clone();

    state.edge_detector.set_window_open(false).await;

    if let Some(window) = app.get_webview_window("capture") {
        let _ = window.hide();
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

    let result = capture::save_as_note(&content.trim(), &settings)?;

    log::info!("Content saved as note");
    Ok(result.message)
}

#[tauri::command]
async fn append_to_daily_note(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();

    capture::append_to_daily_note(&text, &settings)?;

    log::info!("Content appended to daily note");
    Ok(())
}

#[tauri::command]
async fn save_image(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessedImage, String> {
    let settings = state.settings.read().await.clone();

    let result = image_handler::process_dropped_file(&file_path, &settings)?;

    log::info!("Image saved (link_chars={})", result.markdown.chars().count());
    Ok(result)
}

#[tauri::command]
async fn save_image_from_bytes(
    bytes_base64: String,
    filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessedImage, String> {
    let settings = state.settings.read().await.clone();

    log::info!(
        "Received base64 string (file={}, chars={})",
        redact_path_str(&filename),
        bytes_base64.len()
    );

    use base64::{engine::general_purpose, Engine as _};
    let bytes = general_purpose::STANDARD
        .decode(&bytes_base64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    log::info!(
        "Decoded bytes (file={}, size={})",
        redact_path_str(&filename),
        summarize_bytes(bytes.len())
    );

    let result = image_handler::process_dropped_file_from_bytes(bytes, &filename, &settings)?;

    log::info!(
        "Image saved from bytes (link_chars={})",
        result.markdown.chars().count()
    );
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
        log::info!("Autostart enabled");
    } else {
        manager
            .disable()
            .map_err(|e| format!("Failed to disable autostart: {}", e))?;
        log::info!("Autostart disabled");
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
    state.edge_detector.set_window_open(false).await;

    if let Some(window) = app.get_webview_window("capture") {
        let _ = window.hide();
    }

    log::info!("Window hidden");
    Ok(())
}

#[tauri::command]
async fn show_capture(app: AppHandle, state: tauri::State<'_, AppState>) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("capture") {
        let settings = state.settings.read().await;
        position_window_logical(&window, &settings)?;
        let _ = window.show();
        let _ = window.set_focus();
    }
    state.edge_detector.set_window_open(true).await;
    log::info!("Window shown");
    Ok(())
}

#[tauri::command]
async fn get_window_info(state: tauri::State<'_, AppState>) -> Result<serde_json::Value, String> {
    let settings = state.settings.read().await;
    Ok(serde_json::json!({
        "width": settings.window_width,
        "height": settings.window_height,
        "borderRadius": settings.border_radius,
        "backgroundColor": settings.background_color,
        "fontFamily": settings.font_family,
        "fontSize": settings.font_size,
        "edgeSide": settings.edge_side,
    }))
}

#[tauri::command]
async fn open_settings(app: AppHandle) -> Result<(), String> {
    log::info!("Opening settings window");
    if let Some(window) = app.get_webview_window("settings") {
        log::info!("Settings window exists, showing it");
        let _ = window.show();
        let _ = window.center();
        let _ = window.set_focus();
        let _ = window.unminimize();
    } else {
        log::warn!("Settings window does not exist, creating it");
        use tauri::WebviewUrl;
        use tauri::WebviewWindowBuilder;

        let settings_window =
            WebviewWindowBuilder::new(&app, "settings", WebviewUrl::App("settings.html".into()))
                .title("Collector - Einstellungen")
                .inner_size(520.0, 680.0)
                .resizable(true)
                .center()
                .build()
                .map_err(|e| format!("Failed to create settings window: {}", e))?;

        let _ = settings_window.show();
        let _ = settings_window.set_focus();
        log::info!("Settings window created and shown");
    }
    Ok(())
}

#[tauri::command]
async fn close_settings(app: AppHandle) -> Result<(), String> {
    log::info!("Closing settings window");
    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.hide();
        log::info!("Settings window hidden");
    }
    Ok(())
}

#[cfg(target_os = "macos")]
fn configure_macos_window(window: &tauri::WebviewWindow, corner_radius: f64) {
    let ns_window_ptr = match window.ns_window() {
        Ok(ptr) => ptr,
        Err(e) => {
            log::warn!("Failed to get ns_window: {} - window might not be fully initialized yet", e);
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

        log::info!("macOS window configured with corner radius: {}", corner_radius);
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_macos_window(_window: &tauri::WebviewWindow, _corner_radius: f64) {
}

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

fn create_tray_menu(app: &AppHandle) -> Menu<tauri::Wry> {
    let quick_capture = MenuItem::with_id(
        app,
        "quick_capture",
        "Quick Capture (⌘⇧N)",
        true,
        None::<String>,
    )
    .unwrap();
    let settings =
        MenuItem::with_id(app, "settings", "Einstellungen...", true, None::<String>).unwrap();
    let quit = MenuItem::with_id(app, "quit", "Beenden", true, None::<String>).unwrap();

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
        let _ = position_window_logical(&window, settings);
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit("show_capture", ());
    }
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Quick Capture...");

    let settings = Settings::load().unwrap_or_else(|e| {
        log::error!("Failed to load settings: {}, using defaults", e);
        Settings::default()
    });

    log::info!(
        "Settings loaded: vault={}, edge={}",
        settings.vault_name,
        settings.edge_side
    );

    let edge_detector = Arc::new(EdgeDetector::new(settings.clone()));
    let shortcut_manager = Arc::new(ShortcutManager::new());
    let capture_text_shortcut_manager = Arc::new(ShortcutManager::new());
    let save_as_note_shortcut_manager = Arc::new(ShortcutManager::new());

    let app_state = AppState {
        settings: Arc::new(RwLock::new(settings.clone())),
        edge_detector: edge_detector.clone(),
        shortcut_manager: shortcut_manager.clone(),
        capture_text_shortcut_manager: capture_text_shortcut_manager.clone(),
        save_as_note_shortcut_manager: save_as_note_shortcut_manager.clone(),
    };

    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_autostart::init(tauri_plugin_autostart::MacosLauncher::LaunchAgent, Some(vec!["--hidden"])))
        .setup(move |app| {
            let app_handle = app.handle().clone();

            let menu = create_tray_menu(&app_handle);
            let settings_for_tray = settings.clone();
            let _tray = TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quick_capture" => {
                        show_capture_window(app, &settings_for_tray);
                    },
                    "settings" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = open_settings(app_clone).await;
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
                if let Err(e) = shortcut_mgr.register(&app_handle_shortcut, &settings_for_shortcut).await {
                    log::error!("Failed to register shortcut: {}", e);
                }
            });

            let capture_text_mgr = capture_text_shortcut_manager.clone();
            let app_handle_capture_text = app_handle.clone();
            let settings_for_capture_text = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = capture_text_mgr.register_capture_text(&app_handle_capture_text, &settings_for_capture_text).await {
                    log::error!("Failed to register capture_text shortcut: {}", e);
                }
            });

            let save_as_note_mgr = save_as_note_shortcut_manager.clone();
            let app_handle_save_as_note = app_handle.clone();
            let settings_for_save_as_note = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = save_as_note_mgr.register_save_as_note(&app_handle_save_as_note, &settings_for_save_as_note).await {
                    log::error!("Failed to register save_as_note shortcut: {}", e);
                }
            });

            if let Some(window) = app.get_webview_window("capture") {
                let _ = position_window_logical(&window, &settings);

                let window_clone = window.clone();
                let border_radius = settings.border_radius;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                    configure_macos_window(&window_clone, border_radius as f64);
                });

                log::info!("Capture window initialized from config (transparent: true, dragDropEnabled: false)");
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
            save_image,
            save_image_from_bytes,
            toggle_edge_detection,
            set_autostart,
            is_autostart_enabled,
            hide_capture,
            show_capture,
            get_window_info,
            open_settings,
            close_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
