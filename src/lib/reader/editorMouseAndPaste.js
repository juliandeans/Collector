import { normalizeNewlines } from "./contentProcessing.js";
import { placeCursorAtEnd } from "./editorBlockManager.js";

export function insertPlainTextAtSelection(text, editorRef) {
  if (!editorRef) return false;

  const normalizedText = normalizeNewlines(text ?? "");
  if (!normalizedText) return false;

  const selection = window.getSelection();
  if (!selection || selection.rangeCount === 0) {
    editorRef.append(document.createTextNode(normalizedText));
    placeCursorAtEnd(editorRef);
    return true;
  }

  const range = selection.getRangeAt(0);
  if (!editorRef.contains(range.commonAncestorContainer)) {
    editorRef.append(document.createTextNode(normalizedText));
    placeCursorAtEnd(editorRef);
    return true;
  }

  range.deleteContents();
  const textNode = document.createTextNode(normalizedText);
  range.insertNode(textNode);

  const afterRange = document.createRange();
  afterRange.setStart(textNode, textNode.textContent?.length ?? 0);
  afterRange.collapse(true);
  selection.removeAllRanges();
  selection.addRange(afterRange);

  return true;
}

export function handleEditorMouseDown(event, { onNavigateWikilink, onOpenExternalLink }) {
  const wikilink = event.target?.closest?.(".wikilink");
  if (wikilink) {
    event.preventDefault();
    event.stopPropagation();

    const target = wikilink.dataset.target;
    if (!target) return;

    onNavigateWikilink?.({
      target,
      newTab: event.metaKey || event.ctrlKey,
    });
    return;
  }

  const anchor = event.target?.closest?.("a[href]");
  if (!anchor) return;

  const href = anchor.getAttribute("href");
  if (href && /^(https?:\/\/|obsidian:\/\/)/i.test(href)) {
    event.preventDefault();
    event.stopPropagation();
    onOpenExternalLink?.(href);
  }
}

export function handleEditorPaste(event, editorRef, { onChange, checkAutocomplete }) {
  const plainText = event.clipboardData?.getData("text/plain") ?? "";
  if (!plainText) return;

  event.preventDefault();
  event.stopPropagation();

  if (!insertPlainTextAtSelection(plainText, editorRef)) return;

  onChange?.();
  checkAutocomplete?.();
}
