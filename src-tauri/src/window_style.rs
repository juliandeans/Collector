// Native macOS window chrome: rounded corners, opacity, and the
// NSVisualEffectView-based background blur. Isolated from main.rs because
// none of this has anything to do with app bootstrap — it is called both
// during startup and every time settings are saved.

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
pub(crate) fn configure_macos_window(
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
pub(crate) fn configure_macos_window(
    _window: &tauri::WebviewWindow,
    _corner_radius: f64,
    _background_blur_radius: i64,
) {
}
