const WORD_CHARACTER_RE = /[\p{L}\p{N}_]/u;

export function getTagAutocompleteResults(query = "", tags = [], limit = 20) {
  if (!query) return tags.slice(0, limit);

  const lowerQuery = query.toLowerCase();
  return tags
    .filter((tag) => tag.toLowerCase().includes(lowerQuery))
    .slice(0, limit);
}

export function findTagTrigger(text = "", cursor = text.length) {
  const value = String(text ?? "");
  const numericCursor = Number(cursor);
  const cursorIndex = Number.isFinite(numericCursor)
    ? Math.min(Math.max(Math.trunc(numericCursor), 0), value.length)
    : value.length;
  const triggerIndex = value.slice(0, cursorIndex).lastIndexOf("#");

  if (triggerIndex < 0 || cursorIndex <= triggerIndex) return null;

  const previous = getPreviousCharacter(value, triggerIndex);
  if (!isTagBoundary(previous)) return null;

  // Allow an empty query (bare # right before the caret) so typing # opens
  // the full tag list like Obsidian. Reject headings/plain hashes: the next
  // character must not be whitespace or another #.
  const next = getCharacterAt(value, triggerIndex + 1);
  if (next && (/\s/u.test(next) || next === "#")) return null;

  const query = value.slice(triggerIndex + 1, cursorIndex);
  if (/\s/u.test(query)) return null;

  return { triggerIndex, query };
}

function getCharacterAt(text, index) {
  if (index < 0 || index >= text.length) return null;

  const codePoint = text.codePointAt(index);
  return String.fromCodePoint(codePoint);
}

function getPreviousCharacter(text, index) {
  if (index <= 0) return null;

  const previousCodeUnit = text.charCodeAt(index - 1);
  if (previousCodeUnit >= 0xdc00 && previousCodeUnit <= 0xdfff && index >= 2) {
    const precedingCodeUnit = text.charCodeAt(index - 2);
    if (precedingCodeUnit >= 0xd800 && precedingCodeUnit <= 0xdbff) {
      return text.slice(index - 2, index);
    }
  }

  return text.slice(index - 1, index);
}

function isTagBoundary(previous) {
  return (
    previous === null ||
    /\s/u.test(previous) ||
    (!WORD_CHARACTER_RE.test(previous) && previous !== "#")
  );
}
