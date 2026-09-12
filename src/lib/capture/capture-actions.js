import {
    actionHandlers,
    describeRuleTarget,
    evaluateCaptureRules,
    stripTriggers,
} from "./rules.js";

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
    rules = [],
}) {
    try {
        const ruleMatch = evaluateCaptureRules(content, rules);
        const action = ruleMatch.matched
            ? ruleMatch.actions.find(
                  (candidate) =>
                      typeof candidate?.type === "string" &&
                      Object.hasOwn(actionHandlers, candidate.type),
              )
            : null;

        if (ruleMatch.matched && action?.type === "append") {
            const text =
                action.stripTrigger === false
                    ? content
                    : stripTriggers(content, ruleMatch.rule.conditions);

            await invoke("append_to_note_section", {
                path: action.note ?? "",
                heading: action.heading ?? "",
                position: action.position ?? "bottom",
                createHeadingIfMissing: action.createHeadingIfMissing ?? false,
                showTimestamp: action.showTimestamp ?? true,
                text,
            });
            try {
                await invoke("clear_capture_draft");
            } catch (e) {
                console.error("Failed to clear capture draft:", e);
            }
            freeBlobUrls();
            resetCaptureState();
            showStatus(`→ ${describeRuleTarget(ruleMatch.rule)} ✓`, "success");
            deferHideCapture();
            return;
        }

        await invoke("append_to_daily_note", { text: content });
        try {
            await invoke("clear_capture_draft");
        } catch (e) {
            console.error("Failed to clear capture draft:", e);
        }
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
        try {
            await invoke("clear_capture_draft");
        } catch (e) {
            console.error("Failed to clear capture draft:", e);
        }
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
 * Action: close the capture window after cleanup, flushing a draft of the
 * current content first if there is any (draft persistence is a no-op
 * server-side when the user hasn't enabled it in Settings). This is a
 * same-tick top-up of the debounced autosave in App.svelte — it catches
 * the last <400ms of typing that the debounce timer hasn't flushed yet.
 */
export async function closeCaptureAction({
    content,
    invoke,
    freeBlobUrls,
    resetCaptureState,
}) {
    try {
        if (content && content.trim()) {
            await invoke("save_capture_draft", { content });
        } else {
            await invoke("clear_capture_draft");
        }
    } catch (e) {
        console.error("Failed to persist capture draft:", e);
    }

    freeBlobUrls();
    resetCaptureState();

    try {
        await invoke("hide_capture");
    } catch (e) {
        console.error("Failed to hide window:", e);
    }
}

/**
 * Hide the capture window without clearing its in-memory buffer while the
 * backend-owned capture mode continues collecting clipboard text.
 */
export async function hideCaptureWhileCollectingAction({ content, invoke }) {
    try {
        if (content && content.trim()) {
            await invoke("save_capture_draft", { content });
        } else {
            await invoke("clear_capture_draft");
        }
    } catch (e) {
        console.error("Failed to persist capture draft:", e);
    }

    try {
        await invoke("hide_capture");
    } catch (e) {
        console.error("Failed to hide window:", e);
    }
}
