import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { isFileDrag } from "../capture/capture-utils.js";

export function setupGlobalDragListeners({
    setIsDragging,
    getDragCounter,
    setDragCounter,
    handleDrop,
}) {
    const globalDragEnter = (e) => {
        if (!isFileDrag(e)) return;
        e.preventDefault();
        setDragCounter(Math.max(getDragCounter(), 1));
        setIsDragging(true);
    };

    const globalDragOver = (e) => {
        e.preventDefault();
        if (isFileDrag(e)) {
            e.dataTransfer.dropEffect = "copy";
        }
    };

    const globalDragLeave = (e) => {
        if (e.target === document.documentElement) {
            setDragCounter(0);
            setIsDragging(false);
        }
    };

    const globalDrop = (e) => {
        const container = e.target?.closest?.(".capture-container");
        if (container) {
            e.preventDefault();
            e.stopPropagation();
            handleDrop(e);
        }
    };

    document.addEventListener("dragenter", globalDragEnter, true);
    document.addEventListener("dragover", globalDragOver, true);
    document.addEventListener("dragleave", globalDragLeave, true);
    document.addEventListener("drop", globalDrop, true);

    return () => {
        document.removeEventListener("dragenter", globalDragEnter, true);
        document.removeEventListener("dragover", globalDragOver, true);
        document.removeEventListener("dragleave", globalDragLeave, true);
        document.removeEventListener("drop", globalDrop, true);
    };
}

export async function setupCaptureWindowListeners({
    getContent,
    setContent,
    setStatusMessage,
    focusTextarea,
    freeBlobUrls,
    resetCaptureState,
    closeAppendPicker,
    showStatus,
    handleSaveAsNote,
    setUnlistenReset,
    setUnlistenShow,
    setUnlistenInsertCaptureText,
    setUnlistenCaptureTextFailed,
    setUnlistenSaveAsNote,
}) {
    // reset_capture: clear all capture state for a fresh start.
    // Emitted before show_capture when the user wants a clean slate
    // (global shortcut, edge detection, tray).
    // NOT emitted by capture-text (Cmd+Shift+C) — that must append.
    setUnlistenReset(
        await listen("reset_capture", () => {
            freeBlobUrls();
            resetCaptureState();
            setStatusMessage("");
            closeAppendPicker();
        }),
    );

    // Just show and focus — no reset, so existing content survives.
    setUnlistenShow(
        await listen("show_capture", () => {
            focusTextarea();
        }),
    );

    setUnlistenInsertCaptureText(
        await listen("insert_capture_text", (event) => {
            const text = typeof event.payload === "string" ? event.payload : "";
            if (!text.trim()) return;
            const content = getContent();
            // Append captured text to existing content instead of replacing it.
            // If content is empty, set directly; otherwise add spacing then append.
            if (!content.trim()) {
                setContent(text);
            } else {
                // Add spacing: if content doesn't end with a blank line, insert two newlines.
                // If it already ends with a blank line, insert one newline.
                const endsWithBlank = /\n\s*\n$/.test(content);
                setContent(content + (endsWithBlank ? "\n" : "\n\n") + text);
            }
            focusTextarea();
        }),
    );

    setUnlistenCaptureTextFailed(
        await listen("capture_text_failed", (event) => {
            const msg =
                typeof event.payload === "string"
                    ? event.payload
                    : "Nothing to capture";
            showStatus("✗ " + msg, "error");
        }),
    );

    setUnlistenSaveAsNote(
        await listen("save_as_note", () => {
            handleSaveAsNote();
        }),
    );
}

export async function setupSettingsListener({
    getAppSettings,
    setAppSettings,
    applyColorSettings,
    applySettingsUpdate,
}) {
    return await listen("settings_changed", (event) => {
        const merged = applySettingsUpdate(getAppSettings(), event.payload);
        setAppSettings(merged);
        applyColorSettings(merged);
    });
}

export async function setupTauriDragDrop({ handleDragDropEvent }) {
    const currentWindow = await getCurrentWindow();
    return await currentWindow.onDragDropEvent(handleDragDropEvent);
}
