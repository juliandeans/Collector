/// Capture selected text via clipboard-sentinel fallback.
///
/// Strategy:
/// 1. Save current clipboard content (plain text, via `pbpaste`)
/// 2. Write a unique sentinel to clipboard (via `pbcopy`)
/// 3. Wait briefly for shortcut modifiers to release (Cmd+Shift+C → plain Cmd+C)
/// 4. Log which app is frontmost before sending Cmd+C
/// 5. Send Cmd+C to frontmost app via `osascript` (System Events keystroke)
/// 6. Poll clipboard every 50 ms up to ~800 ms until sentinel is gone
/// 7. If clipboard changed -> capture text, restore original clipboard, return text
/// 8. If sentinel remains -> restore original clipboard, return None
///
/// Notes:
/// - pbcopy/pbpaste only handle plain text NSPasteboard type (NSStringPboardType).
///   Rich clipboard content (images, RTF, etc.) is NOT restored -- this is a
///   plain-text-only save/restore for the capture use case.
/// - The sentinel avoids a false negative when selected text happens to match
///   the previous clipboard content.
#[cfg(target_os = "macos")]
pub fn capture_selected_text() -> Option<String> {
    log::debug!("clipboard fallback started");

    // 1. Save current clipboard content
    let previous = run_pbpaste();
    log::debug!(
        "previous clipboard text saved: {}, length={}",
        previous.is_some(),
        previous.as_deref().map(|s| s.len()).unwrap_or(0)
    );

    // 2. Write a unique sentinel
    let sentinel = format!(
        "__COLLECTOR_CLIPBOARD_SENTINEL_{}__",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    );
    if !run_pbcopy(&sentinel) {
        log::error!("failed to write sentinel to clipboard");
        return None;
    }
    log::info!("sentinel written");

    // 3. Small delay so the user can release Cmd+Shift before we send Cmd+C.
    //    Without this, the physical Shift key might still be held from the
    //    global shortcut (Cmd+Shift+C), causing Cmd+Shift+C to arrive at
    //    the target app instead of plain Cmd+C.
    log::info!("waiting for shortcut modifiers to release: 150 ms");
    std::thread::sleep(std::time::Duration::from_millis(150));

    // 4. Log which app is frontmost right before we send the keystroke.
    //    If Collector is frontmost at this point, the capture window opened too early.
    if let Some(app_name) = run_frontmost_app_name() {
        log::info!("frontmost app before Cmd+C: {}", app_name);
    } else {
        log::warn!("could not determine frontmost app before Cmd+C");
    }

    // 5. Send Cmd+C
    if !run_keystroke_copy() {
        log::error!("osascript keystroke failed - Cmd+C may not have been sent.");
        log::error!("macOS may be blocking Collector from controlling System Events.");
        if let Some(prev) = previous {
            run_pbcopy(&prev);
        }
        return None;
    }
    log::info!("Cmd+C sent");

    // 6. Poll clipboard until sentinel is replaced (or timeout)
    let poll_interval = std::time::Duration::from_millis(50);
    let max_attempts = 16; // 16 x 50 ms = 800 ms
    let mut captured: Option<String> = None;

    for attempt in 1..=max_attempts {
        std::thread::sleep(poll_interval);

        let current = run_pbpaste();
        match current {
            Some(ref text) if text == &sentinel => {
                continue;
            }
            Some(text) => {
                log::info!("clipboard replaced sentinel after {} ms", attempt * 50);
                let trimmed = text.trim().to_string();
                if !trimmed.is_empty() {
                    captured = Some(trimmed);
                }
                break;
            }
            None => {
                log::debug!("clipboard became empty after {} ms", attempt * 50);
                break;
            }
        }
    }

    if captured.is_none() {
        log::warn!(
            "no text captured; sentinel unchanged after {} ms",
            max_attempts * 50
        );
    } else {
        log::info!(
            "captured text length={}",
            captured.as_ref().map(|s| s.len()).unwrap_or(0)
        );
    }

    // 7. Restore original clipboard
    let restored = if let Some(prev) = &previous {
        run_pbcopy(prev)
    } else {
        run_pbcopy("")
    };
    log::info!("clipboard restored: {}", restored);

    captured
}

#[cfg(not(target_os = "macos"))]
pub fn capture_selected_text() -> Option<String> {
    None
}

// =============================================================================
// macOS helpers (built-in CLI tools, no FFI)
// =============================================================================

/// Read current clipboard content via `pbpaste`.
/// Returns `None` if clipboard is empty or unreadable.
fn run_pbpaste() -> Option<String> {
    let output = std::process::Command::new("pbpaste")
        .env_remove("LANG")
        .output()
        .ok()?;
    if output.status.success() && !output.stdout.is_empty() {
        String::from_utf8(output.stdout).ok()
    } else {
        None
    }
}

/// Write text to clipboard via `pbcopy`.
/// Returns true if the write succeeded.
fn run_pbcopy(text: &str) -> bool {
    use std::io::Write;
    let mut child = match std::process::Command::new("pbcopy")
        .stdin(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            log::error!("pbcopy spawn failed: {}", e);
            return false;
        }
    };
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(text.as_bytes());
        let _ = stdin.flush();
    }
    match child.wait() {
        Ok(status) => status.success(),
        Err(e) => {
            log::error!("pbcopy wait failed: {}", e);
            false
        }
    }
}

/// Get the name of the currently frontmost application via `osascript`.
fn run_frontmost_app_name() -> Option<String> {
    let output = std::process::Command::new("osascript")
        .args(&[
            "-e",
            "tell application \"System Events\" to get name of first application process whose frontmost is true",
        ])
        .output()
        .ok()?;
    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            None
        } else {
            Some(name)
        }
    } else {
        None
    }
}

/// Send Cmd+C to the frontmost app via `osascript`.
/// Uses `System Events` which performs the keystroke on behalf of Collector.
/// This may require macOS Accessibility or Automation permission.
fn run_keystroke_copy() -> bool {
    let output = std::process::Command::new("osascript")
        .args(&[
            "-e",
            "tell application \"System Events\" to keystroke \"c\" using command down",
        ])
        .output()
        .ok();

    match output {
        Some(out) if out.status.success() => true,
        Some(out) => {
            let stderr = String::from_utf8_lossy(&out.stderr);
            log::error!("osascript keystroke failed: {}", stderr.trim());
            false
        }
        None => {
            log::error!("osascript command failed to spawn");
            false
        }
    }
}
