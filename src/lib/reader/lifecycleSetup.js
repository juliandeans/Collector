import { matchesShortcut as matchesCustomShortcut } from "../shortcutMatching.js";

function hasPrimaryModifier(event) {
  return event.metaKey || event.ctrlKey;
}

function matchesShortcut(event, key) {
  return hasPrimaryModifier(event) && event.key.toLowerCase() === key;
}

/**
 * Sets up global keydown listeners for the reader window.
 *
 * @param {object} callbacks - Named callbacks for hardcoded shortcuts.
 * @param {object[]} customShortcuts - Array of configurable shortcut bindings.
 *   Each entry: { getShortcut: () => string, onMatch: () => void, shouldHandle?: () => boolean }
 *   The getter is called at event time so it always reads the latest value
 *   (e.g. from appSettings), even if settings change while the window is open.
 *   Hardcoded shortcuts are checked first and take precedence.
 */
export function setupListeners(callbacks = {}, customShortcuts = []) {
  const handleGlobalKeydown = async (event) => {
    if (callbacks.isTabContextMenuOpen?.() && event.key === "Escape") {
      event.preventDefault();
      await callbacks.onCloseTabContextMenu?.();
      return;
    }

    if (matchesShortcut(event, "w")) {
      event.preventDefault();
      await callbacks.onCloseActiveTab?.();
      return;
    }

    if (matchesShortcut(event, "f")) {
      event.preventDefault();
      if (callbacks.isSearchOpen?.()) {
        callbacks.onFocusSearch?.();
      } else {
        callbacks.onSearch?.();
      }
      return;
    }

    if (matchesShortcut(event, "s")) {
      event.preventDefault();
      await callbacks.onSave?.();
      return;
    }

    // Tab number shortcuts
    if (hasPrimaryModifier(event) && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      const tabIndex = Number(event.key) - 1;
      if (callbacks.hasTabAtIndex?.(tabIndex)) {
        await callbacks.onActivateTab?.(tabIndex);
      }
      return;
    }

    // Escape: progressively close overlays, then the window
    if (event.key === "Escape") {
      event.preventDefault();
      if (callbacks.isAutocompleteOpen?.()) {
        callbacks.onCloseAutocomplete?.();
        return;
      }
      if (callbacks.isSearchOpen?.()) {
        callbacks.onCloseSearch?.();
        return;
      }
      if (callbacks.isPaletteOpen?.()) {
        callbacks.onClosePalette?.();
      } else {
        await callbacks.onCloseReader?.();
      }
      return;
    }

    // --- Custom configurable shortcuts (checked after hardcoded ones) ---
    for (const { getShortcut, onMatch, shouldHandle } of customShortcuts) {
      const shortcutStr = getShortcut?.();
      if (shortcutStr && matchesCustomShortcut(event, shortcutStr)) {
        if (shouldHandle && !shouldHandle()) continue;
        event.preventDefault();
        await onMatch?.();
        return;
      }
    }
  };

  window.addEventListener("keydown", handleGlobalKeydown);

  return () => {
    window.removeEventListener("keydown", handleGlobalKeydown);
  };
}
