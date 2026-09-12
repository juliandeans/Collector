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

fn show_and_focus_capture(app: &AppHandle) {
    warn_if_failed(app.emit("show_capture", ()), "Failed to emit show_capture");
    if let Some(window) = app.get_webview_window("capture") {
        warn_if_failed(window.show(), "Failed to show capture window");
        warn_if_failed(window.set_focus(), "Failed to focus capture window");
    }
}

async fn hide_window_and_update_edge(app: &AppHandle, window_label: &str) {
    if let Some(window) = app.get_webview_window(window_label) {
        if window.is_visible().unwrap_or(false) {
            warn_if_failed(
                window.hide(),
                &format!("Failed to hide {} window", window_label),
            );
            let state = app.state::<crate::app_state::AppState>();
            if window_label == "capture" {
                state.edge_detector.set_capture_open(false).await;
            } else if window_label == "reader" {
                state.edge_detector.set_reader_open(false).await;
            }
        }
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

    async fn register_single_action<F, Fut>(
        &self,
        app: &AppHandle,
        context: &str,
        raw_shortcut: &str,
        action: F,
    ) -> Result<(), String>
    where
        F: Fn(AppHandle) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        let shortcut_str = normalize_shortcut(raw_shortcut);
        self.unregister_current_shortcuts(app, context).await;

        if shortcut_str.trim().is_empty() {
            return Ok(());
        }

        let shortcut: Shortcut = shortcut_str.parse().map_err(|e| {
            let err_msg = format!("Invalid {} shortcut '{}': {:?}", context, shortcut_str, e);
            log::error!("{}", err_msg);
            err_msg
        })?;

        let app_handle = app.clone();
        let action = Arc::new(action);

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    let app_handle = app_handle.clone();
                    let action = action.clone();
                    tauri::async_runtime::spawn(async move {
                        action(app_handle).await;
                    });
                }
            })
            .map_err(|e| {
                let err_msg = format!(
                    "Failed to register {} shortcut '{}': {:?}",
                    context, shortcut_str, e
                );
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcuts.lock().await = vec![shortcut_str];
        Ok(())
    }

    async fn register_window_shortcuts<F>(
        &self,
        app: &AppHandle,
        context: &str,
        window_label: &'static str,
        open_shortcut: &str,
        close_shortcut: &str,
        closes_window: bool,
        on_show: F,
    ) -> Result<(), String>
    where
        F: Fn(&AppHandle) + Send + Sync + 'static,
    {
        let open_shortcut_str = normalize_shortcut(open_shortcut);
        let close_shortcut_str = normalize_shortcut(close_shortcut);
        self.unregister_current_shortcuts(app, context).await;

        let mut registered_shortcuts = Vec::new();
        let app_handle = app.clone();
        let on_show = Arc::new(on_show);

        if !open_shortcut_str.trim().is_empty() {
            let shortcut: Shortcut = open_shortcut_str.parse().map_err(|e| {
                let err_msg = format!("Invalid {} shortcut '{}': {:?}", context, open_shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

            let app_handle_open = app_handle.clone();
            let on_show = on_show.clone();
            app.global_shortcut()
                .on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_h = app_handle_open.clone();
                        let on_show = on_show.clone();
                        tauri::async_runtime::spawn(async move {
                            if let Some(window) = app_h.get_webview_window(window_label) {
                                if window.is_visible().unwrap_or(false) {
                                    if closes_window {
                                        hide_window_and_update_edge(&app_h, window_label).await;
                                    } else {
                                        warn_if_failed(
                                            window.show(),
                                            &format!("Failed to show {} window", window_label),
                                        );
                                        warn_if_failed(
                                            window.set_focus(),
                                            &format!("Failed to focus {} window", window_label),
                                        );
                                    }
                                    return;
                                }
                            }

                            on_show(&app_h);
                        });
                    }
                })
                .map_err(|e| {
                    let err_msg = format!(
                        "Failed to register {} shortcut '{}': {:?}",
                        context, open_shortcut_str, e
                    );
                    log::error!("{}", err_msg);
                    err_msg
                })?;

            registered_shortcuts.push(open_shortcut_str);
        }

        if !closes_window && !close_shortcut_str.trim().is_empty() {
            let shortcut: Shortcut = close_shortcut_str.parse().map_err(|e| {
                let err_msg = format!(
                    "Invalid {} close shortcut '{}': {:?}",
                    context, close_shortcut_str, e
                );
                log::error!("{}", err_msg);
                err_msg
            })?;

            let app_handle_close = app_handle.clone();
            app.global_shortcut()
                .on_shortcut(shortcut, move |_app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        let app_h = app_handle_close.clone();
                        tauri::async_runtime::spawn(async move {
                            hide_window_and_update_edge(&app_h, window_label).await;
                        });
                    }
                })
                .map_err(|e| {
                    let err_msg = format!(
                        "Failed to register {} close shortcut '{}': {:?}",
                        context, close_shortcut_str, e
                    );
                    log::error!("{}", err_msg);
                    err_msg
                })?;

            registered_shortcuts.push(close_shortcut_str);
        }

        *self.current_shortcuts.lock().await = registered_shortcuts;
        Ok(())
    }

    pub async fn register(&self, app: &AppHandle, settings: &Settings) -> Result<(), String> {
        self.register_window_shortcuts(
            app,
            "global",
            "capture",
            &settings.global_shortcut,
            &settings.global_close_shortcut,
            settings.global_shortcut_closes_window,
            |app| {
                warn_if_failed(app.emit("reset_capture", ()), "Failed to emit reset_capture");
                show_and_focus_capture(app);
            },
        )
        .await
    }

    pub async fn update(&self, app: &AppHandle, settings: &Settings) -> Result<(), String> {
        self.register(app, settings).await
    }

    pub async fn register_capture_text(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        self.register_single_action(app, "capture_text", &settings.capture_text_shortcut, |app| async move {
            let controller = app
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
            show_and_focus_capture(&app);

            if selected.trim().is_empty() {
                log::warn!("No text was captured");
                warn_if_failed(
                    app.emit(
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
                app.emit("insert_capture_text", selected),
                "Failed to emit insert_capture_text",
            );
        }).await
    }

    pub async fn register_save_as_note(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        self.register_single_action(app, "save_as_note", &settings.save_as_note_shortcut, |app| async move {
            warn_if_failed(
                app.emit("save_as_note", ()),
                "Failed to emit save_as_note",
            );
        }).await
    }

    pub async fn register_capture_mode(
        &self,
        app: &AppHandle,
        settings: &Settings,
        controller: Arc<CaptureModeController>,
    ) -> Result<(), String> {
        self.register_single_action(app, "capture_mode", &settings.capture_mode_shortcut, move |app| {
            let controller = controller.clone();
            async move {
                if let Err(error) = controller.toggle(&app).await {
                    log::warn!("Failed to toggle capture mode: {}", error);
                }
            }
        }).await
    }

    pub async fn register_ocr(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        self.register_single_action(app, "ocr", &settings.ocr_shortcut, |app| async move {
            let ocr_result = tauri::async_runtime::spawn_blocking(
                crate::ocr::ocr_clipboard,
            )
            .await;

            show_and_focus_capture(&app);

            match ocr_result {
                Ok(Ok(ocr)) => {
                    log::info!(
                        "OCR text extracted length={}",
                        summarize_text_len(&ocr.text)
                    );
                    tokio::time::sleep(tokio::time::Duration::from_millis(
                        CAPTURE_TEXT_INSERT_DELAY_MS,
                    ))
                    .await;
                    warn_if_failed(
                        app.emit("insert_capture_text", ocr.text),
                        "Failed to emit insert_capture_text",
                    );
                }
                Ok(Err(err_msg)) => {
                    log::warn!("OCR failed: {}", err_msg);
                    warn_if_failed(
                        app.emit("capture_text_failed", err_msg),
                        "Failed to emit capture_text_failed",
                    );
                }
                Err(e) => {
                    log::error!("OCR task join error: {}", e);
                    warn_if_failed(
                        app.emit(
                            "capture_text_failed",
                            "OCR failed to process image",
                        ),
                        "Failed to emit capture_text_failed",
                    );
                }
            }
        }).await
    }

    pub async fn register_reader(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        self.register_window_shortcuts(
            app,
            "reader",
            "reader",
            &settings.reader_shortcut,
            &settings.reader_close_shortcut,
            settings.reader_shortcut_closes_window,
            |app| {
                warn_if_failed(app.emit("show_reader", ()), "Failed to emit show_reader");
            },
        )
        .await
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
