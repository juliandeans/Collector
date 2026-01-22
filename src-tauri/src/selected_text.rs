/// Capture the currently selected text from the active application (macOS).
///
/// Implementation strategy:
/// - Save current clipboard string
/// - Synthesize Cmd+C to copy selection into clipboard
/// - Read clipboard string
/// - Restore previous clipboard
///
/// Notes:
/// - Requires Accessibility permission for the app to send synthetic key events.
/// - If nothing is selected, many apps keep clipboard unchanged; we return whatever was copied.
#[cfg(target_os = "macos")]
pub fn capture_selected_text() -> Option<String> {
    log::info!("capture_selected_text: Starting...");

    let previous = read_clipboard_string();
    log::info!(
        "capture_selected_text: Previous clipboard length={}",
        previous
            .as_deref()
            .map(crate::log_safety::summarize_text_len)
            .unwrap_or(0)
    );

    // Trigger "Copy" in the currently focused app.
    log::info!("capture_selected_text: Synthesizing Cmd+C...");
    if !synthesize_copy() {
        log::error!("capture_selected_text: Failed to synthesize copy!");
        return None;
    }
    log::info!("capture_selected_text: Cmd+C synthesized successfully");

    // Give the target app a moment to update the clipboard.
    // Increased delay since we now have delays in the key synthesis itself
    std::thread::sleep(std::time::Duration::from_millis(250));

    let captured = read_clipboard_string();
    log::info!(
        "capture_selected_text: Captured clipboard length={}",
        captured
            .as_deref()
            .map(crate::log_safety::summarize_text_len)
            .unwrap_or(0)
    );

    // Restore previous clipboard to avoid disrupting the user.
    if let Some(prev) = previous.as_deref() {
        log::info!("capture_selected_text: Restoring previous clipboard");
        write_clipboard_string(prev);
    }

    log::info!("capture_selected_text: Returning captured text");
    captured
}

#[cfg(not(target_os = "macos"))]
pub fn capture_selected_text() -> Option<String> {
    None
}

// =============================================================================
// macOS implementation details
// =============================================================================

#[cfg(target_os = "macos")]
fn read_clipboard_string() -> Option<String> {
    use cocoa::appkit::{NSPasteboard, NSPasteboardTypeString};
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;
    use std::ffi::CStr;

    unsafe {
        let pb: id = NSPasteboard::generalPasteboard(nil);
        if pb == nil {
            return None;
        }
        let s: id = pb.stringForType(NSPasteboardTypeString);
        if s == nil {
            return None;
        }
        let c_str = NSString::UTF8String(s);
        if c_str.is_null() {
            return None;
        }
        Some(CStr::from_ptr(c_str).to_string_lossy().into_owned())
    }
}

#[cfg(target_os = "macos")]
fn write_clipboard_string(value: &str) {
    use cocoa::appkit::{NSPasteboard, NSPasteboardTypeString};
    use cocoa::base::{id, nil};
    use cocoa::foundation::NSString;

    unsafe {
        let pb: id = NSPasteboard::generalPasteboard(nil);
        if pb == nil {
            return;
        }
        // clearContents returns an integer (not an Objective-C object)
        let _ = pb.clearContents();
        let ns_string = NSString::alloc(nil).init_str(value);
        let _: bool = pb.setString_forType(ns_string, NSPasteboardTypeString);
    }
}

#[cfg(target_os = "macos")]
fn synthesize_copy() -> bool {
    use core_graphics::event::{
        CGEvent, CGEventFlags, CGEventTapLocation, CGKeyCode,
    };
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // Check if we have accessibility permissions
    if !check_accessibility_permissions() {
        log::error!("synthesize_copy: NO ACCESSIBILITY PERMISSIONS!");
        log::error!("Please grant Accessibility permissions in System Settings > Privacy & Security > Accessibility");
        return false;
    }

    // macOS virtual keycodes
    const KEY_C: CGKeyCode = 8;
    const KEY_CMD: CGKeyCode = 55;

    let src = CGEventSource::new(CGEventSourceStateID::CombinedSessionState);
    let Ok(src) = src else {
        log::error!("synthesize_copy: Failed to create event source");
        return false;
    };

    // Small delay to ensure the target app is ready
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Press Command
    if let Ok(cmd_down) = CGEvent::new_keyboard_event(src.clone(), KEY_CMD, true) {
        cmd_down.post(CGEventTapLocation::AnnotatedSession);
        std::thread::sleep(std::time::Duration::from_millis(20));
    } else {
        return false;
    }

    // Press C with Command flag
    if let Ok(c_down) = CGEvent::new_keyboard_event(src.clone(), KEY_C, true) {
        c_down.set_flags(CGEventFlags::CGEventFlagCommand);
        c_down.post(CGEventTapLocation::AnnotatedSession);
        std::thread::sleep(std::time::Duration::from_millis(20));
    } else {
        return false;
    }

    // Release C
    if let Ok(c_up) = CGEvent::new_keyboard_event(src.clone(), KEY_C, false) {
        c_up.set_flags(CGEventFlags::CGEventFlagCommand);
        c_up.post(CGEventTapLocation::AnnotatedSession);
        std::thread::sleep(std::time::Duration::from_millis(20));
    } else {
        return false;
    }

    // Release Command
    if let Ok(cmd_up) = CGEvent::new_keyboard_event(src, KEY_CMD, false) {
        cmd_up.post(CGEventTapLocation::AnnotatedSession);
        std::thread::sleep(std::time::Duration::from_millis(20));
    } else {
        return false;
    }

    true
}

/// Check if the app has Accessibility permissions on macOS
#[cfg(target_os = "macos")]
fn check_accessibility_permissions() -> bool {
    use core_graphics::event_source::{CGEventSource, CGEventSourceStateID};

    // Try to create an event source - this will fail if we don't have permissions
    let src = match CGEventSource::new(CGEventSourceStateID::CombinedSessionState) {
        Ok(s) => s,
        Err(_) => {
            log::error!("Failed to create event source - no accessibility permissions");
            return false;
        }
    };

    // Try to create a simple event - this is the real test
    match core_graphics::event::CGEvent::new(src) {
        Ok(_) => true,
        Err(e) => {
            log::error!("Failed to create CGEvent: {:?} - likely missing accessibility permissions", e);
            false
        }
    }
}
