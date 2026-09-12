/**
 * Pure function to insert text into textarea content at the cursor position
 * or replacing the current selection.
 * Returns { nextContent, nextCursorPosition } without directly mutating the DOM.
 *
 * @param {HTMLTextAreaElement|null} textarea
 * @param {string} currentContent
 * @param {string} text
 * @returns {{ nextContent: string, nextCursorPosition: number }}
 */
export function insertTextIntoTextarea(textarea, currentContent, text) {
    const rawContent = currentContent ?? "";
    const insertText = text ?? "";

    if (!textarea) {
        return {
            nextContent: rawContent + insertText,
            nextCursorPosition: rawContent.length + insertText.length,
        };
    }

    const start =
        textarea.selectionStart !== null && textarea.selectionStart !== undefined
            ? textarea.selectionStart
            : rawContent.length;
    const end =
        textarea.selectionEnd !== null && textarea.selectionEnd !== undefined
            ? textarea.selectionEnd
            : start;

    const before = rawContent.slice(0, start);
    const after = rawContent.slice(end);

    const nextContent = before + insertText + after;
    const nextCursorPosition = start + insertText.length;

    return {
        nextContent,
        nextCursorPosition,
    };
}
