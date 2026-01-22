use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};
use tokio::sync::Mutex;

use crate::log_safety::summarize_text_len;
use crate::settings::Settings;

pub struct ShortcutManager {
    current_shortcut: Arc<Mutex<Option<String>>>,
}

impl ShortcutManager {
    pub fn new() -> Self {
        Self {
            current_shortcut: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn register(&self, app: &AppHandle, settings: &Settings) -> Result<(), String> {
        let shortcut_str = normalize_shortcut(&settings.global_shortcut);
        log::info!("Attempting to register global shortcut: '{}'", shortcut_str);

        // Skip if shortcut is empty
        if shortcut_str.trim().is_empty() {
            log::info!("Global shortcut is empty, skipping registration");
            // Clear any stored shortcut
            *self.current_shortcut.lock().await = None;
            return Ok(());
        }

        // Check if we need to unregister the old one
        let old_shortcut = self.current_shortcut.lock().await.clone();
        if let Some(old) = old_shortcut {
            if old != shortcut_str {
                log::info!("Unregistering old shortcut: {}", old);
                if let Ok(shortcut) = old.parse::<Shortcut>() {
                    let _ = app.global_shortcut().unregister(shortcut);
                }
            } else {
                log::info!("Shortcut unchanged, skipping re-registration");
                return Ok(());
            }
        }

        log::info!("Parsing shortcut: '{}'", shortcut_str);
        let shortcut: Shortcut = shortcut_str
            .parse()
            .map_err(|e| {
                let err_msg = format!("Invalid shortcut '{}': {:?}", shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

        log::info!("Registering shortcut handler...");
        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    log::info!("Global shortcut triggered (open window)");
                    let app_handle2 = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        // Open/focus capture window and reset UI state
                        let _ = app_handle2.emit("show_capture", ());
                        if let Some(window) = app_handle2.get_webview_window("capture") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }
                    });
                }
            })
            .map_err(|e| {
                let err_msg = format!("Failed to register shortcut '{}': {:?}", shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcut.lock().await = Some(shortcut_str.clone());
        log::info!("Global shortcut successfully registered: {}", shortcut_str);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn unregister(&self, app: &AppHandle) -> Result<(), String> {
        let shortcut_str = self.current_shortcut.lock().await.take();
        if let Some(shortcut_str) = shortcut_str {
            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                let _ = app.global_shortcut().unregister(shortcut);
                log::info!("Unregistered shortcut: {}", shortcut_str);
            }
        }
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
        log::info!("Attempting to register capture_text shortcut: '{}'", shortcut_str);

        // Skip if shortcut is empty
        if shortcut_str.trim().is_empty() {
            log::info!("Capture text shortcut is empty, skipping registration");
            *self.current_shortcut.lock().await = None;
            return Ok(());
        }

        // Check if we need to unregister the old one
        let old_shortcut = self.current_shortcut.lock().await.clone();
        if let Some(old) = old_shortcut {
            if old != shortcut_str {
                log::info!("Unregistering old capture_text shortcut: {}", old);
                if let Ok(shortcut) = old.parse::<Shortcut>() {
                    let _ = app.global_shortcut().unregister(shortcut);
                }
            } else {
                log::info!("Capture text shortcut unchanged, skipping re-registration");
                return Ok(());
            }
        }

        log::info!("Parsing capture_text shortcut: '{}'", shortcut_str);
        let shortcut: Shortcut = shortcut_str
            .parse()
            .map_err(|e| {
                let err_msg = format!("Invalid capture_text shortcut '{}': {:?}", shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

        log::info!("Registering capture_text shortcut handler...");
        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    log::info!("Capture text shortcut triggered");
                    let app_handle2 = app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        // WICHTIG: Capture text FIRST, before opening window
                        // Otherwise the window steals focus and Cmd+C goes to the wrong app
                        log::info!("Capturing selected text (BEFORE opening window)...");
                        let selected =
                            tauri::async_runtime::spawn_blocking(crate::selected_text::capture_selected_text)
                                .await
                                .ok()
                                .flatten()
                                .unwrap_or_default();

                        log::info!(
                            "Captured text length={}",
                            summarize_text_len(&selected)
                        );

                        // NOW open/focus capture window
                        let _ = app_handle2.emit("show_capture", ());
                        if let Some(window) = app_handle2.get_webview_window("capture") {
                            let _ = window.show();
                            let _ = window.set_focus();
                        }

                        if selected.trim().is_empty() {
                            log::warn!("No text was captured");
                            return;
                        }

                        // Ensure show_capture listeners ran first (they clear content).
                        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;

                        // Send to all windows (frontend listens globally)
                        log::info!("Emitting insert_capture_text event");
                        let _ = app_handle2.emit("insert_capture_text", selected);
                    });
                }
            })
            .map_err(|e| {
                let err_msg = format!("Failed to register capture_text shortcut '{}': {:?}", shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcut.lock().await = Some(shortcut_str.clone());
        log::info!("Capture text shortcut successfully registered: {}", shortcut_str);
        Ok(())
    }

    pub async fn register_save_as_note(
        &self,
        app: &AppHandle,
        settings: &Settings,
    ) -> Result<(), String> {
        let shortcut_str = normalize_shortcut(&settings.save_as_note_shortcut);
        log::info!("Attempting to register save_as_note shortcut: '{}'", shortcut_str);

        // Skip if shortcut is empty
        if shortcut_str.trim().is_empty() {
            log::info!("Save as note shortcut is empty, skipping registration");
            *self.current_shortcut.lock().await = None;
            return Ok(());
        }

        // Check if we need to unregister the old one
        let old_shortcut = self.current_shortcut.lock().await.clone();
        if let Some(old) = old_shortcut {
            if old != shortcut_str {
                log::info!("Unregistering old save_as_note shortcut: {}", old);
                if let Ok(shortcut) = old.parse::<Shortcut>() {
                    let _ = app.global_shortcut().unregister(shortcut);
                }
            } else {
                log::info!("Save as note shortcut unchanged, skipping re-registration");
                return Ok(());
            }
        }

        log::info!("Parsing save_as_note shortcut: '{}'", shortcut_str);
        let shortcut: Shortcut = shortcut_str
            .parse()
            .map_err(|e| {
                let err_msg = format!("Invalid save_as_note shortcut '{}': {:?}", shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

        log::info!("Registering save_as_note shortcut handler...");
        let app_handle = app.clone();
        app.global_shortcut()
            .on_shortcut(shortcut.clone(), move |_app, _shortcut, event| {
                if event.state == ShortcutState::Pressed {
                    log::info!("Save as note shortcut triggered");
                    let _ = app_handle.emit("save_as_note", ());
                }
            })
            .map_err(|e| {
                let err_msg = format!("Failed to register save_as_note shortcut '{}': {:?}", shortcut_str, e);
                log::error!("{}", err_msg);
                err_msg
            })?;

        *self.current_shortcut.lock().await = Some(shortcut_str.clone());
        log::info!("Save as note shortcut successfully registered: {}", shortcut_str);
        Ok(())
    }
}

fn normalize_shortcut(shortcut: &str) -> String {
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
        "A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L", "M",
        "N", "O", "P", "Q", "R", "S", "T", "U", "V", "W", "X", "Y", "Z",
        "0", "1", "2", "3", "4", "5", "6", "7", "8", "9",
        "F1", "F2", "F3", "F4", "F5", "F6", "F7", "F8", "F9", "F10", "F11", "F12",
        "Space", "Tab", "Enter", "Escape", "Backspace", "Delete",
        "Up", "Down", "Left", "Right", "Home", "End", "PageUp", "PageDown",
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
