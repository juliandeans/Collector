<script>
  import { createEventDispatcher, onMount, onDestroy, tick } from "svelte";
  import {
    getCachedImageSrc,
    markdownLineToHtml,
    markdownToHtml,
    normalizeNewlines,
    preprocessContent,
    warmImagesInText,
  } from "./contentProcessing.js";
  import { elementToMarkdownLine, htmlToMarkdown } from "./editorSerialization.js";

  export let rawContent = "";
  export let appSettings = {};
  export let vaultNotes = [];
  export let missingFileMessage = "";
  export let showSearch = false;
  export let showAutocomplete = false;
  export let autocompleteResults = [];
  export let autocompleteIndex = 0;
  export let codeblockMap = new Map();
  export let hiddenBlockMap = new Map();
  export let setStrippedFrontmatter = () => {};

  const dispatch = createEventDispatcher();

  let editorRef;
  let scrollRef;
  let isComposing = false;
  let isRenderingContent = false;
  let activeParagraphEl = null;
  let renderRequestId = 0;
  let autocompleteRange = null;

  function getAutocompleteMatches(query) {
    if (!query) return vaultNotes.slice(0, 10);

    const lower = query.toLowerCase();
    return vaultNotes
      .filter(
        (note) =>
          note.name.toLowerCase().includes(lower) ||
          note.relative_path.toLowerCase().includes(lower),
      )
      .slice(0, 8);
  }

  function emitAutocompleteChange({
    open = false,
    query = "",
    results = [],
    range = null,
    index = 0,
  } = {}) {
    autocompleteRange = range;
    dispatch("autocompleteChange", {
      open,
      query,
      results,
      range,
      index,
    });
  }

  function closeAutocompleteInternal() {
    emitAutocompleteChange({
      open: false,
      query: "",
      results: [],
      range: null,
      index: 0,
    });
  }

  function checkAutocomplete() {
    if (!editorRef) return;

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) {
      closeAutocompleteInternal();
      return;
    }

    const range = selection.getRangeAt(0);
    const node = range.startContainer;
    if (node.nodeType !== Node.TEXT_NODE) {
      closeAutocompleteInternal();
      return;
    }

    const text = node.textContent ?? "";
    const cursor = range.startOffset;
    const before = text.slice(0, cursor);
    const triggerIndex = before.lastIndexOf("[[");

    if (triggerIndex === -1) {
      closeAutocompleteInternal();
      return;
    }

    const between = before.slice(triggerIndex + 2);
    if (between.includes("]]") || between.includes("\n")) {
      closeAutocompleteInternal();
      return;
    }

    const results = getAutocompleteMatches(between);
    const triggerRange = document.createRange();
    triggerRange.setStart(node, triggerIndex);
    triggerRange.collapse(true);

    emitAutocompleteChange({
      open: results.length > 0,
      query: between,
      results,
      range: triggerRange,
      index: 0,
    });
  }

  export function clearAutocomplete() {
    autocompleteRange = null;
  }

  export function insertAutocompleteResult(note) {
    if (!autocompleteRange) return;

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;

    const currentRange = selection.getRangeAt(0);
    const insertRange = document.createRange();
    insertRange.setStart(
      autocompleteRange.startContainer,
      autocompleteRange.startOffset,
    );
    insertRange.setEnd(currentRange.startContainer, currentRange.startOffset);
    insertRange.deleteContents();

    const linkText = document.createTextNode(`[[${note.name}]]`);
    insertRange.insertNode(linkText);

    const afterRange = document.createRange();
    afterRange.setStartAfter(linkText);
    afterRange.collapse(true);
    selection.removeAllRanges();
    selection.addRange(afterRange);

    closeAutocompleteInternal();
    dispatch("change");
  }

  function handleAutocompleteKeydown(event) {
    if (!showAutocomplete) return false;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      dispatch(
        "autocompleteIndexChange",
        Math.min(autocompleteIndex + 1, autocompleteResults.length - 1),
      );
      return true;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      dispatch("autocompleteIndexChange", Math.max(autocompleteIndex - 1, 0));
      return true;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      if (autocompleteResults[autocompleteIndex]) {
        insertAutocompleteResult(autocompleteResults[autocompleteIndex]);
      }
      return true;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      closeAutocompleteInternal();
      return true;
    }

    return false;
  }

  function placeCursorAtEnd(el) {
    if (!el) return;

    const range = document.createRange();
    const selection = window.getSelection();
    if (!selection) return;

    range.selectNodeContents(el);
    range.collapse(false);
    selection.removeAllRanges();
    selection.addRange(range);
  }

  function rerenderBlock(el) {
    if (!editorRef || !el || !editorRef.contains(el)) return null;

    const raw = elementToMarkdownLine(el);
    if (raw === null) return el;

    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = markdownLineToHtml(raw);
    const newEl = tempDiv.firstElementChild;
    if (!newEl) return el;

    el.replaceWith(newEl);
    return newEl;
  }

  function finalizeActiveBlock() {
    if (activeParagraphEl && editorRef && editorRef.contains(activeParagraphEl)) {
      rerenderBlock(activeParagraphEl);
    }
    activeParagraphEl = null;
  }

  function switchActiveBlock(newEl) {
    if (activeParagraphEl && editorRef && editorRef.contains(activeParagraphEl)) {
      rerenderBlock(activeParagraphEl);
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
    if (isRenderingContent || !editorRef) return;

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
    checkAutocomplete();
    dispatch("selectionChange");
  }

  function handleEditorBlur(event) {
    if (editorRef && editorRef.contains(event.relatedTarget)) return;
    finalizeActiveBlock();
  }

  function patchResolvedImages() {
    if (!editorRef) return;

    const blankImages = editorRef.querySelectorAll('img[src=""]');
    for (const image of blankImages) {
      const path = image.getAttribute("data-path") || image.getAttribute("alt");
      if (!path) continue;

      const cachedSrc = getCachedImageSrc(path);
      if (cachedSrc) {
        image.src = cachedSrc;
      }
    }
  }

  export function getMarkdown() {
    if (!editorRef) {
      return preprocessContent(rawContent, {
        appSettings,
        codeblockMap,
        hiddenBlockMap,
        setStrippedFrontmatter,
      });
    }

    return normalizeNewlines(htmlToMarkdown(editorRef));
  }

  export async function renderContent(raw = "") {
    const processed = preprocessContent(raw, {
      appSettings,
      codeblockMap,
      hiddenBlockMap,
      setStrippedFrontmatter,
    });
    activeParagraphEl = null;

    if (!editorRef) return;
    if (scrollRef) scrollRef.scrollTop = 0;

    const requestId = ++renderRequestId;
    isRenderingContent = true;
    editorRef.innerHTML = markdownToHtml(processed, appSettings);
    isRenderingContent = false;

    const loadedAnyMissingImages = await warmImagesInText(processed);
    if (!loadedAnyMissingImages) {
      return;
    }

    if (requestId !== renderRequestId || !editorRef) {
      return;
    }

    patchResolvedImages();
  }

  export async function restoreScroll(path, scrollPositions) {
    await tick();
    await new Promise((resolve) => {
      requestAnimationFrame(() => {
        if (scrollRef) {
          scrollRef.scrollTop = scrollPositions.get(path) ?? 0;
        }
        resolve();
      });
    });
  }

  export function finalizeBlock() {
    finalizeActiveBlock();
  }

  export function focus() {
    editorRef?.focus();
  }

  export function getEditorElement() {
    return editorRef;
  }

  export function getScrollElement() {
    return scrollRef;
  }

  export function getScrollTop() {
    return scrollRef?.scrollTop ?? 0;
  }

  function handleInput() {
    if (isComposing || isRenderingContent || !editorRef) return;
    dispatch("change");
    checkAutocomplete();
  }

  function handleCompositionStart() {
    isComposing = true;
  }

  function handleCompositionEnd() {
    isComposing = false;
    dispatch("change");
    checkAutocomplete();
  }

  function handleKeydown(event) {
    const key = event.key.toLowerCase();

    if (handleAutocompleteKeydown(event)) {
      event.stopPropagation();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "f") {
      event.preventDefault();
      event.stopPropagation();
      dispatch("openSearchRequest");
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      if (showAutocomplete) {
        closeAutocompleteInternal();
        return;
      }
      if (showSearch) {
        dispatch("closeSearchRequest");
        return;
      }
      dispatch("closeRequest");
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "s") {
      event.preventDefault();
      event.stopPropagation();
      dispatch("saveRequest");
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "k") {
      event.preventDefault();
      event.stopPropagation();
      dispatch("openPaletteRequest");
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "p") {
      event.preventDefault();
      event.stopPropagation();
      dispatch("openPaletteRequest");
    }
  }

  function handleEditorMouseDown(event) {
    const wikilink = event.target?.closest?.(".wikilink");
    if (wikilink) {
      event.preventDefault();
      event.stopPropagation();

      const target = wikilink.dataset.target;
      if (!target) return;

      dispatch("navigateWikilink", {
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
      dispatch("openExternalLink", href);
    }
  }

  function insertPlainTextAtSelection(text) {
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

  function handleEditorPaste(event) {
    const plainText = event.clipboardData?.getData("text/plain") ?? "";
    if (!plainText) return;

    event.preventDefault();
    event.stopPropagation();

    if (!insertPlainTextAtSelection(plainText)) return;

    dispatch("change");
    checkAutocomplete();
  }

  onMount(() => {
    document.addEventListener("selectionchange", handleSelectionChange);
  });

  onDestroy(() => {
    document.removeEventListener("selectionchange", handleSelectionChange);
  });
</script>

<div class="editor-scroll" bind:this={scrollRef} on:scroll={() => dispatch("scroll")}>
  {#if missingFileMessage.trim()}
    <div class="missing-file-banner">{missingFileMessage}</div>
  {/if}
  <div
    class="editor-body"
    contenteditable="true"
    bind:this={editorRef}
    spellcheck="true"
    role="textbox"
    aria-multiline="true"
    tabindex="0"
    data-placeholder="Start writing..."
    on:input={handleInput}
    on:keydown={handleKeydown}
    on:paste={handleEditorPaste}
    on:mousedown={handleEditorMouseDown}
    on:blur={handleEditorBlur}
    on:compositionstart={handleCompositionStart}
    on:compositionend={handleCompositionEnd}
  ></div>
</div>

<style>
  .editor-scroll {
    position: relative;
    flex: 1;
    overflow-y: auto;
    background: transparent;
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .missing-file-banner {
    margin: 12px 16px 0;
    padding: 10px 12px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.72);
    font-size: 12px;
    border: 0.5px solid rgba(255, 255, 255, 0.08);
  }

  .editor-body {
    min-height: 100%;
    padding: 16px 20px 40px;
    outline: none;
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    line-height: 1.7;
    word-break: break-word;
    white-space: pre-wrap;
    caret-color: var(--accent-color, #8b5cf6);
    -webkit-user-modify: read-write;
  }

  .editor-body[data-placeholder]:empty::before {
    content: attr(data-placeholder);
    color: var(--placeholder-color);
    pointer-events: none;
  }

  .editor-body :global(p) {
    margin: 0;
    min-height: 1.7em;
  }

  .editor-body :global(h1) {
    margin: 8px 0 4px;
    font-size: 1.5em;
    font-weight: 700;
    line-height: 1.25;
  }

  .editor-body :global(h2) {
    margin: 6px 0 3px;
    font-size: 1.25em;
    font-weight: 600;
    line-height: 1.3;
  }

  .editor-body :global(h3) {
    margin: 4px 0 2px;
    font-size: 1.1em;
    font-weight: 600;
    line-height: 1.35;
  }

  .editor-body :global(h4) {
    margin: 3px 0 2px;
    font-size: 1em;
    font-weight: 600;
    line-height: 1.4;
  }

  .editor-body :global(h5) {
    margin: 2px 0 1px;
    font-size: 0.94em;
    font-weight: 600;
    line-height: 1.45;
    letter-spacing: 0.01em;
  }

  .editor-body :global(h6) {
    margin: 2px 0 1px;
    font-size: 0.88em;
    font-weight: 600;
    line-height: 1.45;
    letter-spacing: 0.02em;
    color: var(--reader-text-secondary);
  }

  .editor-body :global(blockquote) {
    margin: 2px 0;
    padding-left: 12px;
    border-left: 3px solid var(--accent-color, #8b5cf6);
    color: var(--reader-text-secondary);
  }

  .editor-body :global(.callout) {
    margin: 6px 0;
    padding: 10px 14px;
    border-left: 3px solid;
    border-radius: 6px;
    white-space: normal;
  }

  .editor-body :global(.callout-title) {
    display: flex;
    align-items: center;
    gap: 6px;
    margin-bottom: 4px;
    font-size: 13px;
    font-weight: 600;
  }

  .editor-body :global(.callout-icon) {
    font-size: 14px;
    line-height: 1;
  }

  .editor-body :global(.callout-content) {
    font-size: 13px;
    line-height: 1.6;
    opacity: 0.9;
  }

  .editor-body :global(code) {
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(0, 0, 0, 0.08);
    font-family: "SF Mono", Monaco, monospace;
    font-size: 0.88em;
  }

  .editor-body :global(strong) {
    font-weight: 600;
  }

  .editor-body :global(em) {
    font-style: italic;
  }

  .editor-body :global(a) {
    color: var(--external-link-color, #60a5fa);
    text-decoration: none;
    cursor: pointer;
  }

  .editor-body :global(a:hover) {
    text-decoration: underline;
  }

  .editor-body :global(hr) {
    margin: 8px 0;
    border: none;
    border-top: 1px solid var(--border-color);
  }

  .editor-body :global(p.list-item)::before {
    content: "•";
    margin-right: 8px;
    color: var(--reader-text-secondary);
  }

  .editor-body :global(.md-checkbox) {
    margin-right: 6px;
    accent-color: var(--accent-color, #8b5cf6);
    cursor: pointer;
  }

  .editor-body :global(.wikilink) {
    color: var(--internal-link-color, #a78bfa);
    opacity: 0.9;
    cursor: pointer;
    border-bottom: 1px solid transparent;
    transition:
      border-color 0.15s ease,
      opacity 0.15s ease;
  }

  .editor-body :global(.wikilink:hover) {
    opacity: 1;
    border-bottom-color: var(--internal-link-color, #a78bfa);
  }

  .editor-body :global(.md-image) {
    display: block;
    max-width: 100%;
    height: auto;
    margin: 8px 0;
    border-radius: 6px;
    cursor: default;
  }

  .editor-body :global(p:has(.md-image)) {
    margin: 4px 0;
  }

  .editor-body :global(.md-image[src=""]) {
    display: none;
  }

  .editor-body :global(.codeblock-pill) {
    display: inline-flex;
    align-items: center;
    width: fit-content;
    min-height: 20px;
    gap: 5px;
    margin: 2px 0;
    padding: 2px 6px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.78);
    cursor: default;
  }

  .editor-body :global(.codeblock-pill):hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .editor-body :global(.codeblock-icon) {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--accent-color, #8b5cf6);
    opacity: 0.85;
  }

  .editor-body :global(.codeblock-lang) {
    font-size: 11px;
    font-weight: 600;
    text-transform: lowercase;
  }

  .editor-body :global(.hidden-marker) {
    display: none;
  }

  .editor-body :global(.raw-mode) {
    margin-left: -4px;
    padding-left: 4px;
    border-radius: 3px;
    background: color-mix(
      in srgb,
      var(--accent-color, #8b5cf6) 10%,
      transparent
    );
  }

  .editor-scroll::-webkit-scrollbar {
    width: 6px;
  }

  .editor-scroll::-webkit-scrollbar-track {
    background: transparent;
  }

  .editor-scroll::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.12);
    border-radius: 3px;
  }
</style>
