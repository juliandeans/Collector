import {
    appendToDailyNoteAction,
    saveAsNoteAction,
    closeCaptureAction,
    hideCaptureWhileCollectingAction,
} from "../capture/capture-actions.js";
import { matchesShortcut } from "../capture/capture-utils.js";
import { ocrClipboardImage, normalizeOcrError } from "../ocr/ocrClient.js";
import { insertTextIntoTextarea } from "../ocr/insertUtils.js";

export function createCaptureActions({
    invoke,
    getContent,
    setContent,
    getIsLoading,
    setIsLoading,
    getIsOcrLoading,
    setIsOcrLoading,
    getCaptureModeActive,
    setCaptureModeActive,
    showStatus,
    freeBlobUrls,
    resetCaptureState,
    getTextarea,
    tick,
    getAppSettings,
    setAppSettings,
}) {
    function deferHideCapture() {
        setTimeout(async () => {
            try {
                await invoke("hide_capture");
            } catch (e) {
                console.error("Hide error:", e);
            }
        }, 200);
    }

    async function handleAppendToDaily() {
        const content = getContent();
        if (!content.trim() || getIsLoading()) {
            if (!content.trim()) {
                showStatus("Nothing to append", "error");
            }
            return;
        }

        setIsLoading(true);

        const appSettings = getAppSettings?.() ?? {};
        await appendToDailyNoteAction({
            content: content.trim(),
            rules: appSettings.capture_rules,
            invoke,
            showStatus,
            freeBlobUrls,
            resetCaptureState,
            deferHideCapture,
            setLoading: setIsLoading,
        });
    }

    async function handleSaveAsNote() {
        const content = getContent();
        if (!content.trim() || getIsLoading()) return;

        setIsLoading(true);

        await saveAsNoteAction({
            content,
            invoke,
            showStatus,
            freeBlobUrls,
            resetCaptureState,
            deferHideCapture,
            setLoading: setIsLoading,
        });
    }

    async function handleClose() {
        const content = getContent();
        if (getCaptureModeActive()) {
            await hideCaptureWhileCollectingAction({
                content,
                invoke,
            });
            return;
        }

        await closeCaptureAction({
            content,
            invoke,
            freeBlobUrls,
            resetCaptureState,
        });
    }

    async function toggleCaptureMode() {
        try {
            const nextState = await invoke("toggle_capture_mode");
            setCaptureModeActive(nextState);
        } catch (e) {
            showStatus("✗ " + e.toString(), "error");
        }
    }

    async function handleOcrClipboard() {
        if (getIsOcrLoading()) return;
        setIsOcrLoading(true);
        showStatus("Extracting text…", "info");
        try {
            const result = await ocrClipboardImage({ invoke });
            if (!result?.text) {
                showStatus("No text recognized", "error");
                return;
            }
            const textarea = getTextarea();
            const { nextContent, nextCursorPosition } = insertTextIntoTextarea(
                textarea,
                getContent(),
                result.text,
            );
            setContent(nextContent);
            showStatus("Text extracted", "success");
            await tick();
            const currentTextarea = getTextarea();
            if (currentTextarea) {
                currentTextarea.selectionStart = currentTextarea.selectionEnd =
                    nextCursorPosition;
                currentTextarea.focus();
            }
        } catch (err) {
            showStatus(normalizeOcrError(err), "error");
        } finally {
            setIsOcrLoading(false);
        }
    }

    async function openSettings() {
        try {
            await invoke("open_settings");
        } catch (e) {
            console.error("Failed to open settings:", e);
        }
    }

    async function handleClearCapture() {
        if (getIsLoading() || getIsOcrLoading()) return;

        freeBlobUrls();
        resetCaptureState();
        let draftCleared = true;
        try {
            await invoke("clear_capture_draft");
        } catch (e) {
            console.error("Failed to clear capture draft:", e);
            draftCleared = false;
        }
        await tick();
        getTextarea()?.focus();
        if (draftCleared) {
            showStatus("Capture cleared", "success", 1800);
        } else {
            showStatus(
                "Capture cleared, but the saved draft could not be removed",
                "error",
            );
        }
    }

    let isTogglingDraft = false;

    async function handleToggleDraftPersistence() {
        if (!getAppSettings || !setAppSettings || isTogglingDraft) return;
        isTogglingDraft = true;

        try {
            const currentSettings = getAppSettings();
            const nextState = !currentSettings.persist_capture_draft;
            const nextSettings = {
                ...currentSettings,
                persist_capture_draft: nextState,
            };

            // 1. Save settings to backend first so the backend gate in save_capture_draft is open
            try {
                await invoke("save_settings", { newSettings: nextSettings });
            } catch (e) {
                console.error("Failed to save draft persistence setting:", e);
                showStatus(
                    "Failed to update setting: " + e.toString(),
                    "error",
                );
                return;
            }

            // 2. Update local state
            setAppSettings(nextSettings);

            // 3. Save or clear draft according to the new state
            if (nextState) {
                const content = getContent();
                if (content.trim()) {
                    await invoke("save_capture_draft", { content }).catch(
                        (e) => {
                            console.error(
                                "Failed to persist capture draft:",
                                e,
                            );
                        },
                    );
                }
                showStatus("Keep unsaved text: ON", "info", 1800);
            } else {
                await invoke("clear_capture_draft").catch((e) => {
                    console.error("Failed to clear capture draft:", e);
                });
                showStatus("Keep unsaved text: OFF", "info", 1800);
            }
        } finally {
            isTogglingDraft = false;
        }
    }

    return {
        deferHideCapture,
        handleAppendToDaily,
        handleSaveAsNote,
        handleClose,
        toggleCaptureMode,
        handleOcrClipboard,
        openSettings,
        handleToggleDraftPersistence,
        handleClearCapture,
    };
}

export function handleAppShortcut({
    event,
    appSettings,
    actions,
    openAppendPicker,
}) {
    if (event.metaKey && event.key === ",") {
        event.preventDefault();
        actions.openSettings();
        return true;
    }

    if (matchesShortcut(event, appSettings.ocr_shortcut)) {
        event.preventDefault();
        actions.handleOcrClipboard();
        return true;
    }

    if (matchesShortcut(event, appSettings.persist_capture_draft_shortcut)) {
        event.preventDefault();
        actions.handleToggleDraftPersistence();
        return true;
    }

    if (matchesShortcut(event, appSettings.clear_capture_shortcut)) {
        event.preventDefault();
        actions.handleClearCapture();
        return true;
    }

    if (matchesShortcut(event, appSettings.append_to_note_shortcut)) {
        event.preventDefault();
        openAppendPicker();
        return true;
    }

    if (matchesShortcut(event, appSettings.save_to_daily_shortcut)) {
        event.preventDefault();
        actions.handleAppendToDaily();
        return true;
    }

    if (matchesShortcut(event, appSettings.save_as_note_shortcut)) {
        event.preventDefault();
        actions.handleSaveAsNote();
        return true;
    }

    if (event.key === "Escape") {
        event.preventDefault();
        actions.handleClose();
        return true;
    }

    return false;
}
