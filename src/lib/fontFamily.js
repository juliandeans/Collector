// Font family setting helpers shared by the Capture window, Reader and Settings.

export const SYSTEM_FONT_VALUE = "-apple-system, BlinkMacSystemFont, SF Pro Display";

const SYSTEM_FONT_CSS = '-apple-system, BlinkMacSystemFont, "SF Pro Display", sans-serif';
const FALLBACK_TAIL = "-apple-system, BlinkMacSystemFont, sans-serif";

// Characters that could end the CSS value or the style attribute,
// or open a comment that swallows the declarations after it (/*).
const UNSAFE_CHARS = /["'\\;{}<>\r\n/*]/g;

// Shown when the installed families cannot be read (command failed, browser preview).
export const FALLBACK_FONT_FAMILIES = [
  "SF Pro",
  "SF Pro Display",
  "SF Pro Text",
  "SF Mono",
  "Avenir Next",
  "Helvetica Neue",
  "Helvetica",
  "Arial",
  "Georgia",
  "Times New Roman",
  "Courier New",
  "Monaco",
  "Menlo",
];

/**
 * Turn the stored font_family setting into a safe CSS font-family value.
 * A single family gets the system font as fallback, so a deleted font
 * shows the system font instead of the browser default.
 * @param {unknown} value
 * @returns {string}
 */
export function fontFamilyCss(value) {
  const cleaned =
    typeof value === "string"
      ? value.replace(UNSAFE_CHARS, "").replace(/[\s,]+$/, "").trim()
      : "";
  if (!cleaned || cleaned === SYSTEM_FONT_VALUE) return SYSTEM_FONT_CSS;
  if (cleaned.includes(",")) return cleaned;
  return `"${cleaned}", ${FALLBACK_TAIL}`;
}

/**
 * Build the Font Family select options.
 * @param {unknown} installed family names from list_font_families
 * @param {unknown} current the stored font_family value
 * @returns {Array<{ value: string, label: string }>}
 */
export function buildFontOptions(installed, current) {
  const families =
    Array.isArray(installed) && installed.length > 0 ? installed : FALLBACK_FONT_FAMILIES;
  const options = [{ value: SYSTEM_FONT_VALUE, label: "System (Default)" }];
  const saved = typeof current === "string" ? current : "";
  if (saved && saved !== SYSTEM_FONT_VALUE && !families.includes(saved)) {
    options.push({ value: saved, label: `${saved} (not installed)` });
  }
  for (const name of families) {
    options.push({ value: name, label: name });
  }
  return options;
}
