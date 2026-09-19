use std::sync::atomic::Ordering;
use std::sync::Arc;

use tauri::{AppHandle, LogicalPosition, LogicalSize, Manager};

use crate::app_state::AppState;
use crate::settings::Settings;

pub(crate) fn position_window_logical(
    window: &tauri::WebviewWindow,
    settings: &Settings,
) -> Result<(), String> {
    let app = window.app_handle();
    let monitor = crate::edge_detect::get_screen_bounds(app);

    if settings.remember_window_geometry {
        if let (Some(x), Some(y), Some(width), Some(height)) = (
            settings.remembered_window_x,
            settings.remembered_window_y,
            settings.remembered_window_width,
            settings.remembered_window_height,
        ) {
            let all_monitors = crate::edge_detect::get_all_monitor_bounds_from_app(app);
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
        _ => monitor.x as f64 + monitor.width as f64 - width,
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

pub(crate) fn handle_capture_window_geometry_event(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    debounce: Arc<std::sync::atomic::AtomicU64>,
) {
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

pub(crate) fn position_reader_window_logical(
    window: &tauri::WebviewWindow,
    settings: &Settings,
) -> Result<(), String> {
    let app = window.app_handle();
    let monitor = crate::edge_detect::get_screen_bounds(app);

    if settings.remember_reader_geometry {
        if let (Some(x), Some(y), Some(width), Some(height)) = (
            settings.remembered_reader_x,
            settings.remembered_reader_y,
            settings.remembered_reader_width,
            settings.remembered_reader_height,
        ) {
            let all_monitors = crate::edge_detect::get_all_monitor_bounds_from_app(app);
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
                "Remembered reader geometry is off-screen for all connected monitors; falling back to edge position"
            );
        }
    }

    let width = settings.reader_width as f64;
    let height = settings.reader_height as f64;
    let y = monitor.y as f64 + (monitor.height as f64 - height) / 2.0;

    window
        .set_size(LogicalSize::new(width, height))
        .map_err(|e| e.to_string())?;

    let x = match settings.reader_edge_side.as_str() {
        "left" => monitor.x as f64,
        _ => monitor.x as f64 + monitor.width as f64 - width,
    };

    window
        .set_position(LogicalPosition::new(x, y))
        .map_err(|e| e.to_string())?;

    Ok(())
}

pub(crate) fn handle_reader_window_geometry_event(
    app: &AppHandle,
    window: &tauri::WebviewWindow,
    debounce: Arc<std::sync::atomic::AtomicU64>,
) {
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
        let remember = state.settings.read().await.remember_reader_geometry;
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
        let unchanged = settings.remembered_reader_x == Some(logical_x)
            && settings.remembered_reader_y == Some(logical_y)
            && settings.remembered_reader_width == Some(logical_width)
            && settings.remembered_reader_height == Some(logical_height);
        if unchanged {
            return;
        }

        settings.remembered_reader_x = Some(logical_x);
        settings.remembered_reader_y = Some(logical_y);
        settings.remembered_reader_width = Some(logical_width);
        settings.remembered_reader_height = Some(logical_height);

        if let Err(error) = settings.save() {
            log::warn!("Failed to persist remembered reader geometry: {}", error);
        }
    });
}
