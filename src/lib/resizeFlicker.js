import { getCurrentWindow } from "@tauri-apps/api/window";

const RESIZE_SETTLE_MS = 200;

// backdrop-filter and box-shadow on a borderless, transparent window are
// expensive to keep recomputing continuously, which is visible as flicker
// during a live native resize drag. Toggling them off for the duration of
// an active resize (restored once resize events stop arriving for
// RESIZE_SETTLE_MS) is the standard mitigation for this class of problem.
//
// onResized() also fires for the app's own programmatic set_size() calls
// (window show, settings save, snapping back to a remembered size) — not
// just user-driven drags. Reacting to every onResized event would flash
// backdrop-filter/box-shadow off on those too, working against the fix's
// own goal. So this only reacts to onResized while "armed" by a call to
// armResize() (wired to the resize handle's own mousedown), and disarms
// itself once the resize settles.
//
// armResize() itself starts the same settle countdown it uses between
// resize events — a mousedown that never produces a Resized event (a
// click-without-drag on the small handle, or startResizeDragging silently
// no-op'ing) still clears itself out after RESIZE_SETTLE_MS instead of
// leaving backdrop-filter/box-shadow suppressed for the rest of the
// session. A genuine drag's first onResized event arrives well within
// that window and re-schedules the same timer, so it never actually
// expires mid-resize.
//
// Shared between App.svelte (Capture) and Reader.svelte.
export async function setupResizeFlicker(setResizing) {
  let settleTimer;
  let armed = false;

  function scheduleSettle() {
    clearTimeout(settleTimer);
    settleTimer = setTimeout(() => {
      armed = false;
      setResizing(false);
    }, RESIZE_SETTLE_MS);
  }

  const unlisten = await getCurrentWindow().onResized(() => {
    if (!armed) return;
    setResizing(true);
    scheduleSettle();
  });

  function armResize() {
    armed = true;
    setResizing(true);
    scheduleSettle();
  }

  function cleanup() {
    clearTimeout(settleTimer);
    unlisten();
  }

  return { armResize, cleanup };
}
