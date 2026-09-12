import { getAutocompleteResults } from "./autocomplete.js";

export function createEditorAutocompleteManager({
  getEditorRef,
  getVaultNotes,
  getAutocompleteLimit,
  getShowAutocomplete,
  getAutocompleteIndex,
  getAutocompleteResultsList,
  onAutocompleteChange,
  onAutocompleteIndexChange,
  onChange,
}) {
  let autocompleteRange = null;

  function getAutocompleteMatches(query) {
    return getAutocompleteResults(
      query,
      getVaultNotes(),
      getAutocompleteLimit(),
    );
  }

  function emitAutocompleteChange({
    open = false,
    query = "",
    results = [],
    range = null,
    index = 0,
  } = {}) {
    autocompleteRange = range;
    onAutocompleteChange?.({
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
    const editorRef = getEditorRef();
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

  function clearAutocomplete() {
    autocompleteRange = null;
  }

  function insertAutocompleteResult(note) {
    if (!autocompleteRange) return;

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;

    const currentRange = selection.getRangeAt(0);
    const insertRange = document.createRange();
    insertRange.setStart(
      autocompleteRange.startContainer,
      autocompleteRange.startOffset,
    );
    insertRange.setEnd(
      currentRange.startContainer,
      currentRange.startOffset,
    );
    insertRange.deleteContents();

    const linkText = document.createTextNode(`[[${note.name}]]`);
    insertRange.insertNode(linkText);

    const afterRange = document.createRange();
    afterRange.setStartAfter(linkText);
    afterRange.collapse(true);
    selection.removeAllRanges();
    selection.addRange(afterRange);

    closeAutocompleteInternal();
    onChange?.();
  }

  function handleAutocompleteKeydown(event) {
    if (!getShowAutocomplete()) return false;

    const results = getAutocompleteResultsList();
    const index = getAutocompleteIndex();

    if (event.key === "ArrowDown") {
      event.preventDefault();
      onAutocompleteIndexChange?.(
        Math.min(index + 1, results.length - 1),
      );
      return true;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      onAutocompleteIndexChange?.(
        Math.max(index - 1, 0),
      );
      return true;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      if (results[index]) {
        insertAutocompleteResult(results[index]);
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

  return {
    checkAutocomplete,
    clearAutocomplete,
    insertAutocompleteResult,
    handleAutocompleteKeydown,
    closeAutocomplete: closeAutocompleteInternal,
  };
}
