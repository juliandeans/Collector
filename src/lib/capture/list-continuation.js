/**
 * List syntax and list key handling, shared by the Capture textarea and the
 * Reader editor.
 *
 * - Enter at a list item ("-", "*", "+" or "1." style) inserts a newline plus
 *   the next list prefix. An empty nested item moves up one level; an empty
 *   top-level item is removed instead of continued. Checkboxes ("- [ ]")
 *   restart as unchecked.
 * - Tab on a list item indents it by one level (a tab, or the vault's Obsidian
 *   space setting); numbered items restart at "1.". Shift+Tab outdents and
 *   continues the numbering of the list level above (Obsidian-style).
 *   Selection must stay within one line.
 * Pure functions — no DOM access.
 */

export const LIST_PATTERN = /^(\s*)([-*+]|\d+\.)([ \t]+)(.*)$/;
/** Obsidian's default indentation (`useTab: true`). */
export const DEFAULT_INDENT_UNIT = "\t";
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
    indentUnit = DEFAULT_INDENT_UNIT,
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

    // Empty item (plain or checkbox-only): a nested one moves up to its
    // parent's level (like Shift+Tab, numbering included), like Obsidian; a
    // top-level one ends the list by removing the prefix.
    const checkboxToken = CHECKBOX_PATTERN.exec(itemText)?.[0] ?? "";
    const withoutCheckbox = itemText.slice(checkboxToken.length);
    if (withoutCheckbox.trim() === "") {
        if (findParentIndent(content, lineStart, indentColumnWidth(indent)) !== null) {
            return computeListIndentation({
                content,
                selectionStart: lineEnd,
                selectionEnd: lineEnd,
                key: "Tab",
                shiftKey: true,
                indentUnit,
            });
        }
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
 * Tab / Shift+Tab indentation for list items. `indentUnit` is the vault's
 * Obsidian indentation unit (a tab or spaces); outdent lands on the parent
 * level when one exists and otherwise removes one trailing indentation unit.
 * The selection must be within a single line. Non-list lines are not handled.
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
    indentUnit = DEFAULT_INDENT_UNIT,
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

    const [, indent, marker] = match;

    if (!shiftKey) {
        // One level = the vault's Obsidian indent unit (tab or N spaces),
        // inserted at the line start.
        if (!isNumberedMarker(marker)) {
            const nextContent =
                content.slice(0, startLineStart) + indentUnit + content.slice(startLineStart);
            return { nextContent, nextCursorPosition: selectionStart + indentUnit.length };
        }
        // Indenting a numbered item restarts the nested list at "1.".
        const nextContent =
            content.slice(0, startLineStart) +
            indentUnit +
            indent +
            "1." +
            content.slice(startLineStart + indent.length + marker.length);
        const nextCaretInLine = mapCaretInLine(selectionStart - startLineStart, {
            oldIndentLength: indent.length,
            oldMarkerLength: marker.length,
            newIndentLength: indentUnit.length + indent.length,
            newMarkerLength: "1.".length,
        });
        return { nextContent, nextCursorPosition: startLineStart + nextCaretInLine };
    }

    // Outdent lands on the level of the parent item (the nearest list line
    // above with a smaller indent), like Obsidian. Without a parent in the
    // same list, remove one indent unit from the end of the indent.
    const parentIndent = findParentIndent(content, startLineStart, indentColumnWidth(indent));
    let nextIndent;
    if (parentIndent !== null) {
        nextIndent = parentIndent;
    } else {
        nextIndent = removeOneIndentUnit(indent, indentUnit);
        if (nextIndent === indent) {
            return null;
        }
    }

    // Numbered items continue the numbering of the level they land on.
    const nextMarker = isNumberedMarker(marker)
        ? `${findNumberAfterOutdent(content, startLineStart, indentColumnWidth(nextIndent))}.`
        : marker;

    const nextContent =
        content.slice(0, startLineStart) +
        nextIndent +
        nextMarker +
        content.slice(startLineStart + indent.length + marker.length);

    const nextCaretInLine = mapCaretInLine(selectionStart - startLineStart, {
        oldIndentLength: indent.length,
        oldMarkerLength: marker.length,
        newIndentLength: nextIndent.length,
        newMarkerLength: nextMarker.length,
    });

    return { nextContent, nextCursorPosition: startLineStart + nextCaretInLine };
}

/**
 * Maps a caret offset within a list line whose indent and marker were
 * replaced. Regions are classified by the ORIGINAL boundaries: inside the
 * indent the caret moves with the indent change (clamped to the new indent),
 * inside the marker it goes right after the new marker, after the marker it
 * keeps its place in the item text.
 */
function mapCaretInLine(
    caretInLine,
    { oldIndentLength, oldMarkerLength, newIndentLength, newMarkerLength },
) {
    const indentDelta = newIndentLength - oldIndentLength;
    if (caretInLine <= oldIndentLength) {
        return Math.min(Math.max(caretInLine + indentDelta, 0), newIndentLength);
    }
    if (caretInLine < oldIndentLength + oldMarkerLength) {
        return newIndentLength + newMarkerLength;
    }
    return caretInLine + indentDelta + (newMarkerLength - oldMarkerLength);
}

/**
 * Indent string of the nearest list line above `lineStart` whose indent is
 * narrower than `currentWidth` columns, or null when a non-list line comes
 * first (the item has no parent in this list).
 */
function findParentIndent(content, lineStart, currentWidth) {
    // lineStart - 1 drops the newline directly before the current line.
    const linesAbove = content.slice(0, Math.max(0, lineStart - 1)).split("\n");
    if (lineStart === 0) return null;
    for (let i = linesAbove.length - 1; i >= 0; i--) {
        const parsed = parseListLine(linesAbove[i]);
        if (!parsed) return null;
        if (indentColumnWidth(parsed.indent) < currentWidth) return parsed.indent;
    }
    return null;
}

/**
 * Removes one indentation level from the end of `indent`: a trailing tab,
 * else up to one unit of trailing spaces (a tab unit counts as 4 spaces).
 */
function removeOneIndentUnit(indent, indentUnit) {
    if (indent.endsWith("\t")) return indent.slice(0, -1);
    const maxSpaces = indentUnit === "\t" ? 4 : indentUnit.length;
    let end = indent.length;
    while (end > 0 && indent.length - end < maxSpaces && indent[end - 1] === " ") {
        end -= 1;
    }
    return indent.slice(0, end);
}

/**
 * Visual width of indentation in columns: a tab counts as 4, every other
 * whitespace character as 1.
 */
export function indentColumnWidth(indent) {
    let width = 0;
    for (const char of indent) {
        width += char === "\t" ? 4 : 1;
    }
    return width;
}

/**
 * Number for a numbered item after outdenting to `outdentedWidth` columns:
 * the next number after the numbered item directly above on that level.
 * Scanning upward stops at the first non-list line, at a shallower line, or
 * at a bullet on the target level — in all those cases the item restarts at 1.
 */
function findNumberAfterOutdent(content, lineStart, outdentedWidth) {
    // lineStart - 1 drops the newline directly before the current line.
    const linesAbove = content.slice(0, Math.max(0, lineStart - 1)).split("\n");
    for (let i = linesAbove.length - 1; i >= 0; i--) {
        const parsed = parseListLine(linesAbove[i]);
        if (!parsed) return 1;
        const width = indentColumnWidth(parsed.indent);
        if (width < outdentedWidth) return 1;
        if (width > outdentedWidth) continue;
        return isNumberedMarker(parsed.marker) ? parseInt(parsed.marker, 10) + 1 : 1;
    }
    return 1;
}

// Captures ALL whitespace after the box so the item text never starts with
// whitespace (the Reader trims item text when saving; `- [x]  a` must still
// round-trip).
const CHECKBOX_PARTS_PATTERN = /^\[( |x|X)\]([ \t]*)/;

/**
 * Splits a markdown list line into its raw parts so it can be rebuilt
 * byte-identically with formatListLine. Returns null for non-list lines.
 *
 * @param {string} line
 * @returns {{ indent: string, marker: string, spacing: string,
 *   checkbox: null | " " | "x" | "X", checkboxSpacing: string,
 *   text: string } | null}
 */
export function parseListLine(line) {
    if (typeof line !== "string") return null;
    const match = LIST_PATTERN.exec(line);
    if (!match) return null;

    const [, indent, marker, spacing, rest = ""] = match;
    const box = CHECKBOX_PARTS_PATTERN.exec(rest);
    if (!box) {
        return { indent, marker, spacing, checkbox: null, checkboxSpacing: "", text: rest };
    }
    return {
        indent,
        marker,
        spacing,
        checkbox: /** @type {" " | "x" | "X"} */ (box[1]),
        checkboxSpacing: box[2],
        text: rest.slice(box[0].length),
    };
}

/**
 * Inverse of parseListLine.
 */
export function formatListLine({
    indent = "",
    marker = "-",
    spacing = " ",
    checkbox = null,
    checkboxSpacing = "",
    text = "",
} = {}) {
    const box = checkbox === null ? "" : `[${checkbox}]${checkboxSpacing}`;
    return `${indent}${marker}${spacing}${box}${text}`;
}
