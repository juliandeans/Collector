import {
  LIST_PATTERN,
  computeListContinuation,
  computeListIndentation,
} from "../capture/list-continuation.js";
import { elementToMarkdownLine } from "./editorSerialization.js";

/**
 * List keys for the Reader editor: Enter / Tab / Shift+Tab on the active
 * raw-mode block, using exactly the Capture window's rules. The active block
 * holds a single raw markdown line, so the Capture functions (which work on
 * text + caret offsets) apply to it directly. For Tab the markdown lines of
 * the blocks above are prepended so list renumbering can see the outer list.
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
  precedingLines = [],
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

  // Run the capture functions on a temporary document: the active line plus
  // the markdown lines above it, so outdenting can continue the numbering of
  // the outer list. The prefix itself never changes, so it is stripped again
  // (content and caret offsets) before splitting into per-block lines.
  const prefix = precedingLines.length > 0 ? `${precedingLines.join("\n")}\n` : "";

  const input = {
    content: prefix + lineText,
    selectionStart: selectionStart + prefix.length,
    selectionEnd: selectionEnd + prefix.length,
    key,
    shiftKey,
    metaKey,
    ctrlKey,
    altKey,
  };

  let result = null;
  if (key === "Enter") result = computeListContinuation(input);
  else if (key === "Tab") result = computeListIndentation(input);
  if (!result) return null;

  return splitLineEdit({
    nextContent: result.nextContent.slice(prefix.length),
    nextCursorPosition: result.nextCursorPosition - prefix.length,
  });
}

/**
 * Markdown lines of the list blocks directly above `block`, top-to-bottom,
 * for list renumbering.
 */
function collectPrecedingLines(block) {
  const lines = [];
  let sibling = block.previousElementSibling;
  while (sibling) {
    const line = elementToMarkdownLine(sibling);
    // The renumbering scan stops at the first non-list line anyway, so the
    // walk ends at the list's boundary (blank line, paragraph, code block).
    if (line === null || !LIST_PATTERN.test(line)) break;
    lines.push(line);
    sibling = sibling.previousElementSibling;
  }
  return lines.reverse();
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
    // Only Tab on a list line needs the outer list; skip the DOM walk otherwise.
    precedingLines:
      event.key === "Tab" && LIST_PATTERN.test(block.textContent ?? "")
        ? collectPrecedingLines(block)
        : [],
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
