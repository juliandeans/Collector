import {
  applySearchHighlights,
  clearSearchHighlights,
  runSearch as collectSearchMatches,
  stepSearch as getNextSearchIndex,
} from "./searchLogic.js";

export function createSearchCoordinator({
  getEditorComponent,
  getSearchInputRef,
  getSearchQuery,
  setSearchQuery,
  getSearchMatches,
  setSearchMatches,
  getSearchIndex,
  setSearchIndex,
  getShowSearch,
  setShowSearch,
  tick,
}) {
  function clearHighlights() {
    clearSearchHighlights();
  }

  function highlightMatches() {
    applySearchHighlights(getSearchMatches(), getSearchIndex());
  }

  function scrollToMatch(index) {
    const range = getSearchMatches()[index];
    const scrollEl = getEditorComponent()?.getScrollElement?.();
    if (!range || !scrollEl) return;

    const rect = range.getClientRects()[0] ?? range.getBoundingClientRect();
    const scrollRect = scrollEl.getBoundingClientRect();
    if (!rect) return;

    if (rect.top < scrollRect.top || rect.bottom > scrollRect.bottom) {
      const targetTop =
        scrollEl.scrollTop +
        (rect.top - scrollRect.top) -
        scrollEl.clientHeight / 2 +
        rect.height / 2;

      scrollEl.scrollTo({
        top: Math.max(targetTop, 0),
        behavior: "smooth",
      });
    }
  }

  function runSearch(eventOrQuery) {
    let query = "";
    if (typeof eventOrQuery === "string") {
      query = eventOrQuery;
      setSearchQuery(query);
    } else if (eventOrQuery?.currentTarget?.value !== undefined) {
      query = eventOrQuery.currentTarget.value;
      setSearchQuery(query);
    } else {
      query = getSearchQuery();
    }

    clearHighlights();
    setSearchMatches([]);
    setSearchIndex(0);

    const editorEl = getEditorComponent()?.getEditorElement?.();
    const matches = collectSearchMatches(query, editorEl);
    setSearchMatches(matches);
    applySearchHighlights(matches, 0);

    if (matches.length > 0) {
      const range = matches[0];
      const scrollEl = getEditorComponent()?.getScrollElement?.();
      if (range && scrollEl) {
        const rect = range.getClientRects()[0] ?? range.getBoundingClientRect();
        const scrollRect = scrollEl.getBoundingClientRect();
        if (rect && (rect.top < scrollRect.top || rect.bottom > scrollRect.bottom)) {
          const targetTop =
            scrollEl.scrollTop +
            (rect.top - scrollRect.top) -
            scrollEl.clientHeight / 2 +
            rect.height / 2;
          scrollEl.scrollTo({
            top: Math.max(targetTop, 0),
            behavior: "smooth",
          });
        }
      }
    }
  }

  function stepSearch(direction) {
    const matches = getSearchMatches();
    if (matches.length === 0) return;

    const nextIndex = getNextSearchIndex(matches, getSearchIndex(), direction);
    setSearchIndex(nextIndex);
    applySearchHighlights(matches, nextIndex);
    scrollToMatch(nextIndex);
  }

  function closeSearch() {
    setShowSearch(false);
    setSearchQuery("");
    clearHighlights();
    setSearchMatches([]);
    setSearchIndex(0);
    getEditorComponent()?.focus();
  }

  async function openSearch() {
    setShowSearch(true);
    await tick?.();
    const input = getSearchInputRef?.();
    input?.focus();
    input?.select();
  }

  return {
    clearHighlights,
    highlightMatches,
    scrollToMatch,
    runSearch,
    stepSearch,
    closeSearch,
    openSearch,
  };
}
