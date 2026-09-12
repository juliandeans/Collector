use std::sync::{Arc, Mutex, MutexGuard};
use std::time::Duration;

use serde::Serialize;
use tauri::async_runtime::JoinHandle;
use tauri::{AppHandle, Emitter, Manager};

const CLIPBOARD_POLL_INTERVAL_MS: u64 = 200;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ClipboardDecision {
    Ignore,
    Append(String),
}

/// Decide whether a clipboard snapshot should be appended.
///
/// The caller updates its baseline for every new change count, including
/// snapshots that do not contain usable plain text.
pub(crate) fn decide_clipboard_change(
    active: bool,
    last_seen_change_count: Option<i64>,
    change_count: i64,
    text: Option<&str>,
) -> ClipboardDecision {
    if !active || last_seen_change_count == Some(change_count) {
        return ClipboardDecision::Ignore;
    }

    match text.map(str::trim).filter(|text| !text.is_empty()) {
        Some(text) => ClipboardDecision::Append(text.to_string()),
        None => ClipboardDecision::Ignore,
    }
}

#[derive(Default)]
struct ControllerState {
    active: bool,
    last_pasteboard_change_count: Option<i64>,
    suppression_depth: usize,
    generation: u64,
}

#[derive(Clone, Serialize)]
struct CaptureModeStatePayload {
    active: bool,
}

pub(crate) struct CaptureModeController {
    state: Arc<Mutex<ControllerState>>,
    watcher: Mutex<Option<JoinHandle<()>>>,
}

impl CaptureModeController {
    pub(crate) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(ControllerState::default())),
            watcher: Mutex::new(None),
        }
    }

    pub(crate) fn is_active(&self) -> bool {
        self.lock_state().active
    }

    pub(crate) async fn set_enabled(
        &self,
        app: &AppHandle,
        requested: bool,
    ) -> Result<bool, String> {
        self.update_enabled(app, Some(requested)).await
    }

    pub(crate) async fn toggle(&self, app: &AppHandle) -> Result<bool, String> {
        self.update_enabled(app, None).await
    }

    /// Temporarily suppress all clipboard observations made by the selected
    /// text fallback. The suppression depth and change-count baseline live in
    /// the same mutex as the watcher decision, so no poll can observe one
    /// without the other.
    pub(crate) fn suppress_watcher(&self, suppressed: bool) {
        let mut state = self.lock_state();

        if suppressed {
            state.suppression_depth = state.suppression_depth.saturating_add(1);
            return;
        }

        if state.suppression_depth == 0 {
            return;
        }

        state.suppression_depth -= 1;
        if state.suppression_depth == 0 {
            state.last_pasteboard_change_count = read_pasteboard_change_count();
        }
    }

    async fn update_enabled(
        &self,
        app: &AppHandle,
        requested: Option<bool>,
    ) -> Result<bool, String> {
        let (changed, active) = {
            // Serializing state changes with watcher replacement prevents a
            // stale enable task from installing a handle after a disable.
            let mut watcher = self
                .watcher
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            let mut state = self.lock_state();
            let active = requested.unwrap_or(!state.active);

            if state.active == active {
                return Ok(active);
            }

            state.active = active;
            state.generation = state.generation.wrapping_add(1);
            let generation = state.generation;

            if active {
                state.last_pasteboard_change_count = read_pasteboard_change_count();
                if let Some(previous) = watcher.take() {
                    previous.abort();
                }

                let controller_state = self.state.clone();
                let app_handle = app.clone();
                *watcher = Some(tauri::async_runtime::spawn(async move {
                    let mut interval =
                        tokio::time::interval(Duration::from_millis(CLIPBOARD_POLL_INTERVAL_MS));
                    interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

                    loop {
                        interval.tick().await;
                        if !poll_clipboard(&app_handle, &controller_state, generation) {
                            break;
                        }
                    }
                }));
            } else {
                state.last_pasteboard_change_count = None;
                if let Some(previous) = watcher.take() {
                    previous.abort();
                }
            }

            (true, active)
        };

        if changed {
            if active {
                log::info!("Capture mode enabled");
            } else {
                log::info!("Capture mode disabled");
            }
            emit_state_changed(app, active);
        }

        Ok(active)
    }

    fn lock_state(&self) -> MutexGuard<'_, ControllerState> {
        self.state.lock().unwrap_or_else(|error| error.into_inner())
    }
}

fn emit_state_changed(app: &AppHandle, active: bool) {
    if let Some(window) = app.get_webview_window("capture") {
        if let Err(error) = window.emit(
            "capture_mode_state_changed",
            CaptureModeStatePayload { active },
        ) {
            log::warn!("Failed to emit capture mode state: {}", error);
        }
    }
}

fn poll_clipboard(app: &AppHandle, state: &Arc<Mutex<ControllerState>>, generation: u64) -> bool {
    let mut state = state.lock().unwrap_or_else(|error| error.into_inner());
    if !state.active || state.generation != generation {
        return false;
    }

    if state.suppression_depth > 0 {
        return true;
    }

    let Some(change_count) = read_pasteboard_change_count() else {
        return true;
    };

    if state.last_pasteboard_change_count == Some(change_count) {
        return true;
    }

    let text = read_pasteboard_text();
    let decision = decide_clipboard_change(
        state.active,
        state.last_pasteboard_change_count,
        change_count,
        text.as_deref(),
    );
    state.last_pasteboard_change_count = Some(change_count);

    if let ClipboardDecision::Append(text) = decision {
        if let Some(window) = app.get_webview_window("capture") {
            if let Err(error) = window.emit("capture_mode_append", &text) {
                log::warn!("Failed to emit capture mode text: {}", error);
            } else {
                log::info!("Clipboard text captured ({} chars)", text.chars().count());
            }
        }
    }

    true
}

#[cfg(target_os = "macos")]
fn read_pasteboard_change_count() -> Option<i64> {
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};

    unsafe {
        let pool: id = msg_send![class!(NSAutoreleasePool), new];
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard == nil {
            let _: () = msg_send![pool, drain];
            return None;
        }

        let change_count: isize = msg_send![pasteboard, changeCount];
        let _: () = msg_send![pool, drain];
        Some(change_count as i64)
    }
}

#[cfg(not(target_os = "macos"))]
fn read_pasteboard_change_count() -> Option<i64> {
    None
}

#[cfg(target_os = "macos")]
fn read_pasteboard_text() -> Option<String> {
    use cocoa::appkit::NSPasteboardTypeString;
    use cocoa::base::{id, nil};
    use objc::{class, msg_send, sel, sel_impl};
    use std::ffi::CStr;
    use std::os::raw::c_char;

    unsafe {
        let pool: id = msg_send![class!(NSAutoreleasePool), new];
        let pasteboard: id = msg_send![class!(NSPasteboard), generalPasteboard];
        if pasteboard == nil {
            let _: () = msg_send![pool, drain];
            return None;
        }

        let string: id = msg_send![pasteboard, stringForType: NSPasteboardTypeString];
        let text = if string == nil {
            None
        } else {
            let utf8: *const c_char = msg_send![string, UTF8String];
            if utf8.is_null() {
                None
            } else {
                Some(CStr::from_ptr(utf8).to_string_lossy().into_owned())
            }
        };

        let _: () = msg_send![pool, drain];
        text
    }
}

#[cfg(not(target_os = "macos"))]
fn read_pasteboard_text() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_ignores_clipboard_change() {
        let decision = decide_clipboard_change(false, None, 5, Some("hello"));
        assert_eq!(decision, ClipboardDecision::Ignore);
    }

    #[test]
    fn active_with_unchanged_change_count_ignores() {
        let decision = decide_clipboard_change(true, Some(5), 5, Some("hello"));
        assert_eq!(decision, ClipboardDecision::Ignore);
    }

    #[test]
    fn active_with_new_change_count_and_text_appends() {
        let decision = decide_clipboard_change(true, Some(5), 6, Some("hello"));
        assert_eq!(decision, ClipboardDecision::Append("hello".to_string()));
    }

    #[test]
    fn active_with_new_change_count_and_empty_text_ignores() {
        let decision = decide_clipboard_change(true, Some(5), 6, Some("   "));
        assert_eq!(decision, ClipboardDecision::Ignore);
    }

    #[test]
    fn active_with_new_change_count_and_no_text_ignores() {
        let decision = decide_clipboard_change(true, Some(5), 6, None);
        assert_eq!(decision, ClipboardDecision::Ignore);
    }

    #[test]
    fn identical_text_at_a_new_change_count_is_appended_again() {
        let first = decide_clipboard_change(true, Some(5), 6, Some("same"));
        assert_eq!(first, ClipboardDecision::Append("same".to_string()));

        // No deduplication: the same text at a later change count still counts
        // as a new clipboard entry.
        let second = decide_clipboard_change(true, Some(6), 7, Some("same"));
        assert_eq!(second, ClipboardDecision::Append("same".to_string()));
    }

    #[test]
    fn text_is_trimmed_before_being_appended() {
        let decision = decide_clipboard_change(true, Some(5), 6, Some("  hello  "));
        assert_eq!(decision, ClipboardDecision::Append("hello".to_string()));
    }

    #[test]
    fn suppression_nests_and_only_fully_releases_at_depth_zero() {
        let controller = CaptureModeController::new();

        controller.suppress_watcher(true);
        controller.suppress_watcher(true);
        assert_eq!(controller.lock_state().suppression_depth, 2);

        controller.suppress_watcher(false);
        assert_eq!(controller.lock_state().suppression_depth, 1);

        controller.suppress_watcher(false);
        assert_eq!(controller.lock_state().suppression_depth, 0);
    }

    #[test]
    fn releasing_an_unsuppressed_controller_is_a_no_op() {
        let controller = CaptureModeController::new();
        controller.suppress_watcher(false);
        assert_eq!(controller.lock_state().suppression_depth, 0);
    }
}
