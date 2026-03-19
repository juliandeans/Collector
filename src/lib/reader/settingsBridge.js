import { normalizePinnedNotes } from "./tabState.js";

const READER_FILTER_KEYS = [
  "reader_hide_frontmatter",
  "reader_hide_dataview",
  "reader_hide_obsidian_comments",
  "reader_hide_inline_fields",
  "reader_hide_html",
  "reader_hide_callouts",
];

export function computeBrightnessFilter(brightness = 0) {
  if (brightness === 0) return "";

  if (brightness > 0) {
    const brightnessValue = 1 + (brightness / 100) * 0.6;
    const contrastValue = 1 - (brightness / 100) * 0.25;
    return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
  }

  const brightnessValue = 1 + (brightness / 100) * 0.7;
  const contrastValue = 1 + (-brightness / 100) * 0.3;
  return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
}

export function applyColorSettings(settings = {}) {
  if (typeof document === "undefined") return;

  const root = document.documentElement;
  root.style.setProperty(
    "--reader-text-secondary",
    "color-mix(in srgb, var(--app-text-color) 60%, transparent)",
  );
  root.style.setProperty("--accent-color", settings.accent_color ?? "#8b5cf6");
  root.style.setProperty(
    "--internal-link-color",
    settings.internal_link_color ?? "#a78bfa",
  );
  root.style.setProperty(
    "--external-link-color",
    settings.external_link_color ?? "#60a5fa",
  );
}

export function applySettings(currentSettings = {}, settings = {}) {
  return {
    ...currentSettings,
    vault_name: settings.vault_name ?? currentSettings.vault_name,
    vault_path: settings.vault_path ?? currentSettings.vault_path,
    background_color:
      settings.background_color ?? currentSettings.background_color,
    font_family: settings.font_family ?? currentSettings.font_family,
    font_size: settings.font_size ?? currentSettings.font_size,
    border_radius: settings.border_radius ?? currentSettings.border_radius,
    window_transparency:
      settings.window_transparency ?? currentSettings.window_transparency,
    window_blur: settings.window_blur ?? currentSettings.window_blur,
    window_saturation:
      settings.window_saturation ?? currentSettings.window_saturation,
    window_brightness:
      settings.window_brightness ?? currentSettings.window_brightness,
    text_color: settings.text_color ?? currentSettings.text_color,
    accent_color: settings.accent_color ?? currentSettings.accent_color,
    internal_link_color:
      settings.internal_link_color ?? currentSettings.internal_link_color,
    external_link_color:
      settings.external_link_color ?? currentSettings.external_link_color,
    pinned_notes: normalizePinnedNotes(
      settings.pinned_notes ?? currentSettings.pinned_notes,
    ),
    reader_hide_frontmatter:
      settings.reader_hide_frontmatter ??
      currentSettings.reader_hide_frontmatter,
    reader_hide_dataview:
      settings.reader_hide_dataview ?? currentSettings.reader_hide_dataview,
    reader_hide_obsidian_comments:
      settings.reader_hide_obsidian_comments ??
      currentSettings.reader_hide_obsidian_comments,
    reader_hide_inline_fields:
      settings.reader_hide_inline_fields ??
      currentSettings.reader_hide_inline_fields,
    reader_hide_html: settings.reader_hide_html ?? currentSettings.reader_hide_html,
    reader_hide_callouts:
      settings.reader_hide_callouts ?? currentSettings.reader_hide_callouts,
  };
}

export function getReaderFilterSettings(settings = {}) {
  return {
    reader_hide_frontmatter: settings.reader_hide_frontmatter,
    reader_hide_dataview: settings.reader_hide_dataview,
    reader_hide_obsidian_comments: settings.reader_hide_obsidian_comments,
    reader_hide_inline_fields: settings.reader_hide_inline_fields,
    reader_hide_html: settings.reader_hide_html,
    reader_hide_callouts: settings.reader_hide_callouts,
  };
}

export function haveReaderFilterChanges(
  previousFilters = {},
  nextFilters = {},
) {
  return READER_FILTER_KEYS.some(
    (key) => previousFilters[key] !== nextFilters[key],
  );
}
