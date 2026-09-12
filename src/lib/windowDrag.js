import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Attaches a capture-phase mousedown listener that enables window dragging when holding Command (Meta key).
 * Normal clicks and drags without Command are left intact for standard interactions like text selection.
 *
 * @param {HTMLElement|Window} [target=window] The element or window to listen on.
 * @returns {() => void} Cleanup function to remove the listener.
 */
export function setupCmdWindowDrag(target = typeof window !== "undefined" ? window : null) {
  if (!target) return () => {};

  const handleMouseDown = (event) => {
    // Only trigger for primary (left) click while Command (metaKey) is held.
    if (event.button !== 0 || !event.metaKey) {
      return;
    }

    // Allow Cmd-clicks on interactive elements (e.g. Cmd+click wikilink to open in new tab, buttons, or resize handle)
    if (event.target?.closest?.(".wikilink, a[href], button, input, select, .resize-handle")) {
      return;
    }

    event.preventDefault();
    try {
      getCurrentWindow().startDragging();
    } catch (error) {
      console.warn("Could not start window drag:", error);
    }
  };

  target.addEventListener("mousedown", handleMouseDown, { capture: true });

  return () => {
    target.removeEventListener("mousedown", handleMouseDown, { capture: true });
  };
}
