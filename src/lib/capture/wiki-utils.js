/**
 * Checks if there is an active [[wikilink trigger before the cursor.
 * Returns null if no active trigger is found (e.g. already closed, broken by newline).
 *
 * @param {string} text  – Full textarea value
 * @param {number} cursor – Caret position
 * @returns {{ triggerIndex: number, query: string } | null}
 */
export function findWikiTrigger(text, cursor) {
   const before = text.slice(0, cursor);
   const triggerIndex = before.lastIndexOf("[[");

   if (
      triggerIndex === -1 ||
      before.slice(triggerIndex + 2).includes("]]") ||
      before.slice(triggerIndex + 2).includes("\n")
   ) {
      return null;
   }

   const query = before.slice(triggerIndex + 2);
   return { triggerIndex, query };
}

/**
 * Computes the string values needed to insert a completed [[wikilink]] at
 * the cursor position, replacing the trigger text.
 *
 * @param {string} text       – Full textarea value
 * @param {number} cursor     – Current caret position
 * @param {number} queryLength – Length of the autocomplete query
 * @param {string} noteName   – Name of the note to link
 * @returns {{ before: string, after: string, insertion: string, newPosition: number }}
 */
export function computeWikilinkInsertion(text, cursor, queryLength, noteName) {
   const anchor = cursor - queryLength - 2;
   const before = text.slice(0, anchor);
   const after = text.slice(cursor);
   const insertion = `[[${noteName}]]`;
   const newPosition = before.length + insertion.length;

   return { before, after, insertion, newPosition };
}

/**
 * Computes the string values needed to insert a completed #tag at the cursor,
 * replacing the tag trigger and adding one trailing space.
 *
 * @param {string} text       – Full textarea value
 * @param {number} cursor     – Current caret position
 * @param {number} queryLength – Length of the autocomplete query
 * @param {string} tag        – Tag to insert without the leading #
 * @returns {{ before: string, after: string, insertion: string, newPosition: number }}
 */
export function computeTagInsertion(text, cursor, queryLength, tag) {
   const anchor = cursor - queryLength - 1;
   const before = text.slice(0, anchor);
   const after = text.slice(cursor);
   const insertion = `#${tag} `;
   const newPosition = before.length + insertion.length;

   return { before, after, insertion, newPosition };
}
