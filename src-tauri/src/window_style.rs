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

// CACornerMask bit values (QuartzCore) — selects which of a CALayer's four
// corners `cornerRadius` actually rounds (default is all four). The named
// constants always refer to the layer's own left/right (X) sides regardless
// of any Y-axis coordinate flipping, so pairing MinY+MaxY for one X side
// reliably selects "both corners on that side" on every layer we apply it
// to, independent of whether that layer happens to be geometry-flipped.
//
// Capture/Reader windows dock flush against a physical screen edge
// (`edge_side`/`reader_edge_side`); by default only the two corners facing
// the screen's interior are rounded, so the flush edge stays a clean
// straight line instead of visually "floating" a rounded corner in mid-air
// against the screen bezel. `round_all_corners` opts back into all four.
//
// Known limitation: this keys purely off `edge_side`/`reader_edge_side`,
// not the window's actual current position. The Capture window's
// `remember_window_geometry` setting can place it anywhere on screen
// (`position_window_logical`), so a user who both restores a floating
// remembered position *and* leaves `round_all_corners` off would see two
// square corners on a side that isn't actually against any screen edge.
// Not worth the added geometry-detection complexity for what both requires
// two independent, uncommon settings choices at once — documented instead
// of auto-detected.
#[cfg(target_os = "macos")]
fn masked_corners(edge_side: &str, round_all_corners: bool) -> u64 {
    const MIN_X_MIN_Y: u64 = 1 << 0;
    const MAX_X_MIN_Y: u64 = 1 << 1;
    const MIN_X_MAX_Y: u64 = 1 << 2;
    const MAX_X_MAX_Y: u64 = 1 << 3;

    if round_all_corners {
        return MIN_X_MIN_Y | MAX_X_MIN_Y | MIN_X_MAX_Y | MAX_X_MAX_Y;
    }

    match edge_side {
        // Docked at the left screen edge: the window's left corners touch
        // the edge, round the right (interior-facing) corners instead.
        // Matches src/lib/cornerRadius.js's fallback convention exactly
        // (also keyed on "left", defaulting everything else to "right"
        // behavior), so an unrecognized value never disagrees between the
        // native mask here and the CSS shape there.
        "left" => MAX_X_MIN_Y | MAX_X_MAX_Y,
        // Docked right (or any other/unrecognized value): round the left
        // corners.
        _ => MIN_X_MIN_Y | MIN_X_MAX_Y,
    }
}

#[cfg(target_os = "macos")]
pub(crate) fn configure_macos_window(
    window: &tauri::WebviewWindow,
    corner_radius: f64,
    background_blur_radius: i64,
    edge_side: &str,
    round_all_corners: bool,
) {
    let masked_corners_value = masked_corners(edge_side, round_all_corners);

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
            let _: () = msg_send![layer, setMaskedCorners: masked_corners_value];

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

    // window-vibrancy's EffectsBuilder only exposes a uniform .radius(), no
    // per-corner masking, so the NSVisualEffectView it just added is still
    // rounded on all four corners regardless of `masked_corners_value`
    // above. Without also masking it here, the flush screen-edge corner
    // would show a rounded blur shape peeking out from behind the now
    // squared-off content/webview layers. Queued on the same main-thread
    // dispatcher right after set_effects(), so it runs after that call's
    // own internal main-thread work has actually added the tagged view.
    let window_for_blur_mask = window.clone();
    if let Err(e) = window.run_on_main_thread(move || {
        let ns_window_ptr = match window_for_blur_mask.ns_window() {
            Ok(ptr) => ptr,
            Err(e) => {
                log::warn!(
                    "Failed to get ns_window for blur mask: {} - window might not be fully initialized yet",
                    e
                );
                return;
            }
        };
        unsafe {
            let ns_window = ns_window_ptr as id;
            if ns_window.is_null() {
                log::warn!("NSWindow pointer is null - cannot mask blur view");
                return;
            }
            let content_view: id = msg_send![ns_window, contentView];
            if content_view.is_null() {
                log::warn!("Content view is null - cannot mask blur view");
                return;
            }
            // A null tagged view here is normal, not a failure: it just
            // means background blur is currently off (no NSVisualEffectView
            // was added by set_effects() above).
            let blur_view: id = msg_send![content_view, viewWithTag: BLUR_VIEW_TAG];
            if blur_view.is_null() {
                return;
            }
            let _: () = msg_send![blur_view, setWantsLayer: YES];
            let blur_layer: id = msg_send![blur_view, layer];
            if !blur_layer.is_null() {
                let _: () = msg_send![blur_layer, setMaskedCorners: masked_corners_value];
            }
        }
    }) {
        log::warn!("Failed to mask blur view corners: {}", e);
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
            let _: () = msg_send![webview_layer, setMaskedCorners: masked_corners_value];
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
    _edge_side: &str,
    _round_all_corners: bool,
) {
}

#[cfg(all(test, target_os = "macos"))]
mod masked_corners_tests {
    use super::masked_corners;

    const MIN_X_MIN_Y: u64 = 1 << 0;
    const MAX_X_MIN_Y: u64 = 1 << 1;
    const MIN_X_MAX_Y: u64 = 1 << 2;
    const MAX_X_MAX_Y: u64 = 1 << 3;
    const ALL_CORNERS: u64 = MIN_X_MIN_Y | MAX_X_MIN_Y | MIN_X_MAX_Y | MAX_X_MAX_Y;

    #[test]
    fn rounds_left_corners_when_docked_right() {
        assert_eq!(masked_corners("right", false), MIN_X_MIN_Y | MIN_X_MAX_Y);
    }

    #[test]
    fn rounds_right_corners_when_docked_left() {
        assert_eq!(masked_corners("left", false), MAX_X_MIN_Y | MAX_X_MAX_Y);
    }

    #[test]
    fn treats_unrecognized_edge_side_like_right() {
        // Matches src/lib/cornerRadius.js's fallback convention: only
        // "left" is matched explicitly, everything else (including an
        // unrecognized value) behaves like "right".
        assert_eq!(masked_corners("", false), MIN_X_MIN_Y | MIN_X_MAX_Y);
        assert_eq!(masked_corners("bogus", false), MIN_X_MIN_Y | MIN_X_MAX_Y);
    }

    #[test]
    fn rounds_all_corners_when_round_all_corners_is_true() {
        assert_eq!(masked_corners("right", true), ALL_CORNERS);
        assert_eq!(masked_corners("left", true), ALL_CORNERS);
    }
}
