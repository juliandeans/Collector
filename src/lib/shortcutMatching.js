const MODIFIER_TOKENS = new Set([
  "cmd",
  "command",
  "commandorcontrol",
  "ctrl",
  "control",
  "shift",
  "alt",
  "option",
  "opt",
]);

const KEY_ALIASES = new Map([
  ["arrowup", "up"],
  ["arrowdown", "down"],
  ["arrowleft", "left"],
  ["arrowright", "right"],
  [" ", "space"],
  ["spacebar", "space"],
  ["esc", "escape"],
]);

function normalizeToken(value = "") {
  const rawToken = String(value).toLowerCase();
  if (KEY_ALIASES.has(rawToken)) return KEY_ALIASES.get(rawToken);

  const token = rawToken.trim();
  return KEY_ALIASES.get(token) ?? token;
}

function parseShortcut(shortcutString = "") {
  const parts = String(shortcutString)
    .split("+")
    .map((part) => part.trim())
    .filter(Boolean);

  if (parts.length < 2) return null;

  const shortcut = {
    hasCmd: false,
    hasCtrl: false,
    hasPrimary: false,
    hasShift: false,
    hasAlt: false,
    key: "",
  };

  for (const part of parts) {
    const token = normalizeToken(part);

    if (MODIFIER_TOKENS.has(token)) {
      if (token === "cmd" || token === "command") shortcut.hasCmd = true;
      if (token === "ctrl" || token === "control") shortcut.hasCtrl = true;
      if (token === "commandorcontrol") shortcut.hasPrimary = true;
      if (token === "shift") shortcut.hasShift = true;
      if (token === "alt" || token === "option" || token === "opt") {
        shortcut.hasAlt = true;
      }
      continue;
    }

    if (shortcut.key) return null;
    shortcut.key = token;
  }

  if (!shortcut.key) return null;
  if (
    !shortcut.hasCmd &&
    !shortcut.hasCtrl &&
    !shortcut.hasPrimary &&
    !shortcut.hasShift &&
    !shortcut.hasAlt
  ) {
    return null;
  }

  return shortcut;
}

function modifiersMatch(event, shortcut) {
  if (shortcut.hasPrimary) {
    return (
      (event.metaKey || event.ctrlKey) &&
      event.shiftKey === shortcut.hasShift &&
      event.altKey === shortcut.hasAlt
    );
  }

  return (
    event.metaKey === shortcut.hasCmd &&
    event.ctrlKey === shortcut.hasCtrl &&
    event.shiftKey === shortcut.hasShift &&
    event.altKey === shortcut.hasAlt
  );
}

export function matchesShortcut(event, shortcutString) {
  const shortcut = parseShortcut(shortcutString);
  if (!shortcut) return false;

  return (
    modifiersMatch(event, shortcut) &&
    normalizeToken(event.key) === shortcut.key
  );
}
