import {
  computeListContinuation,
  computeListIndentation,
} from "../capture/list-continuation.js";

/**
 * List keys for the Reader editor: Enter / Tab / Shift+Tab on the active
 * raw-mode block, using exactly the Capture window's rules. The active block
 * holds a single raw markdown line, so the Capture functions (which work on
 * text + caret offsets) apply to it directly.
 */

export function splitLineEdit({ nextContent, nextCursorPosition }) {
  const newline = nextContent.indexOf("\n");
  if (newline === -1) {
    return { lines: [nextContent], caretLine: 0, caretOffset: nextCursorPosition };
  }
  return {
    lines: [nextContent.slice(0, newline), nextContent.slice(newline + 1)],
    caretLine: 1,
    caretOffset: nextCursorPosition - newline - 1,
  };
}

export function computeListKeyEdit({
  lineText,
  selectionStart,
  selectionEnd,
  key,
  shiftKey = false,
  metaKey = false,
  ctrlKey = false,
  altKey = false,
  isComposing = false,
}) {
  if (isComposing) return null;
  if (typeof lineText !== "string" || lineText.includes("\n")) return null;

  const input = {
    content: lineText,
    selectionStart,
    selectionEnd,
    key,
    shiftKey,
    metaKey,
    ctrlKey,
    altKey,
  };

  let result = null;
  if (key === "Enter") result = computeListContinuation(input);
  else if (key === "Tab") result = computeListIndentation(input);

  return result ? splitLineEdit(result) : null;
}

function selectionOffsetsIn(block) {
  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0) return null;

  const range = selection.getRangeAt(0);
  if (!block.contains(range.startContainer) || !block.contains(range.endContainer)) {
    return null;
  }

  const measure = document.createRange();
  measure.selectNodeContents(block);
  measure.setEnd(range.startContainer, range.startOffset);
  const start = measure.toString().length;
  measure.setEnd(range.endContainer, range.endOffset);
  const end = measure.toString().length;
  return { start, end };
}

function setBlockText(block, text) {
  if (text) {
    block.textContent = text;
  } else {
    // Empty paragraphs need a <br> to keep their height, like "<p><br></p>".
    block.innerHTML = "<br>";
  }
}

function placeCaret(block, offset) {
  const selection = window.getSelection();
  if (!selection) return;

  const range = document.createRange();
  const textNode = block.firstChild;
  if (textNode && textNode.nodeType === Node.TEXT_NODE) {
    range.setStart(textNode, Math.min(offset, textNode.textContent.length));
  } else {
    range.setStart(block, 0);
  }
  range.collapse(true);
  selection.removeAllRanges();
  selection.addRange(range);
}

export function handleListKey(event, { editorRef, blockManager }) {
  if (event.key !== "Enter" && event.key !== "Tab") return false;

  const block = blockManager.getActiveParagraph();
  if (!block || !editorRef?.contains(block) || !block.classList.contains("raw-mode")) {
    return false;
  }

  const offsets = selectionOffsetsIn(block);
  if (!offsets) return false;

  const edit = computeListKeyEdit({
    lineText: block.textContent ?? "",
    selectionStart: offsets.start,
    selectionEnd: offsets.end,
    key: event.key,
    shiftKey: event.shiftKey,
    metaKey: event.metaKey,
    ctrlKey: event.ctrlKey,
    altKey: event.altKey,
    isComposing: event.isComposing,
  });
  if (!edit) return false;

  event.preventDefault();
  setBlockText(block, edit.lines[0]);

  if (edit.lines.length === 1) {
    placeCaret(block, edit.caretOffset);
    return true;
  }

  const next = document.createElement("p");
  next.className = "raw-mode";
  setBlockText(next, edit.lines[1]);
  block.after(next);
  // Re-renders the previous block as a list item and makes `next` active.
  blockManager.switchActiveBlock(next);
  placeCaret(next, edit.caretOffset);
  return true;
}
