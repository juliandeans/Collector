<script>
    import { ENABLE_DEMO_FAKE_BG } from "./lib/demo-fake-bg.js";
    const DEMO_FAKE_BG = import.meta.env.DEV && ENABLE_DEMO_FAKE_BG;

    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy, tick } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { defaultSettings } from "./lib/stores.js";
    import { cornerRadiusCss } from "./lib/cornerRadius.js";
    import { setupResizeFlicker } from "./lib/resizeFlicker.js";
    import {
        applyColorSettings,
        loadInitialSettings,
        applySettingsUpdate,
        computeBrightnessFilter,
    } from "./lib/app/settingsBridge.js";
    import {
        setupGlobalDragListeners,
        setupCaptureWindowListeners,
        setupSettingsListener,
        setupTauriDragDrop,
    } from "./lib/app/lifecycleSetup.js";
    import * as dragDropImages from "./lib/app/dragDropImages.js";
    import {
        computeClosedState,
        computeInputState,
        computeInsertion,
    } from "./lib/app/wikiAutocomplete.js";
    import * as appendPicker from "./lib/app/appendPicker.js";
    import AppendToPicker from "./lib/reader/AppendToPicker.svelte";
    import CaptureWikilinkOverlay from "./lib/capture/CaptureWikilinkOverlay.svelte";
    import CaptureActionBar from "./lib/capture/CaptureActionBar.svelte";
    import CaptureModeButton from "./lib/capture/CaptureModeButton.svelte";
    import CaptureStatus from "./lib/capture/CaptureStatus.svelte";
    import CaptureLoadingIndicator from "./lib/capture/CaptureLoadingIndicator.svelte";
    import ResizeHandle from "./lib/ResizeHandle.svelte";
    import CaptureImageGallery from "./lib/capture/CaptureImageGallery.svelte";
    import CaptureAccentLine from "./lib/capture/CaptureAccentLine.svelte";
    import CaptureDropOverlay from "./lib/capture/CaptureDropOverlay.svelte";
    import OcrDropOverlay from "./lib/ocr/OcrDropOverlay.svelte";
    import {
        ocrClipboardImage,
        getModifierFlags,
        isOcrModifierActive,
        normalizeOcrError,
    } from "./lib/ocr/ocrClient.js";
    import { insertTextIntoTextarea } from "./lib/ocr/insertUtils.js";
    import CaptureEditor from "./lib/capture/CaptureEditor.svelte";
    import { filterPaletteNotes } from "./lib/reader/paletteLogic.js";
    import { matchesShortcut } from "./lib/capture/capture-utils.js";
    import {
        appendToDailyNoteAction,
        saveAsNoteAction,
        closeCaptureAction,
        hideCaptureWhileCollectingAction,
    } from "./lib/capture/capture-actions.js";
    import { appendCapturedText } from "./lib/capture/captureTextAppend.js";

    let textareaRef = $state();
    let content = $state("");
    let captureModeActive = $state(false);
    let pendingCaptureContent = $state("");
    let draftSaveTimer;
    let isDragging = $state(false);
    let isLoading = $state(false);
    let isOcrLoading = $state(false);
    let isOcrModifierHeld = $state(false);
    let statusMessage = $state("");
    let statusType = $state("");
    let showAppendPicker = $state(false);
    let appendPickerQuery = $state("");
    let appendPickerNotes = $state([]);
    let appendPickerSelectedIndex = $state(0);
    let appendPickerStep = $state(1);
    let appendPickerSelectedNote = $state(null);
    let appendPickerHeadings = $state([]);
    let appendPickerHeadingIndex = $state(0);
    let appendPickerInputRef = $state();
    let uploadedImages = $state([]);
    let unlistenReset;
    let unlistenShow;
    let unlistenInsertCaptureText;
    let unlistenCaptureTextFailed;
    let unlistenSaveAsNote;
    let unlistenCaptureModeStateChanged;
    let unlistenCaptureModeAppend;
    let unlistenSettingsChanged;
    let unlistenDragDrop;
    let cleanupResizeFlicker;
    let armResize = $state(() => {});
    let isResizing = $state(false);
    let isTauri = false;
    let cleanupGlobalDragListeners;
    let wikiAutocompleteOpen = $state(false);
    let wikiAutocompleteQuery = "";
    let wikiAutocompleteIndex = $state(0);
    let wikiAutocompleteMatches = $state([]);
    let wikiAutocompletePosition = $state({ left: 0, top: 0 });
    let wikiAnchorPos = 0;

    let appSettings = $state({ ...defaultSettings });

    let showNotePaths = $derived(appSettings?.show_note_paths ?? true);
    let autocompleteResults = $derived(appSettings?.autocomplete_results ?? 20);

    let brightnessFilter = $derived(computeBrightnessFilter(appSettings.window_brightness));

    $effect(() => {
        const currentContent = content;
        clearTimeout(draftSaveTimer);

        if (!isTauri || !currentContent.trim()) return;

        draftSaveTimer = setTimeout(() => {
            invoke("save_capture_draft", { content: currentContent }).catch((e) => {
                console.error("Failed to persist capture draft:", e);
            });
        }, 400);
    });

    $effect(() => {
        if (isLoading || !pendingCaptureContent.trim()) return;

        const pending = pendingCaptureContent;
        pendingCaptureContent = "";
        content = appendCapturedText(content, pending);
    });

    function applyAppendPickerState(patch) {
        if (patch.showAppendPicker !== undefined) {
            showAppendPicker = patch.showAppendPicker;
        }
        if (patch.appendPickerStep !== undefined) {
            appendPickerStep = patch.appendPickerStep;
        }
        if (patch.appendPickerSelectedNote !== undefined) {
            appendPickerSelectedNote = patch.appendPickerSelectedNote;
        }
        if (patch.appendPickerHeadings !== undefined) {
            appendPickerHeadings = patch.appendPickerHeadings;
        }
        if (patch.appendPickerQuery !== undefined) {
            appendPickerQuery = patch.appendPickerQuery;
        }
        if (patch.appendPickerSelectedIndex !== undefined) {
            appendPickerSelectedIndex = patch.appendPickerSelectedIndex;
        }
        if (patch.appendPickerHeadingIndex !== undefined) {
            appendPickerHeadingIndex = patch.appendPickerHeadingIndex;
        }
    }

    function closeAppendPicker() {
        applyAppendPickerState(appendPicker.computeClosedPickerState());
        textareaRef?.focus();
    }

    function returnAppendPickerToStep1() {
        applyAppendPickerState(appendPicker.computeStep1PickerState());
    }

    function handleDragDropEvent(event) {
        dragDropImages.handleDragDropEvent(event, {
            setIsDragging: (v) => {
                isDragging = v;
            },
            setDragCounter: (v) => {
                dragCounter = v;
            },
            handleTauriFileDrop,
        });
    }

    function preloadVaultNotes() {
        appendPicker.preloadVaultNotes({
            invoke,
            setNotes: (v) => {
                appendPickerNotes = v;
            },
        });
    }

    /* ── onMount ── */

    onMount(async () => {
        try {
            await getCurrentWindow();
            isTauri = true;
        } catch (e) {
            // Browser preview mode has no Tauri window API.
            isTauri = false;
        }

        cleanupGlobalDragListeners = setupGlobalDragListeners({
            setIsDragging: (v) => {
                isDragging = v;
            },
            getDragCounter: () => dragCounter,
            setDragCounter: (v) => {
                dragCounter = v;
            },
            handleDrop,
        });

        if (isTauri) {
            try {
                await setupCaptureWindowListeners({
                    invoke,
                    getContent: () => content,
                    setContent: (v) => {
                        content = v;
                    },
                    getIsLoading: () => isLoading,
                    getPendingCaptureContent: () => pendingCaptureContent,
                    setPendingCaptureContent: (v) => {
                        pendingCaptureContent = v;
                    },
                    getCaptureModeActive: () => captureModeActive,
                    setCaptureModeActive: (v) => {
                        captureModeActive = v;
                    },
                    setStatusMessage: (v) => {
                        statusMessage = v;
                    },
                    // Required: after the native Tauri capture events, macOS needs a
                    // short delay before the textarea reliably accepts focus.
                    focusTextarea: () => setTimeout(() => textareaRef?.focus(), 50),
                    freeBlobUrls,
                    resetCaptureState,
                    closeAppendPicker,
                    showStatus,
                    handleSaveAsNote,
                    setUnlistenReset: (fn) => {
                        unlistenReset = fn;
                    },
                    setUnlistenShow: (fn) => {
                        unlistenShow = fn;
                    },
                    setUnlistenInsertCaptureText: (fn) => {
                        unlistenInsertCaptureText = fn;
                    },
                    setUnlistenCaptureTextFailed: (fn) => {
                        unlistenCaptureTextFailed = fn;
                    },
                    setUnlistenSaveAsNote: (fn) => {
                        unlistenSaveAsNote = fn;
                    },
                    setUnlistenCaptureModeStateChanged: (fn) => {
                        unlistenCaptureModeStateChanged = fn;
                    },
                    setUnlistenCaptureModeAppend: (fn) => {
                        unlistenCaptureModeAppend = fn;
                    },
                });
                unlistenSettingsChanged = await setupSettingsListener({
                    getAppSettings: () => appSettings,
                    setAppSettings: (v) => {
                        appSettings = v;
                    },
                    applyColorSettings,
                    applySettingsUpdate,
                });
                unlistenDragDrop = await setupTauriDragDrop({ handleDragDropEvent });
                const resizeFlicker = await setupResizeFlicker((value) => {
                    isResizing = value;
                });
                armResize = resizeFlicker.armResize;
                cleanupResizeFlicker = resizeFlicker.cleanup;
                await loadInitialSettings({
                    invoke,
                    setAppSettings: (settings) => {
                        appSettings = settings;
                    },
                });
                captureModeActive = await invoke("get_capture_mode_state");
                preloadVaultNotes();
            } catch (e) {
                console.error("Failed to listen to events:", e);
            }
        }
    });

    onDestroy(() => {
        unlistenReset?.();
        unlistenShow?.();
        unlistenInsertCaptureText?.();
        unlistenCaptureTextFailed?.();
        unlistenSaveAsNote?.();
        unlistenCaptureModeStateChanged?.();
        unlistenCaptureModeAppend?.();
        unlistenSettingsChanged?.();
        unlistenDragDrop?.();
        cleanupResizeFlicker?.();
        cleanupGlobalDragListeners?.();

        freeBlobUrls();
        stopModifierPolling();
    });

    let ocrPollInterval = null;
    function startModifierPolling() {
        if (ocrPollInterval) return;
        const check = async () => {
            if (!isDragging) {
                stopModifierPolling();
                return;
            }
            try {
                const flags = await getModifierFlags({ invoke });
                isOcrModifierHeld = isOcrModifierActive(flags, appSettings?.ocr_drag_modifier);
            } catch {
                isOcrModifierHeld = false;
            }
        };
        check();
        ocrPollInterval = setInterval(check, 80);
    }

    function stopModifierPolling() {
        if (ocrPollInterval) {
            clearInterval(ocrPollInterval);
            ocrPollInterval = null;
        }
        isOcrModifierHeld = false;
    }

    $effect(() => {
        if (isDragging) {
            startModifierPolling();
        } else {
            stopModifierPolling();
        }
    });

    async function handleOcrClipboard() {
        if (isOcrLoading) return;
        isOcrLoading = true;
        showStatus("Extracting text…", "info");
        try {
            const result = await ocrClipboardImage({ invoke });
            if (!result?.text) {
                showStatus("No text recognized", "error");
                return;
            }
            const { nextContent, nextCursorPosition } = insertTextIntoTextarea(
                textareaRef,
                content,
                result.text,
            );
            content = nextContent;
            showStatus("Text extracted", "success");
            await tick();
            if (textareaRef) {
                textareaRef.selectionStart = textareaRef.selectionEnd = nextCursorPosition;
                textareaRef.focus();
            }
        } catch (err) {
            showStatus(normalizeOcrError(err), "error");
        } finally {
            isOcrLoading = false;
        }
    }

    function showStatus(message, type = "success") {
        statusMessage = message;
        statusType = type;
        // Hide the transient status toast after its display period.
        setTimeout(() => (statusMessage = ""), 2000);
    }

    function freeBlobUrls() {
        uploadedImages.forEach((img) => {
            if (img.preview && img.preview.startsWith("blob:")) {
                URL.revokeObjectURL(img.preview);
            }
        });
    }

    function resetCaptureState() {
        content = "";
        uploadedImages = [];
        isDragging = false;
        dragCounter = 0;
        isLoading = false;
    }

    function deferHideCapture() {
        setTimeout(async () => {
            try {
                await invoke("hide_capture");
            } catch (e) {
                console.error("Hide error:", e);
            }
        }, 200);
    }

    function openAppendPicker() {
        if (!content.trim()) return;

        applyAppendPickerState(appendPicker.computeOpenPickerState());
        appendPickerInputRef?.focus();

        appendPicker.loadVaultNotesIfEmpty({
            invoke,
            getNotes: () => appendPickerNotes,
            setNotes: (v) => {
                appendPickerNotes = v;
            },
            showStatus,
        });
    }

    async function handleAppendToDaily() {
        if (!content.trim() || isLoading) {
            if (!content.trim()) {
                showStatus("Nothing to append", "error");
            }
            return;
        }

        isLoading = true;

        await appendToDailyNoteAction({
            content: content.trim(),
            invoke,
            showStatus,
            freeBlobUrls,
            resetCaptureState,
            deferHideCapture,
            setLoading: (v) => {
                isLoading = v;
            },
        });
    }

    async function handleClose() {
        if (captureModeActive) {
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
            captureModeActive = await invoke("toggle_capture_mode");
        } catch (e) {
            showStatus("✗ " + e.toString(), "error");
        }
    }

    async function handleSaveAsNote() {
        if (!content.trim() || isLoading) return;

        isLoading = true;

        await saveAsNoteAction({
            content,
            invoke,
            showStatus,
            freeBlobUrls,
            resetCaptureState,
            deferHideCapture,
            setLoading: (v) => {
                isLoading = v;
            },
        });
    }

    async function handleNoteSelected(note) {
        await appendPicker.selectNote(note, {
            invoke,
            getSelectedNote: () => appendPickerSelectedNote,
            setSelectedNote: (v) => {
                appendPickerSelectedNote = v;
            },
            setHeadings: (v) => {
                appendPickerHeadings = v;
            },
            setHeadingIndex: (v) => {
                appendPickerHeadingIndex = v;
            },
            setStep: (v) => {
                appendPickerStep = v;
            },
        });
    }

    async function handleAppendToNote(note, heading = null) {
        closeAppendPicker();

        await appendPicker.appendToNote(note, heading, {
            invoke,
            getContent: () => content,
            getIsLoading: () => isLoading,
            setLoading: (v) => {
                isLoading = v;
            },
            getEntryHeaderTemplate: () => appSettings.entry_header,
            freeBlobUrls,
            resetCaptureState,
            deferHideCapture,
            showStatus,
        });
    }

    async function handleKeydown(e) {
        if (wikiAutocompleteOpen) {
            if (e.key === "ArrowDown") {
                e.preventDefault();
                wikiAutocompleteIndex = Math.min(
                    wikiAutocompleteIndex + 1,
                    wikiAutocompleteMatches.length - 1,
                );
                return;
            }
            if (e.key === "ArrowUp") {
                e.preventDefault();
                wikiAutocompleteIndex = Math.max(wikiAutocompleteIndex - 1, 0);
                return;
            }
            if (e.key === "Enter" || e.key === "Tab") {
                e.preventDefault();
                if (wikiAutocompleteMatches[wikiAutocompleteIndex]) {
                    insertWikilink(
                        wikiAutocompleteMatches[wikiAutocompleteIndex],
                    );
                }
                return;
            }
            if (e.key === "Escape") {
                e.preventDefault();
                closeWikiAutocomplete();
                return;
            }
        }

        if (e.metaKey && e.key === ",") {
            e.preventDefault();
            openSettings();
            return;
        }

        if (matchesShortcut(e, appSettings.ocr_shortcut)) {
            e.preventDefault();
            handleOcrClipboard();
            return;
        }

        if (matchesShortcut(e, appSettings.append_to_note_shortcut)) {
            e.preventDefault();
            openAppendPicker();
            return;
        }

        if (matchesShortcut(e, appSettings.save_to_daily_shortcut)) {
            e.preventDefault();
            handleAppendToDaily();
            return;
        }

        if (matchesShortcut(e, appSettings.save_as_note_shortcut)) {
            e.preventDefault();
            handleSaveAsNote();
            return;
        }

        if (e.key === "Escape") {
            e.preventDefault();
            handleClose();
            return;
        }
    }

    function applyWikiState({ open, query, matches, index, position, anchorPos }) {
        wikiAutocompleteOpen = open;
        wikiAutocompleteQuery = query;
        wikiAutocompleteMatches = matches;
        wikiAutocompleteIndex = index;
        wikiAutocompletePosition = position;
        wikiAnchorPos = anchorPos;
    }

    function handleWikiInput() {
        const state = computeInputState({
            textarea: textareaRef,
            notes: appendPickerNotes,
            limit: autocompleteResults,
        });
        if (state === null) return;
        applyWikiState(state);
    }

    function insertWikilink(note) {
        const newContent = computeInsertion({
            textarea: textareaRef,
            query: wikiAutocompleteQuery,
            note,
        });
        if (newContent === null) return;

        content = newContent;
        applyWikiState(computeClosedState());
    }

    function closeWikiAutocomplete() {
        applyWikiState(computeClosedState());
    }

    async function openSettings() {
        try {
            await invoke("open_settings");
        } catch (e) {
            console.error("Failed to open settings:", e);
        }
    }

    let dragCounter = 0;

    function handleDragEnter(e) {
        dragDropImages.handleDragEnter(e, {
            getDragCounter: () => dragCounter,
            setDragCounter: (v) => {
                dragCounter = v;
            },
            setIsDragging: (v) => {
                isDragging = v;
            },
        });
    }

    function handleDragLeave(e) {
        dragDropImages.handleDragLeave(e, {
            getDragCounter: () => dragCounter,
            setDragCounter: (v) => {
                dragCounter = v;
            },
            setIsDragging: (v) => {
                isDragging = v;
            },
        });
    }

    function handleDragOver(e) {
        dragDropImages.handleDragOver(e, {
            getIsDragging: () => isDragging,
            setIsDragging: (v) => {
                isDragging = v;
            },
        });
    }

    async function handleDrop(e) {
        await dragDropImages.handleDrop(e, {
            invoke,
            getIsTauri: () => isTauri,
            getTextarea: () => textareaRef,
            getContent: () => content,
            setContent: (v) => {
                content = v;
            },
            updateImages: (fn) => {
                uploadedImages = fn(uploadedImages);
            },
            setIsDragging: (v) => {
                isDragging = v;
            },
            setDragCounter: (v) => {
                dragCounter = v;
            },
            showStatus,
            tick,
            getAppSettings: () => appSettings,
            getIsOcrLoading: () => isOcrLoading,
            setIsOcrLoading: (v) => {
                isOcrLoading = v;
            },
        });
    }

    async function handleTauriFileDrop(paths) {
        await dragDropImages.handleTauriFileDrop(paths, {
            invoke,
            getContent: () => content,
            setContent: (v) => {
                content = v;
            },
            updateImages: (fn) => {
                uploadedImages = fn(uploadedImages);
            },
            getTextarea: () => textareaRef,
            setIsDragging: (v) => {
                isDragging = v;
            },
            setDragCounter: (v) => {
                dragCounter = v;
            },
            showStatus,
            getAppSettings: () => appSettings,
            tick,
            getIsOcrLoading: () => isOcrLoading,
            setIsOcrLoading: (v) => {
                isOcrLoading = v;
            },
        });
    }

    function removeImage(id) {
        dragDropImages.removeImage(id, {
            getImages: () => uploadedImages,
            updateImages: (fn) => {
                uploadedImages = fn(uploadedImages);
            },
        });
    }
</script>

<div
    class="capture-container"
    class:demo-fake-bg={DEMO_FAKE_BG}
    class:dragging={isDragging}
    class:resizing={isResizing}
    class:append-picker-open={showAppendPicker}
    style="
    --app-background: {appSettings.overlay_color};
    --app-font-family: {appSettings.font_family};
    --app-font-size: {appSettings.font_size}px;
    --app-border-radius: {appSettings.border_radius}px;
    --app-corner-radius: {cornerRadiusCss(appSettings.border_radius, appSettings.edge_side, appSettings.round_all_corners)};
    --app-transparency: {appSettings.overlay_strength}%;
    --app-blur: {appSettings.window_blur}px;
    --app-saturation: {appSettings.window_saturation}%;
    --app-text-color: {appSettings.text_color};
    --app-brightness-filter: {brightnessFilter};
  "
    ondragenter={handleDragEnter}
    ondragleave={handleDragLeave}
    ondragover={handleDragOver}
    ondrop={handleDrop}
    role="application"
>
    <CaptureAccentLine />

    <div
        class="content-wrapper"
        role="presentation"
        ondrop={(e) => {
            handleDrop(e);
        }}
        ondragover={(e) => {
            e.preventDefault();
            handleDragOver(e);
        }}
        ondragenter={(e) => {
            handleDragEnter(e);
        }}
    >
        <CaptureImageGallery
            images={uploadedImages}
            onRemove={(id) => removeImage(id)}
            onDrop={(e) => handleDrop(e)}
            onDragover={(e) => {
                e.preventDefault();
                handleDragOver(e);
            }}
            onDragenter={(e) => handleDragEnter(e)}
        />

        <CaptureEditor
            bind:content
            bind:textareaRef
            {isLoading}
            onKeydown={handleKeydown}
            onInput={handleWikiInput}
            onBlur={closeWikiAutocomplete}
            onDrop={handleDrop}
            onDragover={handleDragOver}
            onDragenter={handleDragEnter}
        />
    </div>

    <CaptureActionBar
        show={appSettings.show_capture_action_bar}
        {isLoading}
        saveToDailyShortcut={appSettings.save_to_daily_shortcut}
        saveAsNoteShortcut={appSettings.save_as_note_shortcut}
        appendToNoteShortcut={appSettings.append_to_note_shortcut}
        onDaily={handleAppendToDaily}
        onNewNote={handleSaveAsNote}
        onAppend={openAppendPicker}
    />

    <CaptureModeButton
        show={appSettings.show_capture_mode_button}
        active={captureModeActive}
        shortcut={appSettings.capture_mode_shortcut}
        onToggle={toggleCaptureMode}
    />

    <ResizeHandle onResizeStart={armResize} />

    <CaptureDropOverlay show={isDragging && !isOcrModifierHeld} />
    <OcrDropOverlay show={isDragging && isOcrModifierHeld} isOcrActive={true} />

    <CaptureStatus message={statusMessage} type={statusType} />

    <CaptureWikilinkOverlay
        open={wikiAutocompleteOpen}
        matches={wikiAutocompleteMatches}
        selectedIndex={wikiAutocompleteIndex}
        position={wikiAutocompletePosition}
        showPaths={showNotePaths}
        onSelect={(note) => insertWikilink(note)}
        onHover={(index) => {
            wikiAutocompleteIndex = index;
        }}
    />

    <AppendToPicker
        open={showAppendPicker}
        step={appendPickerStep}
        query={appendPickerQuery}
        notes={filterPaletteNotes(
            appendPickerNotes,
            appendPickerQuery,
            autocompleteResults,
        )}
        selectedIndex={appendPickerSelectedIndex}
        selectedNote={appendPickerSelectedNote}
        headings={appendPickerHeadings}
        headingIndex={appendPickerHeadingIndex}
        showPaths={showNotePaths}
        bind:inputRef={appendPickerInputRef}
        on:queryChange={(e) => {
            appendPickerQuery = e.detail.target.value;
            appendPickerSelectedIndex = 0;
        }}
        on:selectNote={(e) => handleNoteSelected(e.detail)}
        on:selectHeading={(e) =>
            handleAppendToNote(appendPickerSelectedNote, e.detail)}
        on:selectIndex={(e) => {
            appendPickerSelectedIndex = e.detail;
        }}
        on:headingIndexChange={(e) => {
            appendPickerHeadingIndex = e.detail;
        }}
        on:backToStep1={() => {
            returnAppendPickerToStep1();
        }}
        on:close={closeAppendPicker}
    />

    <CaptureLoadingIndicator active={isLoading} />
</div>

<style>
    :global(*) {
        box-sizing: border-box;
        margin: 0;
        padding: 0;
    }

    :global(body) {
        margin: 0;
        padding: 0;
        overflow: hidden;
        background: transparent;
    }

    .capture-container {
        position: fixed;
        top: 0;
        left: 0;
        width: 100%;
        height: 100%;
        background: color-mix(
            in srgb,
            var(--app-background, #1e1e2e) var(--app-transparency, 55%),
            transparent
        );
        backdrop-filter: saturate(var(--app-saturation, 200%))
            var(--app-brightness-filter);
        -webkit-backdrop-filter: saturate(var(--app-saturation, 200%))
            var(--app-brightness-filter);
        border-radius: var(--app-corner-radius, var(--app-border-radius, 12px));
        border: 0.5px solid rgba(0, 0, 0, 0.08);
        box-shadow:
            var(--shadow-md),
            0 2px 8px var(--shadow-sm);
        overflow: clip;
        display: flex;
        flex-direction: column;
        font-family: var(
            --app-font-family,
            -apple-system,
            BlinkMacSystemFont,
            "SF Pro Display",
            sans-serif
        );
        transform: translateZ(0);
        -webkit-transform: translateZ(0);
    }

    .capture-container.dragging {
        background: rgba(255, 255, 255, 0.7);
        border-color: rgba(255, 255, 255, 0.7);
        border-width: 2px;
        box-shadow:
            var(--shadow-md),
            0 2px 8px var(--shadow-sm);
    }

    /* Effects for the JavaScript resize fallback. The native macOS handler
       leaves these effects unchanged. */
    .capture-container.resizing {
        backdrop-filter: none;
        -webkit-backdrop-filter: none;
        box-shadow: none;
    }

    .content-wrapper {
        transition:
            filter 0.12s ease,
            opacity 0.12s ease,
            transform 0.12s ease;
    }

    .capture-container.append-picker-open :global(.accent-line),
    .capture-container.append-picker-open .content-wrapper,
    .capture-container.append-picker-open :global(.action-bar),
    .capture-container.append-picker-open :global(.status-toast),
    .capture-container.append-picker-open :global(.resize-handle),
    .capture-container.append-picker-open :global(.capture-mode-button) {
        filter: blur(4px) brightness(0.62);
        opacity: 0.56;
        transform: scale(0.997);
        pointer-events: none;
    }

    .content-wrapper {
        flex: 1;
        display: flex;
        flex-direction: column;
        overflow: hidden;
    }

    .capture-container:hover :global(.resize-handle) {
        opacity: 0.6;
    }

    .capture-container.demo-fake-bg {
        background:
            radial-gradient(
                circle at 18% 10%,
                rgba(96, 165, 250, 0.13),
                transparent 40%
            ),
            radial-gradient(
                circle at 78% 18%,
                rgba(139, 92, 246, 0.1),
                transparent 44%
            ),
            radial-gradient(
                circle at 52% 88%,
                rgba(59, 130, 246, 0.1),
                transparent 54%
            ),
            linear-gradient(
                145deg,
                rgba(25, 40, 54, 0.96),
                rgba(31, 38, 55, 0.96) 48%,
                rgba(24, 48, 52, 0.97)
            );

        -webkit-backdrop-filter: none;
        backdrop-filter: none;
    }
</style>
