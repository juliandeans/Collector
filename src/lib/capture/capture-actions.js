/**
 * Action: append current content to the daily note.
 */
export async function appendToDailyNoteAction({
    content,
    invoke,
    showStatus,
    freeBlobUrls,
    resetCaptureState,
    deferHideCapture,
    setLoading,
}) {
    try {
        await invoke("append_to_daily_note", { text: content });
        freeBlobUrls();
        resetCaptureState();
        deferHideCapture();
    } catch (e) {
        console.error("Append to daily note failed:", e);
        showStatus("✗ " + e.toString(), "error");
        setLoading(false);
    }
}

/**
 * Action: save current content as a new note.
 * Extracts title from first heading line if present (e.g. "# My Title").
 */
export async function saveAsNoteAction({
    content,
    invoke,
    showStatus,
    freeBlobUrls,
    resetCaptureState,
    deferHideCapture,
    setLoading,
}) {
    try {
        const trimmed = content.trim();
        const lines = trimmed.split("\n");
        const firstLine = lines[0].trimEnd();

        let title = null;
        let body = trimmed;

        if (firstLine.match(/^#\s+.+/)) {
            title = firstLine.replace(/^#+\s+/, "").trim();
            const rest = lines.slice(1);
            while (rest.length > 0 && rest[0].trim() === "") {
                rest.shift();
            }
            body = rest.join("\n").trim();
        }

        if (title !== null && !body.trim()) {
            showStatus("Add some content below the title", "error");
            setLoading(false);
            return;
        }

        await invoke("save_as_note", {
            content: body,
            title: title,
        });
        freeBlobUrls();
        resetCaptureState();
        deferHideCapture();
    } catch (e) {
        console.error("Save as note failed:", e);
        showStatus("✗ " + e.toString(), "error");
        setLoading(false);
    }
}

/**
 * Action: close the capture window after cleanup.
 */
export async function closeCaptureAction({ invoke, freeBlobUrls, resetCaptureState }) {
    freeBlobUrls();
    resetCaptureState();

    try {
        await invoke("hide_capture");
    } catch (e) {
        console.error("Failed to hide window:", e);
    }
}
