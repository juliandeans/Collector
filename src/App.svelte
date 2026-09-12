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
        handleWikiKeydown,
    } from "./lib/app/wikiAutocomplete.js";
    import {
        createCaptureActions,
        handleAppShortcut,
    } from "./lib/app/captureActions.js";
    import { createModifierWatcher } from "./lib/ocr/modifierWatcher.js";
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
    import CaptureEditor from "./lib/capture/CaptureEditor.svelte";
    import { filterPaletteNotes } from "./lib/reader/paletteLogic.js";
    import { appendCapturedText } from "./lib/capture/captureTextAppend.js";

    import "./lib/capture/capture.css";

    let textareaRef = $state();
    let content = $state("");
    let captureModeActive = $state(false);
    let pendingCaptureContent = $state("");
    let draftSaveTimer;
    let isDragging = $state(false);
    let dragCounter = 0;
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

    const actions = createCaptureActions({
        invoke,
        getContent: () => content,
        setContent: (v) => {
            content = v;
        },
        getIsLoading: () => isLoading,
        setIsLoading: (v) => {
            isLoading = v;
        },
        getIsOcrLoading: () => isOcrLoading,
        setIsOcrLoading: (v) => {
            isOcrLoading = v;
        },
        getCaptureModeActive: () => captureModeActive,
        setCaptureModeActive: (v) => {
            captureModeActive = v;
        },
        showStatus,
        freeBlobUrls,
        resetCaptureState,
        getTextarea: () => textareaRef,
        tick,
    });

    const modifierWatcher = createModifierWatcher({
        invoke,
        getIsDragging: () => isDragging,
        getModifierSetting: () => appSettings?.ocr_drag_modifier,
        onActiveChange: (active) => {
            isOcrModifierHeld = active;
        },
    });

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

    $effect(() => {
        if (isDragging) {
            modifierWatcher.start();
        } else {
            modifierWatcher.stop();
        }
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
                    handleSaveAsNote: actions.handleSaveAsNote,
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
        modifierWatcher.stop();
    });

    function showStatus(message, type = "success") {
        statusMessage = message;
        statusType = type;
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
            deferHideCapture: actions.deferHideCapture,
            showStatus,
        });
    }

    async function handleKeydown(e) {
        if (showAppendPicker) return;

        if (
            handleWikiKeydown({
                event: e,
                isOpen: wikiAutocompleteOpen,
                currentIndex: wikiAutocompleteIndex,
                matches: wikiAutocompleteMatches,
                onSelect: insertWikilink,
                onClose: closeWikiAutocomplete,
                setIndex: (i) => {
                    wikiAutocompleteIndex = i;
                },
            })
        ) {
            return;
        }

        handleAppShortcut({
            event: e,
            appSettings,
            actions,
            openAppendPicker,
        });
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

<svelte:window onkeydown={handleKeydown} />

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
            onInput={handleWikiInput}
            onBlur={closeWikiAutocomplete}
            onDrop={handleDrop}
            onDragover={handleDragOver}
            onDragenter={handleDragEnter}
        />
    </div>

    <div class="capture-actions-row">
        <CaptureActionBar
            show={appSettings.show_capture_action_bar}
            {isLoading}
            saveToDailyShortcut={appSettings.save_to_daily_shortcut}
            saveAsNoteShortcut={appSettings.save_as_note_shortcut}
            appendToNoteShortcut={appSettings.append_to_note_shortcut}
            onDaily={actions.handleAppendToDaily}
            onNewNote={actions.handleSaveAsNote}
            onAppend={openAppendPicker}
        />

        <CaptureModeButton
            show={appSettings.show_capture_mode_button}
            active={captureModeActive}
            shortcut={appSettings.capture_mode_shortcut}
            onToggle={actions.toggleCaptureMode}
        />
    </div>

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
