use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager,
};

use crate::app_state::AppState;
use crate::settings::Settings;
use crate::window_geometry::position_window_logical;
use crate::window_style::configure_macos_window;

fn warn_if_failed<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    if let Err(error) = result {
        log::warn!("{}: {}", context, error);
    }
}

pub(crate) fn setup_tray(app: &tauri::App, settings: Settings) -> tauri::Result<()> {
    let menu = create_tray_menu(app.handle());
    let settings_for_tray = settings;
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
                    if let Err(error) = crate::commands::window::open_settings(app_clone).await {
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
    Ok(())
}

pub(crate) fn spawn_update_watcher(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Some(latest) = crate::updater::check_for_update().await {
            let quick_capture = MenuItem::with_id(
                &app_handle,
                "quick_capture",
                "Quick Capture (⌘⇧N)",
                true,
                None::<String>,
            )
            .unwrap();
            let settings = MenuItem::with_id(
                &app_handle,
                "settings",
                "Settings...",
                true,
                None::<String>,
            )
            .unwrap();
            let open_update = MenuItem::with_id(
                &app_handle,
                "open_update",
                format!("Update available: v{}", latest),
                true,
                None::<String>,
            )
            .unwrap();
            let quit =
                MenuItem::with_id(&app_handle, "quit", "Quit", true, None::<String>).unwrap();
            let new_menu = Menu::with_items(
                &app_handle,
                &[
                    &quick_capture,
                    &settings,
                    &PredefinedMenuItem::separator(&app_handle).unwrap(),
                    &open_update,
                    &PredefinedMenuItem::separator(&app_handle).unwrap(),
                    &quit,
                ],
            )
            .unwrap();

            if let Some(tray) = app_handle.tray_by_id("main") {
                let _ = tray.set_menu(Some(new_menu));
            }
        }
    });
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
            &settings.edge_side,
            settings.round_all_corners,
        );
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
        warn_if_failed(app.emit("reset_capture", ()), "Failed to emit reset_capture");
        warn_if_failed(app.emit("show_capture", ()), "Failed to emit show_capture");
    }
}
