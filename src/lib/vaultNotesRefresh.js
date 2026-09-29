/**
 * Wraps the vault note-list fetch so that only the newest response is applied.
 * Several triggers (window shown, palette opened) can overlap; a slower, older
 * response must not overwrite a newer list.
 */
export function createVaultNotesRefresher({ fetchNotes, setNotes }) {
    let latestTicket = 0;

    return async function refreshVaultNotes(mode = "ifStale") {
        const ticket = ++latestTicket;
        const notes = await fetchNotes(mode);
        if (ticket === latestTicket) {
            setNotes(notes);
        }
        return notes;
    };
}

/** Index of `note` (matched by path) in `notes`; 0 if absent, so the first row is selected. */
export function findNoteIndex(notes = [], note) {
    if (!note) return 0;
    const index = notes.findIndex((entry) => entry.path === note.path);
    return index >= 0 ? index : 0;
}

export function vaultPathChanged(previousSettings, nextSettings) {
    return (previousSettings?.vault_path ?? "") !== (nextSettings?.vault_path ?? "");
}
