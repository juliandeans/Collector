<script>
    import { ENABLE_DEMO_FAKE_BG } from "./lib/demo-fake-bg.js";
    const DEMO_FAKE_BG = import.meta.env.DEV && ENABLE_DEMO_FAKE_BG;

    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy, tick } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { defaultSettings } from "./lib/stores.js";
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
    import CaptureStatus from "./lib/capture/CaptureStatus.svelte";
    import CaptureLoadingIndicator from "./lib/capture/CaptureLoadingIndicator.svelte";
    import CaptureResizeHandle from "./lib/capture/CaptureResizeHandle.svelte";
    import CaptureImageGallery from "./lib/capture/CaptureImageGallery.svelte";
    import CaptureAccentLine from "./lib/capture/CaptureAccentLine.svelte";
    import CaptureDropOverlay from "./lib/capture/CaptureDropOverlay.svelte";
    import CaptureEditor from "./lib/capture/CaptureEditor.svelte";
    import { filterPaletteNotes } from "./lib/reader/paletteLogic.js";
    import { matchesShortcut } from "./lib/capture/capture-utils.js";
    import {
        appendToDailyNoteAction,
        saveAsNoteAction,
        closeCaptureAction,
    } from "./lib/capture/capture-actions.js";

    let textareaRef = $state();
    let content = $state("");
    let isDragging = $state(false);
    let isLoading = $state(false);
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
    let unlistenSettingsChanged;
    let unlistenDragDrop;
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
                    getContent: () => content,
                    setContent: (v) => {
                        content = v;
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
                await loadInitialSettings({
                    invoke,
                    setAppSettings: (settings) => {
                        appSettings = settings;
                    },
                });
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
        unlistenSettingsChanged?.();
        unlistenDragDrop?.();
        cleanupGlobalDragListeners?.();

        freeBlobUrls();
    });

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
        await closeCaptureAction({
            invoke,
            freeBlobUrls,
            resetCaptureState,
        });
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
    class:append-picker-open={showAppendPicker}
    style="
    --app-background: {appSettings.overlay_color};
    --app-font-family: {appSettings.font_family};
    --app-font-size: {appSettings.font_size}px;
    --app-border-radius: {appSettings.border_radius}px;
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
            on:remove={(event) => removeImage(event.detail)}
            on:drop={(e) => handleDrop(e)}
            on:dragover={(e) => {
                e.preventDefault();
                handleDragOver(e);
            }}
            on:dragenter={(e) => handleDragEnter(e)}
        />

        <CaptureEditor
            bind:content
            bind:textareaRef
            {isLoading}
            on:keydown={handleKeydown}
            on:input={handleWikiInput}
            on:blur={closeWikiAutocomplete}
            on:drop={handleDrop}
            on:dragover={handleDragOver}
            on:dragenter={handleDragEnter}
        />
    </div>

    <CaptureActionBar
        show={appSettings.show_capture_action_bar}
        {isLoading}
        saveToDailyShortcut={appSettings.save_to_daily_shortcut}
        saveAsNoteShortcut={appSettings.save_as_note_shortcut}
        appendToNoteShortcut={appSettings.append_to_note_shortcut}
        on:daily={handleAppendToDaily}
        on:newNote={handleSaveAsNote}
        on:append={openAppendPicker}
    />

    <CaptureResizeHandle />

    <CaptureDropOverlay show={isDragging} />

    <CaptureStatus message={statusMessage} type={statusType} />

    <CaptureWikilinkOverlay
        open={wikiAutocompleteOpen}
        matches={wikiAutocompleteMatches}
        selectedIndex={wikiAutocompleteIndex}
        position={wikiAutocompletePosition}
        showPaths={showNotePaths}
        on:select={(event) => insertWikilink(event.detail)}
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
        backdrop-filter: blur(var(--app-blur, 80px))
            saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
        -webkit-backdrop-filter: blur(var(--app-blur, 80px))
            saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
        border-radius: var(--app-border-radius, 12px);
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
    .capture-container.append-picker-open :global(.resize-handle) {
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
