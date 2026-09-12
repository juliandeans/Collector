use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tokio::sync::Mutex;

use crate::capture_mode::CaptureModeController;
use crate::log_safety::summarize_text_len;
use crate::settings::Settings;

const CAPTURE_TEXT_INSERT_DELAY_MS: u64 = 200;

pub struct ShortcutManager {
    current_shortcuts: Arc<Mutex<Vec<String>>>,
}

fn warn_if_failed<T, E: std::fmt::Display>(result: Result<T, E>, context: &str) {
    if let Err(error) = result {
        log::warn!("{}: {}", context, error);
    }
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self {
            current_shortcuts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    async fn unregister_current_shortcuts(&self, app: &AppHandle, context: &str) -> Vec<String> {
        let current_shortcuts = std::mem::take(&mut *self.current_shortcuts.lock().await);

        for shortcut_str in &current_shortcuts {
            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                warn_if_failed(
                    app.global_shortcut().unregister(shortcut),
                    &format!("Failed to unregister {} shortcut", context),
                );
            }
        }

        current_shortcuts
    }

    pub async fn register(&self, app: &AppHandle, settings: &Settings) -> Result<(), String> {
        let open_shortcut_str = normalize_shortcut(&settings.global_shortcut);
        let close_shortcut_str = normalize_shortcut(&settings.global_close_shortcut);
        self.unregister_current_shortcuts(app, "global").await;

        let mut registered_shortcuts = Vec::new();
        let app_handle = app.clone();
        let closes_window = settings.global_shortcut_closes_window;

        if !open_shortcut_str.trim().is_empty() {
            let shortcut: Shortcut = open_shortcut_str.parse().map_err(|e| {
                let err_msg = format!("Invalid shortcut '{}': {:?}", open_shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

            let app_handle_open = app_handle.clone();
            app.global_shortcut()
                .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_handle2 = app_handle_open.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(window) = app_handle2.get_webview_window("capture") {
                                if window.is_visible().unwrap_or(false) {
                                    if closes_window {
                                        warn_if_failed(
                                            window.hide(),
                                            "Failed to hide capture window",
                                        );
                                        let state = app_handle2.state::<crate::app_state::AppState>();
                                        state.edge_detector.set_capture_open(false).await;
                                    } else {
                                        warn_if_failed(
                                            window.show(),
                                            "Failed to show capture window",
                                        );
                                        warn_if_failed(
                                            window.set_focus(),
                                            "Failed to focus capture window",
                                        );
                                    }
                                    return;
                                }
                            }

                            warn_if_failed(
                                app_handle2.emit("reset_capture", ()),
                                "Failed to emit reset_capture",
                            );
                            warn_if_failed(
                                app_handle2.emit("show_capture", ()),
                                "Failed to emit show_capture",
                            );
                            if let Some(window) = app_handle2.get_webview_window("capture") {
                                warn_if_failed(window.show(), "Failed to show capture window");
                                warn_if_failed(
                                    window.set_focus(),
                                    "Failed to focus capture window",
                                );
                            }
                        });
                    }
                })
                .map_err(|e| {
                    let err_msg = format!(
                        "Failed to register shortcut '{}': {:?}",
                        open_shortcut_str, e
                    );
                    log::error!("{}", err_msg);
                    err_msg
                })?;

            registered_shortcuts.push(open_shortcut_str.clone());
        }

        if !closes_window && !close_shortcut_str.trim().is_empty() {
            let shortcut: Shortcut = close_shortcut_str.parse().map_err(|e| {
                let err_msg = format!("Invalid shortcut '{}': {:?}", close_shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

            let app_handle_close = app_handle.clone();
            app.global_shortcut()
                .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_handle2 = app_handle_close.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(window) = app_handle2.get_webview_window("capture") {
                                if window.is_visible().unwrap_or(false) {
                                    warn_if_failed(window.hide(), "Failed to hide capture window");
                                    let state = app_handle2.state::<crate::app_state::AppState>();
                                    state.edge_detector.set_capture_open(false).await;
                                }
                            }
                        });
                    }
                })
                .map_err(|e| {
                    let err_msg = format!(
                        "Failed to register shortcut '{}': {:?}",
                        close_shortcut_str, e
                    );
                    log::error!("{}", err_msg);
                    err_msg
                })?;

            registered_shortcuts.push(close_shortcut_str.clone());
        }

        *self.current_shortcuts.lock().await = registered_shortcuts;
        Ok(())
    }

    pub async fn update(&self, app: &AppHandle, settings: &Settings) -> Result<(), String> {
        self.register(app, settings).await
    }

    pub async fn register_capture_text(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        let shortcut_str = normalize_shortcut(&settings.capture_text_shortcut);
        if shortcut_str.trim().is_empty() {
            self.unregister_current_shortcuts(app, "capture_text").await;
            return Ok(());
        }

        self.unregister_current_shortcuts(app, "capture_text").await;

        let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
            let err_msg = format!("Invalid capture_text shortcut '{}': {:?}", shortcut_str, e);
            log::error!("{}", err_msg);
            err_msg
        })?;

        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let app_handle2 = app_handle.clone();

                    tauri::async_runtime::spawn(async move {
                        let controller = app_handle2
                            .state::<crate::app_state::AppState>()
                            .capture_mode_controller
                            .clone();
                        controller.suppress_watcher(true);
                        let selected_result = tauri::async_runtime::spawn_blocking(
                            crate::selected_text::capture_selected_text,
                        )
                        .await;
                        controller.suppress_watcher(false);
                        let selected = selected_result.ok().flatten().unwrap_or_default();

                        log::info!("Captured text length={}", summarize_text_len(&selected));
                        warn_if_failed(
                            app_handle2.emit("show_capture", ()),
                            "Failed to emit show_capture",
                        );
                        if let Some(window) = app_handle2.get_webview_window("capture") {
                            warn_if_failed(window.show(), "Failed to show capture window");
                            warn_if_failed(window.set_focus(), "Failed to focus capture window");
                        }

                        if selected.trim().is_empty() {
                            log::warn!("No text was captured");
                            warn_if_failed(
                                app_handle2.emit(
                                    "capture_text_failed",
                                    "No text was captured. Select text in another app and try again.",
                                ),
                                "Failed to emit capture_text_failed",
                            );
                            return;
                        }

                        tokio::time::sleep(tokio::time::Duration::from_millis(
                            CAPTURE_TEXT_INSERT_DELAY_MS,
                        ))
                        .await;

                        warn_if_failed(
                            app_handle2.emit("insert_capture_text", selected),
                            "Failed to emit insert_capture_text",
                        );
                    });
                }
            })
            .map_err(|e| {
                let err_msg = format!(
                    "Failed to register capture_text shortcut '{}': {:?}",
                    shortcut_str, e
                );
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcuts.lock().await = vec![shortcut_str.clone()];
        Ok(())
    }

    pub async fn register_save_as_note(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        let shortcut_str = normalize_shortcut(&settings.save_as_note_shortcut);
        if shortcut_str.trim().is_empty() {
            self.unregister_current_shortcuts(app, "save_as_note").await;
            return Ok(());
        }

        self.unregister_current_shortcuts(app, "save_as_note").await;

        let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
            let err_msg = format!("Invalid save_as_note shortcut '{}': {:?}", shortcut_str, e);
            log::error!("{}", err_msg);
            err_msg
        })?;

        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    warn_if_failed(
                        app_handle.emit("save_as_note", ()),
                        "Failed to emit save_as_note",
                    );
                }
            })
            .map_err(|e| {
                let err_msg = format!(
                    "Failed to register save_as_note shortcut '{}': {:?}",
                    shortcut_str, e
                );
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcuts.lock().await = vec![shortcut_str.clone()];
        Ok(())
    }

    pub async fn register_capture_mode(
        &self,
        app: &AppHandle,
        settings: &Settings,
        controller: Arc<CaptureModeController>,
    ) -> Result<(), String> {
        let shortcut_str = normalize_shortcut(&settings.capture_mode_shortcut);
        if shortcut_str.trim().is_empty() {
            self.unregister_current_shortcuts(app, "capture_mode").await;
            return Ok(());
        }

        self.unregister_current_shortcuts(app, "capture_mode").await;

        let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
            let err_msg = format!(
                "Invalid capture_mode shortcut '{}': {:?}",
                shortcut_str, e
            );
            log::error!("{}", err_msg);
            err_msg
        })?;

        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let app_handle = app_handle.clone();
                    let controller = controller.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Err(error) = controller.toggle(&app_handle).await {
                            log::warn!("Failed to toggle capture mode: {}", error);
                        }
                    });
                }
            })
            .map_err(|e| {
                let err_msg = format!(
                    "Failed to register capture_mode shortcut '{}': {:?}",
                    shortcut_str, e
                );
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcuts.lock().await = vec![shortcut_str];
        Ok(())
    }

    pub async fn register_reader(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        let open_shortcut_str = normalize_shortcut(&settings.reader_shortcut);
        let close_shortcut_str = normalize_shortcut(&settings.reader_close_shortcut);
        self.unregister_current_shortcuts(app, "reader").await;

        let mut registered_shortcuts = Vec::new();
        let app_handle = app.clone();
        let closes_window = settings.reader_shortcut_closes_window;

        if !open_shortcut_str.trim().is_empty() {
            let shortcut: Shortcut = open_shortcut_str.parse().map_err(|e| {
                let err_msg = format!("Invalid reader shortcut '{}': {:?}", open_shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

            let app_handle_open = app_handle.clone();
            app.global_shortcut()
                .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_handle2 = app_handle_open.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(window) = app_handle2.get_webview_window("reader") {
                                if window.is_visible().unwrap_or(false) {
                                    if closes_window {
                                        warn_if_failed(
                                            window.hide(),
                                            "Failed to hide reader window",
                                        );
                                        let state = app_handle2.state::<crate::app_state::AppState>();
                                        state.edge_detector.set_reader_open(false).await;
                                    } else {
                                        warn_if_failed(
                                            window.show(),
                                            "Failed to show reader window",
                                        );
                                        warn_if_failed(
                                            window.set_focus(),
                                            "Failed to focus reader window",
                                        );
                                    }
                                    return;
                                }
                            }
                            warn_if_failed(
                                app_handle2.emit("show_reader", ()),
                                "Failed to emit show_reader",
                            );
                        });
                    }
                })
                .map_err(|e| {
                    let err_msg = format!(
                        "Failed to register reader shortcut '{}': {:?}",
                        open_shortcut_str, e
                    );
                    log::error!("{}", err_msg);
                    err_msg
                })?;

            registered_shortcuts.push(open_shortcut_str.clone());
        }

        if !closes_window && !close_shortcut_str.trim().is_empty() {
            let shortcut: Shortcut = close_shortcut_str.parse().map_err(|e| {
                let err_msg = format!(
                    "Invalid reader close shortcut '{}': {:?}",
                    close_shortcut_str, e
                );
                log::error!("{}", err_msg);
                err_msg
            })?;

            let app_handle_close = app_handle.clone();
            app.global_shortcut()
                .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_handle2 = app_handle_close.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(window) = app_handle2.get_webview_window("reader") {
                                if window.is_visible().unwrap_or(false) {
                                    warn_if_failed(window.hide(), "Failed to hide reader window");
                                    let state = app_handle2.state::<crate::app_state::AppState>();
                                    state.edge_detector.set_reader_open(false).await;
                                }
                            }
                        });
                    }
                })
                .map_err(|e| {
                    let err_msg = format!(
                        "Failed to register reader close shortcut '{}': {:?}",
                        close_shortcut_str, e
                    );
                    log::error!("{}", err_msg);
                    err_msg
                })?;

            registered_shortcuts.push(close_shortcut_str.clone());
        }

        *self.current_shortcuts.lock().await = registered_shortcuts;
        Ok(())
    }
}

pub fn normalize_shortcut(shortcut: &str) -> String {
    if shortcut.contains("CommandOrControl") {
        return shortcut.to_string();
    }

    let parts: Vec<&str> = shortcut.split('+').collect();
    let normalized: Vec<String> = parts
        .iter()
        .map(|part| {
            let trimmed = part.trim();
            match trimmed {
                "Cmd" | "Command" => "CommandOrControl".to_string(),
                "Ctrl" | "Control" => "CommandOrControl".to_string(),
                "Option" | "Opt" => "Alt".to_string(),
                _ => trimmed.to_string(),
            }
        })
        .collect();

    normalized.join("+")
}

/// Parse a shortcut string to verify it's valid
pub fn validate_shortcut(shortcut: &str) -> Result<(), String> {
    let normalized = normalize_shortcut(shortcut);

    // Basic validation: should contain at least one modifier and one key
    let parts: Vec<&str> = normalized.split('+').collect();

    if parts.len() < 2 {
        return Err("Shortcut must contain at least one modifier and one key".to_string());
    }

    // Check for valid modifiers
    let valid_modifiers = ["CommandOrControl", "Shift", "Alt", "Super"];
    let valid_keys = [
        "A",
        "B",
        "C",
        "D",
        "E",
        "F",
        "G",
        "H",
        "I",
        "J",
        "K",
        "L",
        "M",
        "N",
        "O",
        "P",
        "Q",
        "R",
        "S",
        "T",
        "U",
        "V",
        "W",
        "X",
        "Y",
        "Z",
        "0",
        "1",
        "2",
        "3",
        "4",
        "5",
        "6",
        "7",
        "8",
        "9",
        "F1",
        "F2",
        "F3",
        "F4",
        "F5",
        "F6",
        "F7",
        "F8",
        "F9",
        "F10",
        "F11",
        "F12",
        "Space",
        "Tab",
        "Enter",
        "Escape",
        "Backspace",
        "Delete",
        "Up",
        "Down",
        "Left",
        "Right",
        "Home",
        "End",
        "PageUp",
        "PageDown",
    ];

    let mut has_modifier = false;
    let mut has_key = false;

    for part in &parts {
        let part = part.trim();
        if valid_modifiers.contains(&part) {
            has_modifier = true;
        } else if valid_keys.iter().any(|k| k.eq_ignore_ascii_case(part)) {
            has_key = true;
        }
    }

    if !has_modifier {
        return Err("Shortcut must contain at least one modifier (Cmd, Shift, Alt)".to_string());
    }

    if !has_key {
        return Err("Shortcut must contain a valid key".to_string());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_shortcut() {
        assert_eq!(
            normalize_shortcut("Cmd+Shift+N"),
            "CommandOrControl+Shift+N"
        );
        assert_eq!(
            normalize_shortcut("Command+Option+K"),
            "CommandOrControl+Alt+K"
        );
    }

    #[test]
    fn test_validate_shortcut() {
        assert!(validate_shortcut("Cmd+Shift+N").is_ok());
        assert!(validate_shortcut("Cmd+N").is_ok());
        assert!(validate_shortcut("N").is_err());
        assert!(validate_shortcut("Cmd").is_err());
    }
}
