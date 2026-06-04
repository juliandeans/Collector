function hasPrimaryModifier(event) {
  return event.metaKey || event.ctrlKey;
}

function matchesShortcut(event, key) {
  return hasPrimaryModifier(event) && event.key.toLowerCase() === key;
}

/**
 * Checks if a KeyboardEvent matches a shortcut string like "Cmd+Shift+O".
 * Handles Cmd, Ctrl, Shift, Alt modifiers and most standard keys.
 */
function matchesCustomShortcut(event, shortcutString) {
  if (!shortcutString) return false;

  const parts = shortcutString.split("+").map((p) => p.trim());
  const hasCmd = parts.includes("Cmd") || parts.includes("Command");
  const hasCtrl = parts.includes("Ctrl") || parts.includes("Control");
  const hasShift = parts.includes("Shift");
  const hasAlt =
    parts.includes("Alt") || parts.includes("Option") || parts.includes("Opt");

  const key = parts.find(
    (p) =>
      ![
        "Cmd",
        "Command",
        "Ctrl",
        "Control",
        "Shift",
        "Alt",
        "Option",
        "Opt",
      ].includes(p),
  );

  if (!key) return false;

  const modifiersMatch =
    (event.metaKey === hasCmd || event.ctrlKey === hasCmd) &&
    event.ctrlKey === hasCtrl &&
    event.shiftKey === hasShift &&
    event.altKey === hasAlt;

  const keyMatches = event.key.toLowerCase() === key.toLowerCase();

  return modifiersMatch && keyMatches;
}

/**
 * Sets up global keydown listeners for the reader window.
 *
 * @param {object} callbacks - Named callbacks for hardcoded shortcuts.
 * @param {object[]} customShortcuts - Array of configurable shortcut bindings.
 *   Each entry: { getShortcut: () => string, onMatch: () => void }
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

    if (matchesShortcut(event, "k")) {
      event.preventDefault();
      callbacks.onOpenPalette?.();
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

    if (matchesShortcut(event, "p")) {
      event.preventDefault();
      callbacks.onOpenPalette?.();
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
    for (const { getShortcut, onMatch } of customShortcuts) {
      const shortcutStr = getShortcut?.();
      if (shortcutStr && matchesCustomShortcut(event, shortcutStr)) {
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
