import { elementToMarkdownLine } from "./editorSerialization.js";
import { applyDataStyles, markdownLineToHtml } from "./contentProcessing.js";

export function placeCursorAtEnd(el) {
  if (!el) return;

  const range = document.createRange();
  const selection = window.getSelection();
  if (!selection) return;

  range.selectNodeContents(el);
  range.collapse(false);
  selection.removeAllRanges();
  selection.addRange(range);
}

export function rerenderBlock(el, editorRef) {
  if (!editorRef || !el || !editorRef.contains(el)) return null;

  const raw = elementToMarkdownLine(el);
  if (raw === null) return el;

  const tempDiv = document.createElement("div");
  tempDiv.innerHTML = markdownLineToHtml(raw);
  const newEl = tempDiv.firstElementChild;
  if (!newEl) return el;

  applyDataStyles(newEl);
  el.replaceWith(newEl);
  return newEl;
}

export function createBlockManager({
  getEditorRef,
  getIsRendering,
  onSelectionChange,
  checkAutocomplete,
}) {
  let activeParagraphEl = null;

  function finalizeActiveBlock() {
    const editorRef = getEditorRef();
    if (activeParagraphEl && editorRef && editorRef.contains(activeParagraphEl)) {
      rerenderBlock(activeParagraphEl, editorRef);
    }
    activeParagraphEl = null;
  }

  function switchActiveBlock(newEl) {
    const editorRef = getEditorRef();
    if (activeParagraphEl && editorRef && editorRef.contains(activeParagraphEl)) {
      rerenderBlock(activeParagraphEl, editorRef);
      activeParagraphEl = null;
    }

    if (!editorRef || !newEl || !editorRef.contains(newEl)) {
      activeParagraphEl = null;
      return;
    }

    const raw = elementToMarkdownLine(newEl);
    if (raw === null) {
      activeParagraphEl = null;
      return;
    }

    activeParagraphEl = newEl;
    activeParagraphEl.classList.add("raw-mode");
    activeParagraphEl.textContent = raw;
    placeCursorAtEnd(activeParagraphEl);
  }

  function handleSelectionChange() {
    const editorRef = getEditorRef();
    if (getIsRendering() || !editorRef) return;

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;
    if (!selection.isCollapsed) return;

    const node = selection.getRangeAt(0).startContainer;
    const anchorEl =
      node.nodeType === Node.TEXT_NODE ? node.parentElement : node;
    if (anchorEl?.closest?.(".wikilink")) return;
    if (!editorRef.contains(node)) return;

    let el = anchorEl;
    while (el && el !== editorRef && el.parentElement !== editorRef) {
      el = el.parentElement;
    }

    if (!el || el === editorRef || el === activeParagraphEl) return;
    if (el.classList?.contains("codeblock-pill")) return;

    switchActiveBlock(el);
    checkAutocomplete?.();
    onSelectionChange?.();
  }

  function handleEditorBlur(event) {
    const editorRef = getEditorRef();
    if (editorRef && editorRef.contains(event.relatedTarget)) return;
    finalizeActiveBlock();
  }

  function clearActive() {
    activeParagraphEl = null;
  }

  return {
    getActiveParagraph: () => activeParagraphEl,
    clearActive,
    finalizeActiveBlock,
    switchActiveBlock,
    handleSelectionChange,
    handleEditorBlur,
  };
}
