/**
 * List key handling for the Capture textarea.
 *
 * - Enter at a list item ("-", "*", "+" or "1." style) inserts a newline plus
 *   the next list prefix. Empty list items ("just a marker") are removed
 *   instead of continued. Checkboxes ("- [ ]") restart as unchecked.
 * - Tab on a list item indents it by the marker width (nested item);
 *   Shift+Tab outdents. Selection must stay within one line.
 * Pure functions — no DOM access.
 */

const LIST_PATTERN = /^(\s*)([-*+]|\d+\.)([ \t]+)(.*)$/;
const CHECKBOX_PATTERN = /^\[( |x|X)\]\s?/;

/**
 * @param {object} input
 * @param {string} input.content Current textarea content.
 * @param {number} input.selectionStart
 * @param {number} input.selectionEnd
 * @param {string} input.key event.key
 * @param {boolean} [input.metaKey]
 * @param {boolean} [input.ctrlKey]
 * @param {boolean} [input.shiftKey]
 * @param {boolean} [input.altKey]
 * @returns {{ nextContent: string, nextCursorPosition: number } | null}
 *   null when the event should be handled normally.
 */
export function computeListContinuation({
    content,
    selectionStart,
    selectionEnd,
    key,
    metaKey = false,
    ctrlKey = false,
    shiftKey = false,
    altKey = false,
}) {
    if (key !== "Enter" || metaKey || ctrlKey || shiftKey || altKey) {
        return null;
    }
    if (selectionStart !== selectionEnd) {
        return null;
    }
    if (
        typeof content !== "string" ||
        !Number.isInteger(selectionStart) ||
        selectionStart < 0 ||
        selectionStart > content.length
    ) {
        return null;
    }

    const lineStart = content.lastIndexOf("\n", selectionStart - 1) + 1;
    const newlineIndex = content.indexOf("\n", selectionStart);
    const lineEnd = newlineIndex === -1 ? content.length : newlineIndex;
    const line = content.slice(lineStart, lineEnd);

    const match = LIST_PATTERN.exec(line);
    if (!match) {
        return null;
    }

    const [, indent, marker, spaces = "", itemText = ""] = match;

    // Empty item (plain or checkbox-only): remove the list prefix instead of
    // spawning another one.
    const checkboxToken = CHECKBOX_PATTERN.exec(itemText)?.[0] ?? "";
    const withoutCheckbox = itemText.slice(checkboxToken.length);
    if (withoutCheckbox.trim() === "") {
        const prefixLength =
            indent.length + marker.length + spaces.length + checkboxToken.length;
        const nextContent =
            content.slice(0, lineStart) + content.slice(lineStart + prefixLength);
        return { nextContent, nextCursorPosition: lineStart };
    }

    const nextMarker = isNumberedMarker(marker) ? `${parseInt(marker, 10) + 1}.` : marker;
    const separator = spaces;
    const checkboxPrefix = getCheckboxPrefix(itemText);
    const nextPrefix =
        indent + nextMarker + separator + (checkboxPrefix !== null ? `${checkboxPrefix} ` : "");

    const insertion = "\n" + nextPrefix;
    const nextContent =
        content.slice(0, selectionStart) + insertion + content.slice(selectionStart);
    return { nextContent, nextCursorPosition: selectionStart + insertion.length };
}

function isNumberedMarker(marker) {
    return /^\d+\.$/.test(marker);
}

/**
 * Returns "[ ]" when the item starts with a checkbox, else null.
 * Restarted checkboxes are always unchecked, like every markdown editor.
 */
function getCheckboxPrefix(itemText) {
    return CHECKBOX_PATTERN.test(itemText) ? "[ ]" : null;
}

/**
 * Tab / Shift+Tab indentation for list items. Indent inserts one level of
 * spaces equal to the current marker width (so the item aligns under its
 * content); outdent removes up to that many leading spaces. The selection
 * must be within a single line. Non-list lines are not handled.
 *
 * @returns {{ nextContent: string, nextCursorPosition: number } | null}
 */
export function computeListIndentation({
    content,
    selectionStart,
    selectionEnd,
    key,
    shiftKey = false,
    metaKey = false,
    ctrlKey = false,
    altKey = false,
}) {
    if (key !== "Tab" || metaKey || ctrlKey || altKey) {
        return null;
    }
    if (
        typeof content !== "string" ||
        !Number.isInteger(selectionStart) ||
        !Number.isInteger(selectionEnd) ||
        selectionStart < 0 ||
        selectionStart > selectionEnd ||
        selectionEnd > content.length
    ) {
        return null;
    }

    const startLineStart = content.lastIndexOf("\n", selectionStart - 1) + 1;
    let selectionEndLineStart = content.lastIndexOf("\n", selectionEnd - 1) + 1;
    if (selectionEnd === 0) selectionEndLineStart = 0;
    if (selectionEndLineStart !== startLineStart) {
        return null; // multi-line selection: not handled
    }
    const newlineIndex = content.indexOf("\n", selectionEnd);
    const lineEnd = newlineIndex === -1 ? content.length : newlineIndex;
    const line = content.slice(startLineStart, lineEnd);

    const match = LIST_PATTERN.exec(line);
    if (!match) {
        return null;
    }

    const [, indent, marker, spaces = ""] = match;
    // One indentation step aligns the nested marker under the current
    // item's content column: marker + following spaces (indent excluded,
    // else nested levels double).
    const step = marker.length + spaces.length;

    if (!shiftKey) {
        const insertion = " ".repeat(step);
        const nextContent =
            content.slice(0, startLineStart) + insertion + content.slice(startLineStart);
        return { nextContent, nextCursorPosition: selectionStart + step };
    }

    const leadingSpaces = /^ */.exec(line)[0].length;
    const removable = Math.min(step, leadingSpaces);
    if (removable === 0) {
        return null;
    }
    const nextContent =
        content.slice(0, startLineStart) + content.slice(startLineStart + removable);
    return {
        nextContent,
        nextCursorPosition: Math.max(startLineStart, selectionStart - removable),
    };
}
