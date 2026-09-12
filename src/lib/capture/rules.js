const WORD_CHARACTER_RE = /[\p{L}\p{N}_]/u;

/**
 * @typedef {Object} CaptureRuleCondition
 * @property {string} type
 * @property {string} [value]
 * @property {boolean} [caseSensitive]
 * @property {boolean} [case_sensitive]
 */

/**
 * @typedef {Object} CaptureRuleAction
 * @property {string} type
 * @property {string} [note]
 * @property {string} [heading]
 * @property {string} [position]
 * @property {boolean} [createHeadingIfMissing]
 * @property {boolean} [stripTrigger]
 */

/**
 * @typedef {Object} CaptureRule
 * @property {string} [id]
 * @property {string} [name]
 * @property {boolean} [enabled]
 * @property {CaptureRuleCondition[]} [conditions]
 * @property {"any"|"all"} [match]
 * @property {CaptureRuleAction[]} [actions]
 */

function asString(value) {
 return typeof value === "string" ? value : "";
}

function hasCaseSensitiveFlag(condition) {
 return condition?.caseSensitive ?? condition?.case_sensitive ?? false;
}

function escapeRegExp(value) {
 return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function findAllOccurrences(
 text,
 trigger,
 { caseSensitive, matchesToken } = {},
) {
 const value = asString(text);
 const needle = asString(trigger);
 if (!needle) return [];

 // Regex-basierte Suche (ohne lookbehind): Indizes beziehen sich direkt auf
 // den Originaltext, ein Case-Folding mit geaenderter Laenge (z.B. "\u0130")
 // kann die Offsets daher nicht verschieben.
 const pattern = new RegExp(escapeRegExp(needle), caseSensitive ? "g" : "gi");
 const occurrences = [];

 for (const match of value.matchAll(pattern)) {
  const index = match.index;
  if (!matchesToken || matchesToken(value, index, needle)) {
   occurrences.push({ index, length: needle.length });
  }
 }

 return occurrences;
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

function isWordCharacter(character) {
 return character !== null && WORD_CHARACTER_RE.test(character);
}

function isHashtagToken(text, index, trigger) {
 const previous = getPreviousCharacter(text, index);
 const next = getCharacterAt(text, index + trigger.length);

 // A second hash is not a token boundary, so headings and ##hashtags do not
 // accidentally match a single-hash rule.
 return (
  !isWordCharacter(previous) &&
  previous !== "#" &&
  !isWordCharacter(next) &&
  next !== "#"
 );
}

function containsHandler(text, condition) {
 const value = asString(condition?.value);
 if (!value) return false;

 const caseSensitive = Boolean(hasCaseSensitiveFlag(condition));
 const haystack = caseSensitive ? asString(text) : asString(text).toLowerCase();
 const needle = caseSensitive ? value : value.toLowerCase();
 return haystack.includes(needle);
}

function hashtagHandler(text, condition) {
 const value = asString(condition?.value);
 if (!value) return false;

 return (
  findAllOccurrences(asString(text), value, {
   caseSensitive: false,
   matchesToken: isHashtagToken,
  }).length > 0
 );
}

export const conditionHandlers = {
 contains: containsHandler,
 hashtag: hashtagHandler,
};

/**
 * Dispatch table for action types. Action handlers are used by submit
 * consumers to skip actions introduced by a newer build.
 */
export const actionHandlers = {
 append: () => true,
};

function conditionMatches(text, condition) {
 const type = condition?.type;
 // Own-property check: prototype keys like "toString" must never resolve
 // to a handler and let a foreign type match.
 if (typeof type !== "string" || !Object.hasOwn(conditionHandlers, type)) {
  return false;
 }
 return conditionHandlers[type](text, condition);
}

/**
 * Evaluate ordered capture rules. The first matching enabled rule wins.
 *
 * @param {string} text
 * @param {CaptureRule[]} rules
 * @returns {{ matched: false } | { matched: true, rule: CaptureRule, actions: CaptureRuleAction[] }}
 */
export function evaluateCaptureRules(text, rules) {
 const candidates = Array.isArray(rules) ? rules : [];

 for (const rule of candidates) {
  if (!rule || rule.enabled === false) continue;

  const conditions = Array.isArray(rule.conditions) ? rule.conditions : [];
  if (conditions.length === 0) continue;

  const conditionResults = conditions.map((condition) =>
   conditionMatches(text, condition),
  );
  const matches =
   rule.match === "all"
    ? conditionResults.every(Boolean)
    : conditionResults.some(Boolean);

  if (matches) {
   return {
    matched: true,
    rule,
    actions: Array.isArray(rule.actions) ? rule.actions : [],
   };
  }
 }

 return { matched: false };
}

function removeOccurrences(text, occurrences) {
 if (occurrences.length === 0) return text;

 let result = "";
 let cursor = 0;
 for (const occurrence of occurrences) {
  result += text.slice(cursor, occurrence.index);
  cursor = occurrence.index + occurrence.length;
 }
 return result + text.slice(cursor);
}

function cleanStrippedText(text) {
 return asString(text)
  .split(/\r?\n/u)
  .map((line) => line.replace(/\s+/gu, " ").trim())
  .filter((line) => line.length > 0)
  .join("\n");
}

/**
 * Remove all trigger occurrences from a capture and clean the resulting
 * whitespace. If stripping would leave no content, retain the original text
 * so the backend's non-empty append guard does not reject the capture.
 *
 * @param {string} text
 * @param {CaptureRuleCondition[]} conditions
 * @returns {string}
 */
export function stripTriggers(text, conditions) {
 const original = asString(text);
 let stripped = original;

 if (Array.isArray(conditions)) {
  for (const condition of conditions) {
   const type = condition?.type;
   const value = asString(condition?.value);
   if (!value || !["contains", "hashtag"].includes(type)) continue;

   const occurrences = findAllOccurrences(stripped, value, {
    caseSensitive:
     type === "contains" && Boolean(hasCaseSensitiveFlag(condition)),
    matchesToken: type === "hashtag" ? isHashtagToken : undefined,
   });
   stripped = removeOccurrences(stripped, occurrences);
  }
 }

 const cleaned = cleanStrippedText(stripped);
 return cleaned ? cleaned : original;
}

/**
 * Return the display target of the first append action in a rule.
 *
 * @param {CaptureRule} rule
 * @returns {string}
 */
export function describeRuleTarget(rule) {
 const actions = Array.isArray(rule?.actions) ? rule.actions : [];
 const action = actions.find(
  (candidate) =>
   candidate?.type === "append" && typeof candidate.note === "string",
 );
 if (!action) return "";

 const note = action.note.trim();
 const heading = asString(action.heading).trim();
 return heading ? `${note} / ${heading}` : note;
}
