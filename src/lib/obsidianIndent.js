import { DEFAULT_INDENT_UNIT } from "./capture/list-continuation.js";

/**
 * Indent unit for list Tab, mirroring the vault's Obsidian setting
 * (`get_obsidian_indent`): a tab, or 4 spaces when `use_tab` is false
 * (Obsidian always uses 4 spaces then). Anything unexpected falls back to
 * Obsidian's default (a tab).
 */
export function indentUnitFrom(config) {
    return config?.use_tab === false ? "    " : DEFAULT_INDENT_UNIT;
}

export async function loadIndentUnit(invoke) {
    try {
        return indentUnitFrom(await invoke("get_obsidian_indent"));
    } catch {
        return DEFAULT_INDENT_UNIT;
    }
}

/**
 * Loads can overlap (window shown twice quickly, slow iCloud read); only the
 * newest result is applied, like createVaultNotesRefresher.
 */
export function createIndentUnitRefresher({ invoke, setUnit }) {
    let latestTicket = 0;
    return async function refreshIndentUnit() {
        const ticket = ++latestTicket;
        const unit = await loadIndentUnit(invoke);
        if (ticket === latestTicket) setUnit(unit);
        return unit;
    };
}
