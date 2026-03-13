function hasPrimaryModifier(event) {
  return event.metaKey || event.ctrlKey;
}

function matchesShortcut(event, key) {
  return hasPrimaryModifier(event) && event.key.toLowerCase() === key;
}

export function setupListeners(callbacks = {}) {
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
      await callbacks.onOpenPalette?.();
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
      await callbacks.onOpenPalette?.();
      return;
    }

    if (matchesShortcut(event, "s")) {
      event.preventDefault();
      await callbacks.onSave?.();
      return;
    }

    if (hasPrimaryModifier(event) && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      const tabIndex = Number(event.key) - 1;
      if (callbacks.hasTabAtIndex?.(tabIndex)) {
        await callbacks.onActivateTab?.(tabIndex);
      }
      return;
    }

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
    }
  };

  window.addEventListener("keydown", handleGlobalKeydown);

  return () => {
    window.removeEventListener("keydown", handleGlobalKeydown);
  };
}
