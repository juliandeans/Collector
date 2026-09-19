/**
 * Determines what action should be taken regarding capture draft persistence.
 *
 * @param {Object} params
 * @param {string} [params.content]
 * @param {boolean} [params.isTauri]
 * @param {boolean} [params.isRestoringDraft]
 * @returns {{ action: "ignore" } | { action: "clear" } | { action: "save", content: string }}
 */
export function computeDraftAction({ content = "", isTauri = false, isRestoringDraft = false } = {}) {
    if (!isTauri || isRestoringDraft) {
        return { action: "ignore" };
    }
    if (!content || !content.trim()) {
        return { action: "clear" };
    }
    return { action: "save", content };
}
