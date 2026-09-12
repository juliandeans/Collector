use tauri::{AppHandle, Emitter, Manager};

use crate::app_state::AppState;
use crate::settings::Settings;
use crate::{
    configure_macos_window, position_reader_window_logical, position_window_logical,
    show_reader_internal, warn_if_failed,
};

const SETTINGS_WINDOW_WIDTH: f64 = 980.0;
const SETTINGS_WINDOW_HEIGHT: f64 = 720.0;

#[tauri::command]
pub(crate) async fn save_settings(
    mut new_settings: Settings,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    new_settings.normalize_pinned_note_paths();
    new_settings.normalize_screenshot_path();

    let old_vault_path;
    {
        let mut current = state.settings.write().await;

        new_settings.remembered_window_x = current.remembered_window_x;
        new_settings.remembered_window_y = current.remembered_window_y;
        new_settings.remembered_window_width = current.remembered_window_width;
        new_settings.remembered_window_height = current.remembered_window_height;
        new_settings.remembered_reader_x = current.remembered_reader_x;
        new_settings.remembered_reader_y = current.remembered_reader_y;
        new_settings.remembered_reader_width = current.remembered_reader_width;
        new_settings.remembered_reader_height = current.remembered_reader_height;

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

        old_vault_path = current.vault_path.clone();
        *current = new_settings.clone();
    }

    if old_vault_path != new_settings.vault_path {
        *state.vault_index.write().await = None;
        log::info!("Vault index invalidated: vault path changed");
    }

    state
        .edge_detector
        .update_settings(new_settings.clone())
        .await;

    if let Some(window) = app.get_webview_window("capture") {
        if let Err(e) = position_window_logical(&window, &new_settings) {
            log::warn!("Failed to update window position: {}", e);
        }

        configure_macos_window(
            &window,
            new_settings.border_radius as f64,
            new_settings.window_blur as i64,
            &new_settings.edge_side,
            new_settings.round_all_corners,
        );
    }

    if let Some(reader_window) = app.get_webview_window("reader") {
        if let Err(e) = position_reader_window_logical(&reader_window, &new_settings) {
            log::warn!("Failed to update reader window position: {}", e);
        }

        configure_macos_window(
            &reader_window,
            new_settings.border_radius as f64,
            new_settings.window_blur as i64,
            &new_settings.reader_edge_side,
            new_settings.round_all_corners,
        );
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

    if let Err(e) = state
        .capture_mode_shortcut_manager
        .register_capture_mode(&app, &new_settings, state.capture_mode_controller.clone())
        .await
    {
        log::warn!("Failed to update capture_mode shortcut (non-fatal): {}", e);
    }

    if let Err(e) = state
        .ocr_shortcut_manager
        .register_ocr(&app, &new_settings)
        .await
    {
        log::warn!("Failed to update OCR shortcut (non-fatal): {}", e);
    }

    Ok(())
}

#[tauri::command]
pub(crate) async fn hide_capture(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.edge_detector.set_capture_open(false).await;

    if let Some(window) = app.get_webview_window("capture") {
        warn_if_failed(window.hide(), "Failed to hide capture window");
    }

    Ok(())
}

#[tauri::command]
pub(crate) async fn show_capture(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("capture") {
        let settings = state.settings.read().await;
        position_window_logical(&window, &settings)?;
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
    }
    state.edge_detector.set_capture_open(true).await;
    Ok(())
}

#[tauri::command]
pub(crate) async fn show_reader(
    app: AppHandle,
    _state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    show_reader_internal(&app).await
}

#[tauri::command]
pub(crate) async fn hide_reader(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("reader") {
        warn_if_failed(window.hide(), "Failed to hide reader window");
    }
    state.edge_detector.set_reader_open(false).await;
    Ok(())
}

#[tauri::command]
pub(crate) async fn open_settings(app: AppHandle) -> Result<(), String> {
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
pub(crate) async fn close_settings(app: AppHandle) -> Result<(), String> {
    if let Some(window) = app.get_webview_window("settings") {
        warn_if_failed(window.hide(), "Failed to hide settings window");
    }
    Ok(())
}
