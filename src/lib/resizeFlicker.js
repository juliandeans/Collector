import { getCurrentWindow } from "@tauri-apps/api/window";

const RESIZE_SETTLE_MS = 200;

// Experimental effect suppression for the JavaScript resize fallback.
// macOS resize borders are handled before DOM dispatch by window_resize.rs,
// leaving these effects unchanged. Transparency alone was not established
// as the cause of flicker; native/BTT frame updates were smooth in live tests.
// This timer measures gaps between events, not whether the mouse is held:
// a pause longer than RESIZE_SETTLE_MS disarms it until the next armResize().
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
