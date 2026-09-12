import { getCurrentWindow } from "@tauri-apps/api/window";

/**
 * Checks whether we are executing in a Tauri runtime context.
 * In a pure browser or Vite dev preview without Tauri, window.__TAURI_INTERNALS__ is undefined.
 */
export function isTauriContext() {
   return (
      typeof window !== "undefined" &&
      Boolean(window.__TAURI_INTERNALS__ || window.__TAURI__)
   );
}

/**
 * Checks whether an element should be exempt from initiating window drag.
 * Excludes interactive links, buttons, form controls, and resize handles.
 *
 * NOTE (Design Decision): `textarea` and `[contenteditable]` are INTENTIONALLY NOT exempt.
 * Collector's floating windows (Capture and Reader) consist almost entirely of a textarea
 * or contenteditable editor without traditional window frames. Users must be able to hold
 * Command and drag anywhere across the textarea or editor to move the window. Normal clicks
 * and drags without Command remain untouched and perform standard text selection.
 *
 * @param {EventTarget|null} target
 * @returns {boolean}
 */
export function isDragExemptElement(target) {
   if (!target || typeof target.closest !== "function") return false;
   return Boolean(
      target.closest(
         ".wikilink, a[href], button, input, select, .resize-handle",
      ),
   );
}

/**
 * Attaches a capture-phase mousedown listener that enables window dragging when holding Command (Meta key).
 * Normal clicks and drags without Command are left intact for standard interactions like text selection.
 *
 * @param {HTMLElement|Window} [target=window] The element or window to listen on.
 * @returns {() => void} Cleanup function to remove the listener.
 */
export function setupCmdWindowDrag(
   target = typeof window === "undefined" ? null : window,
) {
   if (!target || !isTauriContext()) {
      return () => {};
   }

   const handleMouseDown = (event) => {
      // Only trigger for primary (left) click while Command (metaKey) is held.
      if (event.button !== 0 || !event.metaKey) {
         return;
      }

      // Allow Cmd-clicks on interactive elements (e.g. Cmd+click wikilink to open in new tab, buttons, or resize handle)
      if (isDragExemptElement(event.target)) {
         return;
      }

      event.preventDefault();
      try {
         const promise = getCurrentWindow().startDragging();
         if (promise && typeof promise.catch === "function") {
            promise.catch((error) => {
               console.warn("Could not start window drag:", error);
            });
         }
      } catch (error) {
         console.warn("Could not start window drag:", error);
      }
   };

   target.addEventListener("mousedown", handleMouseDown, { capture: true });

   return () => {
      target.removeEventListener("mousedown", handleMouseDown, {
         capture: true,
      });
   };
}
