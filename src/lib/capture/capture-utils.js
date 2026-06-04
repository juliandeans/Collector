export { matchesShortcut } from "../shortcutMatching.js";

/**
 * Checks if a drag event contains files.
 * @param {DragEvent} e
 * @returns {boolean}
 */
export function isFileDrag(e) {
    const types = e.dataTransfer?.types;
    if (types && Array.from(types).includes("Files")) return true;
    const items = e.dataTransfer?.items;
    return !!items && items.length > 0;
}

/**
 * Normalizes a file path, decoding file:// URLs.
 * @param {string} path
 * @returns {string}
 */
export function normalizeFilePath(path) {
    if (!path) return "";
    if (path.startsWith("file://")) {
        return decodeURIComponent(path.replace("file://", ""));
    }
    return path;
}

/**
 * Normalizes the result from save_image / save_image_from_bytes invoke calls.
 * @param {string|object} result
 * @returns {{ markdown: string, saved_path: string|null, filename: string|null, preview_data_url: string }}
 */
export function normalizeImageResult(result) {
    if (typeof result === "string") {
        return {
            markdown: result,
            saved_path: null,
            filename: null,
            preview_data_url: "",
        };
    }
    return {
        markdown: result?.markdown ?? "",
        saved_path: result?.saved_path ?? null,
        filename: result?.filename ?? null,
        preview_data_url: result?.preview_data_url ?? "",
    };
}

/**
 * Parses markdown headings from content.
 * @param {string} content
 * @returns {Array<{ level: number, text: string, display: string, lineIndex: number }>}
 */
export function parseHeadings(content) {
    const lines = content.split("\n");
    const headings = [];

    lines.forEach((line, index) => {
        const match = line.match(/^(#{1,6})\s+(.+)/);
        if (!match) return;

        headings.push({
            level: match[1].length,
            text: match[2].trim(),
            display: line.trim(),
            lineIndex: index,
        });
    });

    return headings;
}

/**
 * Formats a timestamp string with date/time replacements.
 * @param {string} [template="#### HH:mm"]
 * @returns {string}
 */
export function formatEntryHeader(template = "#### HH:mm") {
    const now = new Date();
    const replacements = {
        YYYY: String(now.getFullYear()),
        MM: String(now.getMonth() + 1).padStart(2, "0"),
        DD: String(now.getDate()).padStart(2, "0"),
        HH: String(now.getHours()).padStart(2, "0"),
        mm: String(now.getMinutes()).padStart(2, "0"),
        ss: String(now.getSeconds()).padStart(2, "0"),
    };

    return String(template ?? "")
        .replace(/YYYY/g, replacements.YYYY)
        .replace(/MM/g, replacements.MM)
        .replace(/DD/g, replacements.DD)
        .replace(/HH/g, replacements.HH)
        .replace(/mm/g, replacements.mm)
        .replace(/ss/g, replacements.ss);
}

/**
 * Extracts a vault note path from a note object.
 * @param {object} [note={}]
 * @returns {string}
 */
export function getVaultNotePath(note = {}) {
    return note?.relative_path || note?.path || "";
}
