import {
    appendToDailyNoteAction,
    saveAsNoteAction,
    closeCaptureAction,
    hideCaptureWhileCollectingAction,
} from "../capture/capture-actions.js";
import { matchesShortcut } from "../capture/capture-utils.js";
import {
    ocrClipboardImage,
    normalizeOcrError,
} from "../ocr/ocrClient.js";
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

        await appendToDailyNoteAction({
            content: content.trim(),
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
            if (textarea) {
                textarea.selectionStart = textarea.selectionEnd = nextCursorPosition;
                textarea.focus();
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

    return {
        deferHideCapture,
        handleAppendToDaily,
        handleSaveAsNote,
        handleClose,
        toggleCaptureMode,
        handleOcrClipboard,
        openSettings,
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
