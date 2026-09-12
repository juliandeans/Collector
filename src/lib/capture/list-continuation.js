/**
 * List continuation for the Capture textarea.
 *
 * Pressing Enter at the end of a list item ("-", "*", "+" or "1." style)
 * inserts a newline plus the next list prefix at the cursor. Empty list
 * items ("just a marker") are removed instead of continued. Checkboxes
 * ("- [ ]") restart as unchecked. Pure function — no DOM access.
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
