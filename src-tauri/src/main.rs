#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unexpected_cfgs)]

mod app_state;
mod capture;
mod capture_draft;
mod capture_mode;
mod commands;
mod edge_detect;
mod fs_util;
mod image_handler;
mod log_safety;
mod markdown_sections;
mod ocr;
#[cfg(all(debug_assertions, target_os = "macos"))]
mod resize_diagnostics;
#[cfg(target_os = "macos")]
mod resize_geometry;
mod selected_text;
mod settings;
mod shortcuts;
mod tray;
mod updater;
mod vault_index;
mod vault_paths;
mod window_geometry;
#[cfg(target_os = "macos")]
mod window_resize;
mod window_style;

use std::sync::Arc;
use tauri::{AppHandle, Listener, Manager, WindowEvent};
use tokio::sync::RwLock;

use crate::app_state::AppState;
use crate::edge_detect::EdgeDetector;
use crate::settings::Settings;
use crate::shortcuts::ShortcutManager;
use crate::window_geometry::{
    handle_capture_window_geometry_event, handle_reader_window_geometry_event,
    position_reader_window_logical, position_window_logical,
};
use crate::window_style::configure_macos_window;

const WINDOW_INIT_DELAY_MS: u64 = 100;

fn warn_if_failed<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    if let Err(error) = result {
        log::warn!("{}: {}", context, error);
    }
}

async fn show_capture_internal(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(window) = app.get_webview_window("capture") {
        let settings = state.settings.read().await;
        position_window_logical(&window, &settings)?;
        configure_macos_window(
            &window,
            settings.border_radius as f64,
            settings.window_blur as i64,
            &settings.edge_side,
            settings.round_all_corners,
        );
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
    }
    state.edge_detector.set_capture_open(true).await;
    Ok(())
}

pub(crate) async fn show_reader_internal(app: &AppHandle) -> Result<(), String> {
    let state = app.state::<AppState>();
    if let Some(window) = app.get_webview_window("reader") {
        let settings = state.settings.read().await;
        position_reader_window_logical(&window, &settings)?;
        configure_macos_window(
            &window,
            settings.border_radius as f64,
            settings.window_blur as i64,
            &settings.reader_edge_side,
            settings.round_all_corners,
        );
        warn_if_failed(window.show(), "Failed to show reader window");
        warn_if_failed(window.set_focus(), "Failed to focus reader window");
    }
    state.edge_detector.set_reader_open(true).await;
    Ok(())
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
    let capture_mode_controller = Arc::new(capture_mode::CaptureModeController::new());
    let capture_mode_shortcut_manager = Arc::new(ShortcutManager::new());
    let ocr_shortcut_manager = Arc::new(ShortcutManager::new());
    let window_geometry_debounce = Arc::new(std::sync::atomic::AtomicU64::new(0));
    let reader_window_geometry_debounce = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let app_state = AppState {
        settings: Arc::new(RwLock::new(settings.clone())),
        edge_detector: edge_detector.clone(),
        shortcut_manager: shortcut_manager.clone(),
        reader_shortcut_manager: reader_shortcut_manager.clone(),
        capture_text_shortcut_manager: capture_text_shortcut_manager.clone(),
        save_as_note_shortcut_manager: save_as_note_shortcut_manager.clone(),
        capture_mode_controller: capture_mode_controller.clone(),
        capture_mode_shortcut_manager: capture_mode_shortcut_manager.clone(),
        ocr_shortcut_manager: ocr_shortcut_manager.clone(),
        vault_index: Arc::new(RwLock::new(None)),
    };

    let window_geometry_debounce_for_setup = window_geometry_debounce.clone();
    let reader_window_geometry_debounce_for_setup = reader_window_geometry_debounce.clone();

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

            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Accessory);

            tray::setup_tray(app, settings.clone())?;
            tray::spawn_update_watcher(app_handle.clone());

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

            let capture_mode_mgr = capture_mode_shortcut_manager.clone();
            let capture_mode_controller_for_shortcut = capture_mode_controller.clone();
            let app_handle_capture_mode = app_handle.clone();
            let settings_for_capture_mode = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = capture_mode_mgr
                    .register_capture_mode(
                        &app_handle_capture_mode,
                        &settings_for_capture_mode,
                        capture_mode_controller_for_shortcut,
                    )
                    .await
                {
                    log::error!("Failed to register capture_mode shortcut: {}", e);
                }
            });

            let ocr_mgr = ocr_shortcut_manager.clone();
            let app_handle_ocr = app_handle.clone();
            let settings_for_ocr = settings.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = ocr_mgr
                    .register_ocr(&app_handle_ocr, &settings_for_ocr)
                    .await
                {
                    log::error!("Failed to register OCR shortcut: {}", e);
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
                #[cfg(target_os = "macos")]
                window_resize::install(&window)?;
                warn_if_failed(
                    position_window_logical(&window, &settings),
                    "Failed to position capture window",
                );

                let window_clone = window.clone();
                let border_radius = settings.border_radius;
                let window_blur = settings.window_blur;
                let edge_side = settings.edge_side.clone();
                let round_all_corners = settings.round_all_corners;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_INIT_DELAY_MS))
                        .await;
                    configure_macos_window(
                        &window_clone,
                        border_radius as f64,
                        window_blur as i64,
                        &edge_side,
                        round_all_corners,
                    );
                });

                let geometry_app_handle = app_handle.clone();
                let geometry_window = window.clone();
                let geometry_debounce = window_geometry_debounce_for_setup.clone();
                window.on_window_event(move |event| {
                    if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
                        handle_capture_window_geometry_event(
                            &geometry_app_handle,
                            &geometry_window,
                            geometry_debounce.clone(),
                        );
                    }
                });
            }

            if let Some(window) = app.get_webview_window("reader") {
                #[cfg(target_os = "macos")]
                window_resize::install(&window)?;
                warn_if_failed(
                    position_reader_window_logical(&window, &settings),
                    "Failed to position reader window",
                );

                let window_clone = window.clone();
                let border_radius = settings.border_radius;
                let window_blur = settings.window_blur;
                let edge_side = settings.reader_edge_side.clone();
                let round_all_corners = settings.round_all_corners;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_INIT_DELAY_MS))
                        .await;
                    configure_macos_window(
                        &window_clone,
                        border_radius as f64,
                        window_blur as i64,
                        &edge_side,
                        round_all_corners,
                    );
                });

                let reader_geometry_app_handle = app_handle.clone();
                let reader_geometry_window = window.clone();
                let reader_geometry_debounce = reader_window_geometry_debounce_for_setup.clone();
                window.on_window_event(move |event| {
                    if matches!(event, WindowEvent::Moved(_) | WindowEvent::Resized(_)) {
                        handle_reader_window_geometry_event(
                            &reader_geometry_app_handle,
                            &reader_geometry_window,
                            reader_geometry_debounce.clone(),
                        );
                    }
                });
            }

            log::info!("Quick Capture setup complete");
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            #[cfg(all(debug_assertions, target_os = "macos"))]
            resize_diagnostics::resize_diagnostics,
            commands::settings::load_settings,
            commands::window::save_settings,
            commands::capture::save_as_note,
            commands::capture::append_to_daily_note,
            commands::capture::append_to_note,
            commands::capture::save_capture_draft,
            commands::capture::load_capture_draft,
            commands::capture::clear_capture_draft,
            commands::capture_mode::get_capture_mode_state,
            commands::capture_mode::set_capture_mode_enabled,
            commands::capture_mode::toggle_capture_mode,
            commands::vault::read_note_file,
            commands::vault::write_note_file,
            commands::system::open_external_url,
            commands::system::get_running_apps,
            commands::system::list_applications,
            commands::vault::load_image_data_url,
            commands::vault::load_images_batch,
            commands::vault::list_vault_notes,
            commands::capture::get_daily_note_path,
            commands::vault::reindex_vault,
            commands::images::save_image,
            commands::images::save_image_from_bytes,
            commands::ocr::ocr_clipboard_image,
            commands::ocr::ocr_image_file,
            commands::ocr::ocr_image_bytes,
            commands::ocr::get_modifier_flags,
            commands::settings::toggle_edge_detection,
            commands::autostart::set_autostart,
            commands::autostart::is_autostart_enabled,
            commands::window::hide_capture,
            commands::window::show_capture,
            commands::window::show_reader,
            commands::window::hide_reader,
            commands::settings::get_window_info,
            commands::window::open_settings,
            commands::window::close_settings,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
