/**
 * Utility functions for Quick Capture
 */

/**
 * Format a timestamp for display
 * @param {Date} date 
 * @param {string} format - Format string (HH:mm, HH:mm:ss, etc.)
 * @returns {string}
 */
export function formatTimestamp(date = new Date(), format = 'HH:mm') {
  const pad = (n) => n.toString().padStart(2, '0');
  
  const replacements = {
    'YYYY': date.getFullYear().toString(),
    'MM': pad(date.getMonth() + 1),
    'DD': pad(date.getDate()),
    'HH': pad(date.getHours()),
    'mm': pad(date.getMinutes()),
    'ss': pad(date.getSeconds()),
  };
  
  let result = format;
  for (const [key, value] of Object.entries(replacements)) {
    result = result.replace(new RegExp(key, 'g'), value);
  }
  
  return result;
}

/**
 * Build a markdown image link (Obsidian wikilink style)
 * @param {string} filename 
 * @returns {string}
 */
export function buildMarkdownImageLink(filename) {
  return `![[${filename}]]`;
}

/**
 * Build a markdown link
 * @param {string} text 
 * @param {string} url 
 * @returns {string}
 */
export function buildMarkdownLink(text, url) {
  return `[${text}](${url})`;
}

/**
 * Check if a file is a supported image type
 * @param {string} filename 
 * @returns {boolean}
 */
export function isSupportedImage(filename) {
  const ext = filename.split('.').pop()?.toLowerCase();
  return ['png', 'jpg', 'jpeg', 'webp', 'gif'].includes(ext);
}

/**
 * Debounce function
 * @param {Function} fn 
 * @param {number} delay 
 * @returns {Function}
 */
export function debounce(fn, delay = 300) {
  let timeoutId;
  return (...args) => {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn.apply(this, args), delay);
  };
}

/**
 * Apply CSS variables to document root
 * @param {Object} vars - Object with CSS variable names and values
 */
export function applyCssVars(vars) {
  const root = document.documentElement;
  for (const [key, value] of Object.entries(vars)) {
    root.style.setProperty(key, value);
  }
}

/**
 * Validate a hex color
 * @param {string} color 
 * @returns {boolean}
 */
export function isValidHexColor(color) {
  return /^#[0-9A-Fa-f]{6}$/.test(color);
}

/**
 * Get system fonts available on macOS
 * @returns {string[]}
 */
export function getSystemFonts() {
  return [
    'SF Pro',
    'SF Pro Display',
    'SF Pro Text',
    'SF Mono',
    'Avenir Next',
    'Helvetica Neue',
    'Helvetica',
    'Arial',
    'Georgia',
    'Times New Roman',
    'Courier New',
    'Monaco',
    'Menlo',
  ];
}

/**
 * Parse shortcut string for display
 * @param {string} shortcut 
 * @returns {string}
 */
export function formatShortcut(shortcut) {
  return shortcut
    .replace('CommandOrControl', '⌘')
    .replace('Command', '⌘')
    .replace('Cmd', '⌘')
    .replace('Control', '⌃')
    .replace('Ctrl', '⌃')
    .replace('Shift', '⇧')
    .replace('Alt', '⌥')
    .replace('Option', '⌥')
    .replace(/\+/g, '');
}

/**
 * Sleep for a given number of milliseconds
 * @param {number} ms 
 * @returns {Promise<void>}
 */
export function sleep(ms) {
  return new Promise(resolve => setTimeout(resolve, ms));
}
