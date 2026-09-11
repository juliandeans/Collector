#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]
#![allow(unexpected_cfgs)]

mod app_state;
mod capture;
mod capture_draft;
mod commands;
mod edge_detect;
mod fs_util;
mod image_handler;
mod log_safety;
mod markdown_sections;
mod selected_text;
mod settings;
mod shortcuts;
mod updater;
mod vault_index;
mod vault_paths;

use std::sync::Arc;
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Listener, LogicalPosition, LogicalSize, Manager, WindowEvent,
};
use tokio::sync::RwLock;

#[cfg(target_os = "macos")]
use cocoa::base::{id, BOOL, YES};
#[cfg(target_os = "macos")]
use cocoa::foundation::NSInteger;
#[cfg(target_os = "macos")]
use objc::{class, msg_send, sel, sel_impl};
#[cfg(target_os = "macos")]
use tauri::window::{Effect, EffectState, EffectsBuilder};

// Private WindowServer API, used here only to force-clear a window's native
// background blur. NSVisualEffectView's BehindWindow blending is itself
// implemented by telling the WindowServer to blur the window (confirmed via
// diagnostic logging: removing the NSVisualEffectView subview always
// succeeds at the AppKit object-graph level, but the WindowServer keeps
// blurring until this is called with radius 0 — removeFromSuperview alone
// doesn't synchronously undo the WindowServer-side registration).
#[cfg(target_os = "macos")]
#[link(name = "CoreGraphics", kind = "framework")]
extern "C" {
    fn CGSMainConnectionID() -> id;
    fn CGSSetWindowBackgroundBlurRadius(
        connection_id: id,
        window_id: NSInteger,
        radius: i64,
    ) -> i32;
}

use crate::app_state::AppState;
use crate::edge_detect::EdgeDetector;
use crate::settings::Settings;
use crate::shortcuts::ShortcutManager;

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
        );
        warn_if_failed(window.show(), "Failed to show reader window");
        warn_if_failed(window.set_focus(), "Failed to focus reader window");
    }
    state.edge_detector.set_reader_open(true).await;
    Ok(())
}

// Buckets the 0-150 "Background Blur" setting into one of AppKit's fixed
// NSVisualEffectView materials. The stops (0/50/100/150) match the Settings
// slider's step size and are labeled Max/Light/Medium/Strong there — "Max"
// meaning maximum background visibility (no native blur, just the app's own
// CSS tint), not maximum blur strength.
#[cfg(target_os = "macos")]
fn blur_effect_for_level(level: i64) -> Option<Effect> {
    match level {
        i64::MIN..=0 => None,
        1..=50 => Some(Effect::HudWindow),
        51..=100 => Some(Effect::Popover),
        _ => Some(Effect::UnderWindowBackground),
    }
}

#[cfg(target_os = "macos")]
fn configure_macos_window(
    window: &tauri::WebviewWindow,
    corner_radius: f64,
    background_blur_radius: i64,
) {
    // AppKit calls must happen on the main thread. configure_macos_window()
    // is invoked from the async `save_settings` Tauri command, which runs on
    // a tokio worker thread, not the main thread — Tauri's own
    // window.set_effects()/with_webview() calls already dispatch themselves
    // to the main thread internally, but our raw msg_send calls did not, so
    // NSView-hierarchy mutations (removing the old blur view) landed on the
    // object graph but never got a proper main-run-loop CATransaction commit
    // to actually repaint. That's what left the previous blur visible even
    // though the tagged view was confirmably gone. run_on_main_thread here
    // matches what Tauri already does for the effects call below, and
    // preserves ordering since both are queued on the same main-thread
    // dispatcher.
    const BLUR_VIEW_TAG: NSInteger = 91376254; // window-vibrancy's NS_VIEW_TAG_BLUR_VIEW
    let window_for_main_thread = window.clone();
    if let Err(e) = window.run_on_main_thread(move || {
        let ns_window_ptr = match window_for_main_thread.ns_window() {
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

            let _: () = msg_send![ns_window, setMovableByWindowBackground: YES];

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

            let _: () = msg_send![ns_window, setOpaque: BOOL::from(false)];

            let clear_color: id = msg_send![class!(NSColor), clearColor];
            let _: () = msg_send![ns_window, setBackgroundColor: clear_color];

            let _: () = msg_send![ns_window, setHasShadow: YES];
            let _: () = msg_send![ns_window, invalidateShadow];

            // Tauri's set_effects() always *adds* a new NSVisualEffectView
            // subview and never removes a previous one (confirmed in
            // tauri's and window-vibrancy's source — there is no "replace"
            // path on macOS). configure_macos_window() runs every time a
            // window is (re)shown and on every settings save, so without
            // this the blur view would stack indefinitely. Remove any
            // existing tagged view first so at most one ever exists, and
            // explicitly zero the WindowServer's blur registration too,
            // since NSVisualEffectView's BehindWindow blending is itself
            // implemented through that same private call.
            let existing: id = msg_send![content_view, viewWithTag: BLUR_VIEW_TAG];
            if !existing.is_null() {
                let _: () = msg_send![existing, removeFromSuperview];
            }
            let window_number: NSInteger = msg_send![ns_window, windowNumber];
            CGSSetWindowBackgroundBlurRadius(CGSMainConnectionID(), window_number, 0);
        }
    }) {
        log::warn!("Failed to run window configuration on main thread: {}", e);
    }

    // Native window-background blur via AppKit's NSVisualEffectView,
    // WindowServer-composited behind the window (the same mechanism the
    // menu bar and Dock use) — unlike CSS backdrop-filter, which is bounded
    // by the WKWebView's own render surface and produces a hard unblurred
    // strip when the window sits flush against the physical screen edge.
    //
    // AppKit exposes no continuous blur-strength dial, only named materials,
    // so the 0-150 setting value is bucketed into a handful of fixed looks
    // instead of mapping 1:1 to a blur radius.
    let effects = blur_effect_for_level(background_blur_radius).map(|effect| {
        EffectsBuilder::new()
            .effect(effect)
            .state(EffectState::Active)
            .radius(corner_radius)
            .build()
    });
    if let Err(e) = window.set_effects(effects) {
        log::warn!("Failed to set window effects: {}", e);
    }

    // Clip the WKWebView's own layer directly instead of reaching into
    // WebKit-internal sublayers, which the WebContent/GPU process can
    // replace out from under us (e.g. on compositing changes).
    if let Err(e) = window.with_webview(move |platform_webview| unsafe {
        let webview: id = platform_webview.inner() as id;
        if webview.is_null() {
            return;
        }

        let _: () = msg_send![webview, setWantsLayer: YES];

        let webview_layer: id = msg_send![webview, layer];
        if !webview_layer.is_null() {
            let _: () = msg_send![webview_layer, setCornerRadius: corner_radius];
            let _: () = msg_send![webview_layer, setMasksToBounds: YES];
        }
    }) {
        log::warn!("Failed to clip WKWebView layer: {}", e);
    }
}

#[cfg(not(target_os = "macos"))]
fn configure_macos_window(
    _window: &tauri::WebviewWindow,
    _corner_radius: f64,
    _background_blur_radius: i64,
) {
}

fn position_window_logical(
    window: &tauri::WebviewWindow,
    settings: &Settings,
) -> Result<(), String> {
    let app = window.app_handle();
    let monitor = edge_detect::get_screen_bounds(&app);

    if settings.remember_window_geometry {
        if let (Some(x), Some(y), Some(width), Some(height)) = (
            settings.remembered_window_x,
            settings.remembered_window_y,
            settings.remembered_window_width,
            settings.remembered_window_height,
        ) {
            let all_monitors = edge_detect::get_all_monitor_bounds_from_app(&app);
            let overlaps_any_monitor = all_monitors.iter().any(|m| {
                (x as f64) < (m.x as f64 + m.width as f64)
                    && (x as f64 + width as f64) > m.x as f64
                    && (y as f64) < (m.y as f64 + m.height as f64)
                    && (y as f64 + height as f64) > m.y as f64
            });

            if overlaps_any_monitor {
                window
                    .set_size(LogicalSize {
                        width: width as f64,
                        height: height as f64,
                    })
                    .map_err(|e| e.to_string())?;

                window
                    .set_position(LogicalPosition {
                        x: x as f64,
                        y: y as f64,
                    })
                    .map_err(|e| e.to_string())?;

                return Ok(());
            }

            log::warn!(
                "Remembered window geometry is off-screen for all connected monitors; falling back to edge position"
            );
        }
    }

    let width = settings.window_width as f64;
    let height = settings.window_height as f64;

    let y = monitor.y as f64 + (monitor.height as f64 - height) / 2.0;

    let x = match settings.edge_side.as_str() {
        "left" => monitor.x as f64,
        "right" | _ => monitor.x as f64 + monitor.width as f64 - width,
    };

    window
        .set_size(LogicalSize { width, height })
        .map_err(|e| e.to_string())?;

    window
        .set_position(LogicalPosition { x, y })
        .map_err(|e| e.to_string())?;

    Ok(())
}

const WINDOW_GEOMETRY_DEBOUNCE_MS: u64 = 400;

fn handle_capture_window_geometry_event(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    debounce: Arc<std::sync::atomic::AtomicU64>,
) {
    use std::sync::atomic::Ordering;

    let my_generation = debounce.fetch_add(1, Ordering::SeqCst) + 1;

    let app = app.clone();
    let window = window.clone();

    tauri::async_runtime::spawn(async move {
        tokio::time::sleep(tokio::time::Duration::from_millis(
            WINDOW_GEOMETRY_DEBOUNCE_MS,
        ))
        .await;

        if debounce.load(Ordering::SeqCst) != my_generation {
            return;
        }

        let state = app.state::<AppState>();
        let remember = state.settings.read().await.remember_window_geometry;
        if !remember {
            return;
        }

        let Ok(physical_position) = window.outer_position() else {
            return;
        };
        let Ok(physical_size) = window.outer_size() else {
            return;
        };
        let scale = window.scale_factor().unwrap_or(1.0);

        let logical_position = physical_position.to_logical::<f64>(scale);
        let logical_size = physical_size.to_logical::<f64>(scale);

        let logical_x = logical_position.x.round() as i32;
        let logical_y = logical_position.y.round() as i32;
        let logical_width = logical_size.width.round() as u32;
        let logical_height = logical_size.height.round() as u32;

        let mut settings = state.settings.write().await;
        let unchanged = settings.remembered_window_x == Some(logical_x)
            && settings.remembered_window_y == Some(logical_y)
            && settings.remembered_window_width == Some(logical_width)
            && settings.remembered_window_height == Some(logical_height);
        if unchanged {
            return;
        }

        settings.remembered_window_x = Some(logical_x);
        settings.remembered_window_y = Some(logical_y);
        settings.remembered_window_width = Some(logical_width);
        settings.remembered_window_height = Some(logical_height);

        if let Err(error) = settings.save() {
            log::warn!("Failed to persist remembered window geometry: {}", error);
        }
    });
}

fn position_reader_window_logical(
    window: &tauri::WebviewWindow,
    settings: &Settings,
) -> Result<(), String> {
    let app = window.app_handle();
    let monitor = edge_detect::get_screen_bounds(&app);
    let width = settings.reader_width as f64;
    let height = settings.reader_height as f64;
    let y = monitor.y as f64 + (monitor.height as f64 - height) / 2.0;

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;

    let x = match settings.reader_edge_side.as_str() {
        "left" => monitor.x as f64,
        "right" | _ => monitor.x as f64 + monitor.width as f64 - width,
    };

    window
        .set_position(LogicalPosition::new(x, y))
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
        configure_macos_window(
            &window,
            settings.border_radius as f64,
            settings.window_blur as i64,
        );
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
        warn_if_failed(app.emit("reset_capture", ()), "Failed to emit reset_capture");
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
    let window_geometry_debounce = Arc::new(std::sync::atomic::AtomicU64::new(0));

    let app_state = AppState {
        settings: Arc::new(RwLock::new(settings.clone())),
        edge_detector: edge_detector.clone(),
        shortcut_manager: shortcut_manager.clone(),
        reader_shortcut_manager: reader_shortcut_manager.clone(),
        capture_text_shortcut_manager: capture_text_shortcut_manager.clone(),
        save_as_note_shortcut_manager: save_as_note_shortcut_manager.clone(),
        vault_index: Arc::new(RwLock::new(None)),
    };

    let window_geometry_debounce_for_setup = window_geometry_debounce.clone();

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

            let menu = create_tray_menu(&app_handle);
            let settings_for_tray = settings.clone();
            let _tray = TrayIconBuilder::with_id("main")
                .icon(
                    tauri::image::Image::from_bytes(include_bytes!("../icons/tray-icon.png"))
                        .unwrap(),
                )
                .menu(&menu)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "quick_capture" => {
                        show_capture_window(app, &settings_for_tray);
                    }
                    "settings" => {
                        let app_clone = app.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Err(error) = commands::window::open_settings(app_clone).await {
                                log::warn!("Failed to open settings window: {}", error);
                            }
                        });
                    }
                    "open_update" => {
                        open::that("https://github.com/juliandeans/Collector/releases/latest").ok();
                    }
                    "quit" => std::process::exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    handle_tray_event(_tray.app_handle(), event);
                })
                .build(app)?;

            let app_handle_for_update = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Some(latest) = updater::check_for_update().await {
                    let quick_capture = MenuItem::with_id(
                        &app_handle_for_update,
                        "quick_capture",
                        "Quick Capture (⌘⇧N)",
                        true,
                        None::<String>,
                    )
                    .unwrap();
                    let settings = MenuItem::with_id(
                        &app_handle_for_update,
                        "settings",
                        "Settings...",
                        true,
                        None::<String>,
                    )
                    .unwrap();
                    let open_update = MenuItem::with_id(
                        &app_handle_for_update,
                        "open_update",
                        format!("Update available: v{}", latest),
                        true,
                        None::<String>,
                    )
                    .unwrap();
                    let quit = MenuItem::with_id(
                        &app_handle_for_update,
                        "quit",
                        "Quit",
                        true,
                        None::<String>,
                    )
                    .unwrap();
                    let new_menu = Menu::with_items(
                        &app_handle_for_update,
                        &[
                            &quick_capture,
                            &settings,
                            &PredefinedMenuItem::separator(&app_handle_for_update).unwrap(),
                            &open_update,
                            &PredefinedMenuItem::separator(&app_handle_for_update).unwrap(),
                            &quit,
                        ],
                    )
                    .unwrap();

                    if let Some(tray) = app_handle_for_update.tray_by_id("main") {
                        let _ = tray.set_menu(Some(new_menu));
                    }
                }
            });

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
                let window_blur = settings.window_blur;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_INIT_DELAY_MS))
                        .await;
                    configure_macos_window(&window_clone, border_radius as f64, window_blur as i64);
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
                warn_if_failed(
                    position_reader_window_logical(&window, &settings),
                    "Failed to position reader window",
                );

                let window_clone = window.clone();
                let border_radius = settings.border_radius;
                let window_blur = settings.window_blur;
                tauri::async_runtime::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(WINDOW_INIT_DELAY_MS))
                        .await;
                    configure_macos_window(&window_clone, border_radius as f64, window_blur as i64);
                });
            }

            log::info!("Quick Capture setup complete");
            Ok(())
        })
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            commands::settings::load_settings,
            commands::window::save_settings,
            commands::capture::save_as_note,
            commands::capture::append_to_daily_note,
            commands::capture::append_to_note,
            commands::capture::save_capture_draft,
            commands::capture::load_capture_draft,
            commands::capture::clear_capture_draft,
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
