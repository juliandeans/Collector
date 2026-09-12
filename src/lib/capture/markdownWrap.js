/**
 * Markdown formatting shortcuts for the Capture textarea (Cmd+B / Cmd+I).
 *
 * Wraps the selection with **bold** or *italic* markers, toggles them off
 * when the selection is already wrapped (either the markers sit directly
 * outside the selection or the selection itself carries them), and inserts
 * an empty marker pair at the cursor when nothing is selected. Pure
 * function — no DOM access.
 *
 * Known limitation (documented, like every simple editor): nested bold/
 * italic combinations (e.g. italic inside bold) can interleave the shared
 * `*` character; the toggle detects exact matches only.
 */

const STYLES = {
    bold: "**",
    italic: "*",
};

/**
 * @param {object} input
 * @param {string} input.content
 * @param {number} input.selectionStart
 * @param {number} input.selectionEnd
 * @param {string} input.key event.key ("b" / "i", case-insensitive)
 * @param {"bold" | "italic"} input.style
 * @param {boolean} [input.metaKey]
 * @param {boolean} [input.ctrlKey]
 * @param {boolean} [input.shiftKey]
 * @param {boolean} [input.altKey]
 * @returns {{ nextContent: string, nextSelectionStart: number, nextSelectionEnd: number } | null}
 */
export function computeMarkdownWrap({
    content,
    selectionStart,
    selectionEnd,
    key,
    style,
    metaKey = false,
    ctrlKey = false,
    shiftKey = false,
    altKey = false,
}) {
    if (!(metaKey || ctrlKey) || shiftKey || altKey) {
        return null;
    }
    const expected = STYLES[style] ? (style === "bold" ? "b" : "i") : null;
    if (!expected || String(key).toLowerCase() !== expected) {
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

    const marker = STYLES[style];

    // Selection must be within a single line.
    const startLineStart = content.lastIndexOf("\n", selectionStart - 1) + 1;
    const selectionEndLineStart =
        selectionEnd === 0 ? 0 : content.lastIndexOf("\n", selectionEnd - 1) + 1;
    if (selectionEndLineStart !== startLineStart) {
        return null;
    }

    // Unwrap: markers sit directly outside the selection.
    const before = content.slice(selectionStart - marker.length, selectionStart);
    const after = content.slice(selectionEnd, selectionEnd + marker.length);
    if (before === marker && after === marker) {
        return {
            nextContent:
                content.slice(0, selectionStart - marker.length) +
                content.slice(selectionStart, selectionEnd) +
                content.slice(selectionEnd + marker.length),
            nextSelectionStart: selectionStart - marker.length,
            nextSelectionEnd: selectionEnd - marker.length,
        };
    }

    // Unwrap: the selection itself carries the markers.
    const selected = content.slice(selectionStart, selectionEnd);
    if (
        selected.startsWith(marker) &&
        selected.endsWith(marker) &&
        selected.length >= marker.length * 2
    ) {
        return {
            nextContent:
                content.slice(0, selectionStart) +
                selected.slice(marker.length, selected.length - marker.length) +
                content.slice(selectionEnd),
            nextSelectionStart: selectionStart + marker.length,
            nextSelectionEnd: selectionEnd - marker.length,
        };
    }

    // Wrap (or insert an empty pair at the cursor).
    const nextContent =
        content.slice(0, selectionStart) +
        marker +
        content.slice(selectionStart, selectionEnd) +
        marker +
        content.slice(selectionEnd);
    return {
        nextContent,
        nextSelectionStart: selectionStart + marker.length,
        nextSelectionEnd: selectionEnd + marker.length,
    };
}
