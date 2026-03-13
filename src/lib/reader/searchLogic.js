export function clearSearchHighlights() {
  if (typeof CSS !== "undefined" && CSS.highlights) {
    CSS.highlights.clear();
  }
}

export function applySearchHighlights(matches = [], activeIndex = 0) {
  clearSearchHighlights();

  if (
    typeof CSS === "undefined" ||
    !CSS.highlights ||
    typeof Highlight === "undefined" ||
    matches.length === 0
  ) {
    return;
  }

  const allHighlight = new Highlight(...matches);
  CSS.highlights.set("search-result", allHighlight);

  if (matches[activeIndex]) {
    const activeHighlight = new Highlight(matches[activeIndex]);
    CSS.highlights.set("search-active", activeHighlight);
  }
}

export function runSearch(query = "", editorEl) {
  if (!query.trim() || !editorEl) return [];

  const walker = document.createTreeWalker(
    editorEl,
    NodeFilter.SHOW_TEXT,
    null,
  );
  const lowerQuery = query.toLowerCase();
  const ranges = [];
  let node;

  while ((node = walker.nextNode())) {
    const text = node.textContent ?? "";
    const lower = text.toLowerCase();
    let position = 0;

    while (true) {
      const matchIndex = lower.indexOf(lowerQuery, position);
      if (matchIndex === -1) break;

      const range = document.createRange();
      range.setStart(node, matchIndex);
      range.setEnd(node, matchIndex + lowerQuery.length);
      ranges.push(range);
      position = matchIndex + 1;
    }
  }

  return ranges;
}

export function stepSearch(matches = [], currentIndex = 0, direction = 1) {
  if (matches.length === 0) return 0;

  return (currentIndex + direction + matches.length) % matches.length;
}
