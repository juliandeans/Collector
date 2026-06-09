<script>
    import { ENABLE_DEMO_FAKE_BG } from "./lib/demo-fake-bg.js";
    const DEMO_FAKE_BG = import.meta.env.DEV && ENABLE_DEMO_FAKE_BG;

    import { invoke } from "@tauri-apps/api/core";
    import { listen } from "@tauri-apps/api/event";
    import { onMount, onDestroy, tick } from "svelte";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { defaultSettings } from "./lib/stores.js";
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
    import { getAutocompleteResults } from "./lib/reader/autocomplete.js";
    import {
        isFileDrag,
        normalizeFilePath,
        normalizeImageResult,
        parseHeadings,
        formatEntryHeader,
        getVaultNotePath,
        matchesShortcut,
    } from "./lib/capture/capture-utils.js";
    import {
        findWikiTrigger,
        computeWikilinkInsertion,
    } from "./lib/capture/wiki-utils.js";
    import {
        appendToDailyNoteAction,
        saveAsNoteAction,
        closeCaptureAction,
    } from "./lib/capture/capture-actions.js";

    let textareaRef;
    let content = "";
    let isDragging = false;
    let isLoading = false;
    let statusMessage = "";
    let statusType = "";
    let showAppendPicker = false;
    let appendPickerQuery = "";
    let appendPickerNotes = [];
    let appendPickerSelectedIndex = 0;
    let appendPickerStep = 1;
    let appendPickerSelectedNote = null;
    let appendPickerHeadings = [];
    let appendPickerHeadingIndex = 0;
    let appendPickerInputRef;
    let uploadedImages = [];
    let unlistenReset;
    let unlistenShow;
    let unlistenInsertCaptureText;
    let unlistenCaptureTextFailed;
    let unlistenSaveAsNote;
    let unlistenSettingsChanged;
    let unlistenDragDrop;
    let isTauri = false;
    let globalDragEnter;
    let globalDragOver;
    let wikiAutocompleteOpen = false;
    let wikiAutocompleteQuery = "";
    let wikiAutocompleteIndex = 0;
    let wikiAutocompleteMatches = [];
    let wikiAutocompletePosition = { left: 0, top: 0 };
    let wikiAnchorPos = 0;
    let globalDragLeave;
    let globalDrop;

    let appSettings = { ...defaultSettings };

    $: showNotePaths = appSettings?.show_note_paths ?? true;
    $: autocompleteResults = appSettings?.autocomplete_results ?? 20;

    $: brightnessFilter = (() => {
        const b = appSettings.window_brightness;
        if (b === 0) return " brightness(1)";

        if (b > 0) {
            const brightnessValue = 1 + (b / 100) * 0.6;
            const contrastValue = 1 - (b / 100) * 0.25;
            return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
        } else {
            const brightnessValue = 1 + (b / 100) * 0.7;
            const contrastValue = 1 + (-b / 100) * 0.3;
            return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
        }
    })();

    function applyColorSettings(settings = appSettings) {
        const root = document.documentElement;
        root.style.setProperty(
            "--accent-color",
            settings.accent_color ?? "#8b5cf6",
        );
        root.style.setProperty(
            "--internal-link-color",
            settings.internal_link_color ?? "#a78bfa",
        );
        root.style.setProperty(
            "--external-link-color",
            settings.external_link_color ?? "#60a5fa",
        );
    }

    async function insertAfterHeading(notePath, heading, text) {
        const fileContent = await invoke("read_note_file", { path: notePath });
        const lines = fileContent.split("\n");

        let insertAt = lines.length;
        for (let i = heading.lineIndex + 1; i < lines.length; i += 1) {
            const match = lines[i].match(/^(#{1,6})\s/);
            if (match && match[1].length <= heading.level) {
                insertAt = i;
                break;
            }
        }

        const entryLines = [
            formatEntryHeader(appSettings.entry_header ?? "#### HH:mm"),
            ...text.split("\n"),
            "",
        ];
        const separator =
            insertAt > 0 && lines[insertAt - 1]?.trim() !== "" ? [""] : [];

        lines.splice(insertAt, 0, ...separator, ...entryLines);

        await invoke("write_note_file", {
            path: notePath,
            content: lines.join("\n"),
        });
    }

    function closeAppendPicker() {
        showAppendPicker = false;
        appendPickerStep = 1;
        appendPickerSelectedNote = null;
        appendPickerHeadings = [];
        appendPickerQuery = "";
        appendPickerSelectedIndex = 0;
        appendPickerHeadingIndex = 0;
        textareaRef?.focus();
    }

    function returnAppendPickerToStep1() {
        appendPickerStep = 1;
        appendPickerSelectedNote = null;
        appendPickerHeadings = [];
        appendPickerHeadingIndex = 0;
    }

    function handleDragDropEvent(event) {
        const payload = event?.payload;
        if (!payload) return;

        if (payload.type === "enter" || payload.type === "over") {
            isDragging = true;
            return;
        }

        if (payload.type === "leave") {
            isDragging = false;
            dragCounter = 0;
            return;
        }

        if (payload.type === "drop") {
            isDragging = false;
            dragCounter = 0;
            if (Array.isArray(payload.paths)) {
                handleTauriFileDrop(payload.paths);
            }
        }
    }

    /* ── onMount setup helpers ── */

    function setupGlobalDragListeners() {
        globalDragEnter = (e) => {
            if (!isFileDrag(e)) return;
            e.preventDefault();
            dragCounter = Math.max(dragCounter, 1);
            isDragging = true;
        };

        globalDragOver = (e) => {
            e.preventDefault();
            if (isFileDrag(e)) {
                e.dataTransfer.dropEffect = "copy";
            }
        };

        globalDragLeave = (e) => {
            if (e.target === document.documentElement) {
                dragCounter = 0;
                isDragging = false;
            }
        };

        globalDrop = (e) => {
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
    }

    async function setupCaptureWindowListeners() {
        // reset_capture: clear all capture state for a fresh start.
        // Emitted before show_capture when the user wants a clean slate
        // (global shortcut, edge detection, tray).
        // NOT emitted by capture-text (Cmd+Shift+C) — that must append.
        unlistenReset = await listen("reset_capture", () => {
            freeBlobUrls();
            resetCaptureState();
            statusMessage = "";
            closeAppendPicker();
        });

        unlistenShow = await listen("show_capture", () => {
            // Just show and focus — no reset, so existing content survives.
            setTimeout(() => textareaRef?.focus(), 50);
        });

        unlistenInsertCaptureText = await listen("insert_capture_text", (event) => {
            const text = typeof event.payload === "string" ? event.payload : "";
            if (!text.trim()) return;
            // Append captured text to existing content instead of replacing it.
            // If content is empty, set directly; otherwise add spacing then append.
            if (!content.trim()) {
                content = text;
            } else {
                // Add spacing: if content doesn't end with a blank line, insert two newlines.
                // If it already ends with a blank line, insert one newline.
                const endsWithBlank = /\n\s*\n$/.test(content);
                content = content + (endsWithBlank ? "\n" : "\n\n") + text;
            }
            // Required: after the native Tauri insert event, macOS needs a short delay
            // before the textarea reliably accepts focus.
            setTimeout(() => textareaRef?.focus(), 50);
        });

        unlistenCaptureTextFailed = await listen("capture_text_failed", (event) => {
            const msg =
                typeof event.payload === "string"
                    ? event.payload
                    : "Nothing to capture";
            showStatus("✗ " + msg, "error");
        });

        unlistenSaveAsNote = await listen("save_as_note", () => {
            handleSaveAsNote();
        });
    }

    async function setupSettingsListener() {
        unlistenSettingsChanged = await listen("settings_changed", (event) => {
            const newSettings = event.payload;

            appSettings = {
                ...appSettings,
                overlay_color:
                    newSettings.overlay_color ??
                    appSettings.overlay_color,
                font_family: newSettings.font_family ?? appSettings.font_family,
                font_size: newSettings.font_size ?? appSettings.font_size,
                border_radius:
                    newSettings.border_radius ?? appSettings.border_radius,
                overlay_strength:
                    newSettings.overlay_strength ??
                    appSettings.overlay_strength,
                window_blur: newSettings.window_blur ?? appSettings.window_blur,
                window_saturation:
                    newSettings.window_saturation ??
                    appSettings.window_saturation,
                window_brightness:
                    newSettings.window_brightness ??
                    appSettings.window_brightness,
                text_color: newSettings.text_color ?? appSettings.text_color,
                accent_color:
                    newSettings.accent_color ?? appSettings.accent_color,
                internal_link_color:
                    newSettings.internal_link_color ??
                    appSettings.internal_link_color,
                external_link_color:
                    newSettings.external_link_color ??
                    appSettings.external_link_color,
                entry_header:
                    newSettings.entry_header ?? appSettings.entry_header,
                show_note_paths:
                    newSettings.show_note_paths ?? appSettings.show_note_paths,
                autocomplete_results:
                    newSettings.autocomplete_results ??
                    appSettings.autocomplete_results,
                save_to_daily_shortcut:
                    newSettings.save_to_daily_shortcut ??
                    appSettings.save_to_daily_shortcut,
                save_as_note_shortcut:
                    newSettings.save_as_note_shortcut ??
                    appSettings.save_as_note_shortcut,
                append_to_note_shortcut:
                    newSettings.append_to_note_shortcut ??
                    appSettings.append_to_note_shortcut,
                show_capture_action_bar:
                    newSettings.show_capture_action_bar ??
                    appSettings.show_capture_action_bar,
            };
            applyColorSettings(appSettings);
        });
    }

    async function setupTauriDragDrop() {
        const currentWindow = await getCurrentWindow();
        unlistenDragDrop =
            await currentWindow.onDragDropEvent(handleDragDropEvent);
    }

    async function loadInitialSettings() {
        try {
            const settings = await invoke("load_settings");
            appSettings = {
                overlay_color: settings.overlay_color,
                font_family: settings.font_family,
                font_size: settings.font_size,
                border_radius: settings.border_radius,
                overlay_strength:
                    settings.overlay_strength ??
                    defaultSettings.overlay_strength,
                window_blur:
                    settings.window_blur ?? defaultSettings.window_blur,
                window_saturation:
                    settings.window_saturation ??
                    defaultSettings.window_saturation,
                window_brightness:
                    settings.window_brightness ??
                    defaultSettings.window_brightness,
                text_color: settings.text_color ?? defaultSettings.text_color,
                accent_color:
                    settings.accent_color ?? defaultSettings.accent_color,
                internal_link_color:
                    settings.internal_link_color ??
                    defaultSettings.internal_link_color,
                external_link_color:
                    settings.external_link_color ??
                    defaultSettings.external_link_color,
                entry_header:
                    settings.entry_header ?? defaultSettings.entry_header,
                show_note_paths:
                    settings.show_note_paths ?? defaultSettings.show_note_paths,
                autocomplete_results:
                    settings.autocomplete_results ??
                    defaultSettings.autocomplete_results,
                save_to_daily_shortcut:
                    settings.save_to_daily_shortcut ?? "Cmd+Enter",
                save_as_note_shortcut:
                    settings.save_as_note_shortcut ?? "Cmd+Shift+Enter",
                append_to_note_shortcut:
                    settings.append_to_note_shortcut ?? "Cmd+Option+Enter",
                show_capture_action_bar:
                    settings.show_capture_action_bar ?? true,
            };
            applyColorSettings(appSettings);
        } catch (e) {
            console.error("Failed to load initial settings:", e);
        }
    }

    function preloadVaultNotes() {
        // Preload vault notes so the append picker can open immediately.
        invoke("list_vault_notes")
            .then((notes) => {
                appendPickerNotes = notes;
            })
            .catch(() => {
                // Non-fatal: the picker can still lazy-load later.
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

        setupGlobalDragListeners();

        if (isTauri) {
            try {
                await setupCaptureWindowListeners();
                await setupSettingsListener();
                await setupTauriDragDrop();
                await loadInitialSettings();
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

        if (globalDragEnter) {
            document.removeEventListener("dragenter", globalDragEnter, true);
        }
        if (globalDragOver) {
            document.removeEventListener("dragover", globalDragOver, true);
        }
        if (globalDragLeave) {
            document.removeEventListener("dragleave", globalDragLeave, true);
        }
        if (globalDrop) {
            document.removeEventListener("drop", globalDrop, true);
        }

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

        showAppendPicker = true;
        appendPickerStep = 1;
        appendPickerSelectedNote = null;
        appendPickerHeadings = [];
        appendPickerQuery = "";
        appendPickerSelectedIndex = 0;
        appendPickerHeadingIndex = 0;
        appendPickerInputRef?.focus();

        if (appendPickerNotes.length === 0) {
            invoke("list_vault_notes")
                .then((notes) => {
                    appendPickerNotes = notes;
                })
                .catch(() => {
                    showStatus("✗ Could not load vault notes", "error");
                });
        }
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
        // Store the selected note immediately for reference
        appendPickerSelectedNote = note;

        const notePath = getVaultNotePath(note);
        if (!notePath) {
            // No valid path - transition to step 2 with empty headings
            appendPickerHeadings = [];
            appendPickerHeadingIndex = 0;
            appendPickerStep = 2;
            return;
        }

        // Load headings BEFORE transitioning to step 2.
        // This prevents a race condition where the IPC call and UI transition
        // happen simultaneously, which can freeze WKWebView on macOS.
        let loadedHeadings = [];
        try {
            const fileContent = await invoke("read_note_file", {
                path: notePath,
            });
            // Check if user closed picker or selected a different note while loading
            if (getVaultNotePath(appendPickerSelectedNote) !== notePath) return;
            loadedHeadings = parseHeadings(fileContent);
        } catch (e) {
            // If loading fails, proceed with empty headings
            if (getVaultNotePath(appendPickerSelectedNote) !== notePath) return;
            loadedHeadings = [];
        }

        // Now that headings are loaded, transition to step 2
        appendPickerHeadings = loadedHeadings;
        appendPickerHeadingIndex = 0;
        // setTimeout(0) gives WKWebView a single event-loop tick
        // between loading headings and transitioning to step 2.
        // Without this pause, the simultaneous Svelte DOM update
        // and IPC completion cause WKWebView to freeze on macOS.
        await new Promise((resolve) => setTimeout(resolve, 0));
        appendPickerStep = 2;
    }

    async function handleAppendToNote(note, heading = null) {
        closeAppendPicker();

        const notePath = getVaultNotePath(note);
        if (!notePath || !content.trim() || isLoading) return;

        isLoading = true;

        try {
            if (!heading) {
                await invoke("append_to_note", {
                    path: notePath,
                    text: content.trim(),
                });
            } else {
                await insertAfterHeading(notePath, heading, content.trim());
            }

            freeBlobUrls();
            resetCaptureState();
            deferHideCapture();
        } catch (e) {
            showStatus("✗ " + e.toString(), "error");
            isLoading = false;
        }
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

    function openWikiAutocomplete(trigger, matches) {
        wikiAutocompleteQuery = trigger.query;
        wikiAutocompleteMatches = matches;
        wikiAutocompleteIndex = 0;
        wikiAnchorPos = trigger.triggerIndex;
        wikiAutocompletePosition = getWikiDropdownPosition(
            trigger.triggerIndex,
            textareaRef?.selectionStart ?? trigger.triggerIndex,
        );
        wikiAutocompleteOpen = matches.length > 0;
    }

    function resetWikiAutocomplete() {
        wikiAutocompleteOpen = false;
        wikiAutocompleteQuery = "";
        wikiAutocompleteMatches = [];
        wikiAutocompleteIndex = 0;
        wikiAutocompletePosition = { left: 0, top: 0 };
        wikiAnchorPos = 0;
    }

    function handleWikiInput() {
        if (!textareaRef) return;

        const val = textareaRef.value;
        const cursor = textareaRef.selectionStart;
        const trigger = findWikiTrigger(val, cursor);

        if (!trigger) {
            closeWikiAutocomplete();
            return;
        }

        const results = getAutocompleteResults(
            trigger.query,
            appendPickerNotes,
            autocompleteResults,
        );

        openWikiAutocomplete(trigger, results);
    }

    function insertWikilink(note) {
        if (!textareaRef) return;

        const val = textareaRef.value;
        const cursor = textareaRef.selectionStart;
        const queryLen = wikiAutocompleteQuery.length;

        if (cursor - queryLen - 2 < 0) return;

        const { before, after, insertion, newPosition } =
            computeWikilinkInsertion(val, cursor, queryLen, note.name);

        textareaRef.value = before + insertion + after;
        textareaRef.selectionStart = newPosition;
        textareaRef.selectionEnd = newPosition;
        textareaRef.focus();

        content = textareaRef.value;
        closeWikiAutocomplete();
    }

    function getTextareaPosition(textarea, position) {
        const rect = textarea.getBoundingClientRect();
        const style = getComputedStyle(textarea);
        const mirror = document.createElement("div");
        const marker = document.createElement("span");
        const copyStyles = [
            "boxSizing",
            "borderTopWidth",
            "borderRightWidth",
            "borderBottomWidth",
            "borderLeftWidth",
            "fontFamily",
            "fontSize",
            "fontStyle",
            "fontVariant",
            "fontWeight",
            "letterSpacing",
            "lineHeight",
            "paddingTop",
            "paddingRight",
            "paddingBottom",
            "paddingLeft",
            "tabSize",
            "textIndent",
            "textTransform",
            "wordBreak",
            "wordSpacing",
        ];

        copyStyles.forEach((property) => {
            mirror.style[property] = style[property];
        });

        mirror.style.position = "absolute";
        mirror.style.visibility = "hidden";
        mirror.style.pointerEvents = "none";
        mirror.style.whiteSpace = "pre-wrap";
        mirror.style.wordWrap = "break-word";
        mirror.style.overflowWrap = "break-word";
        mirror.style.overflow = "hidden";
        mirror.style.width = `${textarea.offsetWidth}px`;
        mirror.style.top = "0";
        mirror.style.left = "-9999px";

        const textBefore = textarea.value.substring(0, position);
        mirror.textContent = textBefore.endsWith("\n")
            ? `${textBefore} `
            : textBefore;
        marker.textContent = ".";
        mirror.appendChild(marker);
        document.body.appendChild(mirror);

        try {
            const lineHeight = parseFloat(style.lineHeight) || 24;

            return {
                left: rect.left + marker.offsetLeft - textarea.scrollLeft,
                top: rect.top + marker.offsetTop - textarea.scrollTop,
                bottom:
                    rect.top +
                    marker.offsetTop -
                    textarea.scrollTop +
                    lineHeight,
            };
        } finally {
            mirror.remove();
        }
    }

    function clampWikiDropdownLeft(left) {
        const margin = 12;
        const pickerWidth = Math.min(320, window.innerWidth - margin * 2);
        const maxLeft = window.innerWidth - pickerWidth - margin;

        return Math.max(margin, Math.min(left, maxLeft));
    }

    function getWikiDropdownPosition(anchorPosition, cursorPosition) {
        if (!textareaRef) return { left: 0, top: 0 };

        const anchorRect = getTextareaPosition(textareaRef, anchorPosition);
        const caretRect = getTextareaPosition(textareaRef, cursorPosition);

        return {
            left: clampWikiDropdownLeft(anchorRect.left),
            top: caretRect.bottom + 6,
        };
    }

    function closeWikiAutocomplete() {
        resetWikiAutocomplete();
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
        e.preventDefault();

        if (isFileDrag(e)) {
            dragCounter++;
            if (dragCounter === 1) {
                isDragging = true;
            }
        }
    }

    function handleDragLeave(e) {
        e.preventDefault();

        const rect = e.currentTarget.getBoundingClientRect();
        const x = e.clientX;
        const y = e.clientY;
        const isLeaving =
            x < rect.left || x > rect.right || y < rect.top || y > rect.bottom;

        if (isLeaving) {
            dragCounter--;
            if (dragCounter <= 0) {
                dragCounter = 0;
                isDragging = false;
            }
        }
    }

    function handleDragOver(e) {
        e.preventDefault();
        if (isFileDrag(e)) {
            e.dataTransfer.dropEffect = "copy";
            if (!isDragging) {
                isDragging = true;
            }
        } else {
            if (e.dataTransfer) {
                e.dataTransfer.dropEffect = "none";
            }
        }
    }

    async function handleDrop(e) {
        e.preventDefault();
        e.stopPropagation();
        isDragging = false;
        dragCounter = 0;

        if (!e.dataTransfer) {
            showStatus(
                "Failed to import images: dataTransfer unavailable",
                "error",
            );
            return;
        }

        const items = Array.from(e.dataTransfer.items || []);
        const files = Array.from(e.dataTransfer.files || []);

        if (files.length === 0) {
            showStatus("Failed to import images: no files found", "error");
            return;
        }

        const promises = files.map(async (file, index) => {
            const ext = file.name.split(".").pop()?.toLowerCase();
            if (!["png", "jpg", "jpeg", "webp", "gif"].includes(ext)) {
                showStatus("Unsupported image: " + file.name, "error");
                return null;
            }

            try {
                let filePath = file.path || file.webkitRelativePath || null;

                if (!filePath && items[index]) {
                    const item = items[index];
                    if (item.kind === "file") {
                        const fileFromItem = item.getAsFile();
                        if (
                            fileFromItem &&
                            (fileFromItem.path ||
                                fileFromItem.webkitRelativePath)
                        ) {
                            filePath =
                                fileFromItem.path ||
                                fileFromItem.webkitRelativePath;
                        }
                    }
                }

                let normalizedResult;

                if (filePath) {
                    const result = await invoke("save_image", {
                        filePath: filePath,
                    });
                    normalizedResult = normalizeImageResult(result);
                } else {
                    if (!isTauri) {
                        showStatus(
                            "Failed to import images outside the Tauri app",
                            "error",
                        );
                        return null;
                    }

                    const arrayBuffer = await file.arrayBuffer();
                    const uint8Array = new Uint8Array(arrayBuffer);

                    let base64;
                    try {
                        base64 = await new Promise((resolve, reject) => {
                            const reader = new FileReader();
                            reader.onload = () => {
                                const result = reader.result;
                                const base64String =
                                    typeof result === "string"
                                        ? result.split(",")[1] || result
                                        : "";
                                resolve(base64String);
                            };
                            reader.onerror = reject;
                            reader.readAsDataURL(file);
                        });
                    } catch (base64Error) {
                        console.error(
                            "Failed to convert to base64:",
                            base64Error,
                        );
                        throw new Error(
                            "Failed to encode file: " + base64Error.toString(),
                        );
                    }

                    try {
                        if (typeof invoke === "undefined") {
                            throw new Error(
                                "invoke is undefined - not running in Tauri",
                            );
                        }
                        const result = await invoke("save_image_from_bytes", {
                            bytesBase64: base64,
                            filename: file.name,
                        });
                        normalizedResult = normalizeImageResult(result);
                    } catch (invokeError) {
                        console.error("Invoke error:", invokeError);
                        throw invokeError;
                    }
                }

                const previewUrl =
                    normalizedResult?.preview_data_url ||
                    URL.createObjectURL(file);

                return {
                    id: Date.now() + Math.random() + index,
                    filename: normalizedResult?.filename || file.name,
                    markdown: normalizedResult?.markdown || "",
                    preview: previewUrl,
                    file: file,
                };
            } catch (e) {
                console.error("Error processing file:", e);
                showStatus("Failed to import image: " + e.toString(), "error");
                return null;
            }
        });

        const results = await Promise.all(promises);
        const validImages = results.filter((img) => img !== null);

        if (validImages.length > 0) {
            let insertPosition = 0;
            if (textareaRef) {
                const textareaRect = textareaRef.getBoundingClientRect();
                const dropX = e.clientX;
                const dropY = e.clientY;
                const isOverTextarea =
                    dropX >= textareaRect.left &&
                    dropX <= textareaRect.right &&
                    dropY >= textareaRect.top &&
                    dropY <= textareaRect.bottom;

                if (
                    isOverTextarea ||
                    e.target === textareaRef ||
                    e.target.closest("textarea") === textareaRef
                ) {
                    if (
                        textareaRef.selectionStart !== null &&
                        textareaRef.selectionStart !== undefined
                    ) {
                        insertPosition = textareaRef.selectionStart;
                    } else {
                        const lineHeight =
                            parseInt(
                                getComputedStyle(textareaRef).lineHeight,
                            ) || 20;
                        const scrollTop = textareaRef.scrollTop;
                        const relativeY = dropY - textareaRect.top + scrollTop;
                        const estimatedLine = Math.max(
                            0,
                            Math.floor(relativeY / lineHeight),
                        );

                        const lines = content.split("\n");
                        let pos = 0;
                        for (
                            let i = 0;
                            i < Math.min(estimatedLine, lines.length);
                            i++
                        ) {
                            pos += lines[i].length + 1;
                        }
                        insertPosition = Math.min(pos, content.length);
                    }
                } else {
                    insertPosition = content.length;
                }
            } else {
                insertPosition = content.length;
            }

            let currentPosition = insertPosition;
            validImages.forEach((img) => {
                const imageMarkdown = img.markdown + "\n";

                if (currentPosition >= content.length) {
                    if (content.length > 0 && !content.endsWith("\n")) {
                        content += "\n";
                        currentPosition = content.length;
                    }
                    content += imageMarkdown;
                    currentPosition += imageMarkdown.length;
                } else {
                    const before = content.substring(0, currentPosition);
                    const after = content.substring(currentPosition);

                    const isAtLineStart =
                        before.endsWith("\n") || before.length === 0;

                    if (!isAtLineStart && !before.endsWith("\n\n")) {
                        content = before + "\n" + imageMarkdown + after;
                        currentPosition += 1 + imageMarkdown.length;
                    } else {
                        content = before + imageMarkdown + after;
                        currentPosition += imageMarkdown.length;
                    }
                }
            });

            uploadedImages = [...uploadedImages, ...validImages];

            if (textareaRef) {
                await tick();
                const newPosition = currentPosition;
                textareaRef.setSelectionRange(newPosition, newPosition);
                textareaRef.focus();
            }
        }
    }

    async function handleTauriFileDrop(paths) {
        isDragging = false;
        dragCounter = 0;

        if (!paths || paths.length === 0) {
            showStatus("Failed to import images: no files found", "error");
            return;
        }

        const imagePaths = paths.filter((path) => {
            const ext = path.split(".").pop()?.toLowerCase();
            return ["png", "jpg", "jpeg", "webp", "gif"].includes(ext);
        });

        if (imagePaths.length === 0) {
            showStatus(
                "Failed to import images: no supported image files found",
                "error",
            );
            return;
        }

        const promises = imagePaths.map(async (filePath, index) => {
            try {
                const result = await invoke("save_image", {
                    filePath: filePath,
                });
                const normalizedResult = normalizeImageResult(result);

                const previewUrl = normalizedResult.preview_data_url || null;

                return {
                    id: Date.now() + Math.random() + index,
                    filename:
                        normalizedResult.filename ||
                        normalizeFilePath(
                            normalizedResult.saved_path || filePath,
                        )
                            .split("/")
                            .pop() ||
                        `image${index}`,
                    markdown: normalizedResult.markdown,
                    preview: previewUrl,
                    file: null,
                };
            } catch (e) {
                console.error("Error processing file:", e);
                showStatus("Failed to import image: " + e.toString(), "error");
                return null;
            }
        });

        const results = await Promise.all(promises);
        const validImages = results.filter((img) => img !== null);

        if (validImages.length > 0) {
            uploadedImages = [...uploadedImages, ...validImages];
            if (content.length > 0 && !content.endsWith("\n")) {
                content += "\n";
            }
            validImages.forEach((img) => {
                content += img.markdown + "\n";
            });
        }

        // Required: after the native Tauri drag-drop bridge finishes, macOS needs a
        // short delay before the textarea reliably regains focus.
        setTimeout(() => textareaRef?.focus(), 50);
    }

    function removeImage(id) {
        const image = uploadedImages.find((img) => img.id === id);
        if (image && image.preview && image.preview.startsWith("blob:")) {
            URL.revokeObjectURL(image.preview);
        }
        uploadedImages = uploadedImages.filter((img) => img.id !== id);
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
    on:dragenter={handleDragEnter}
    on:dragleave={handleDragLeave}
    on:dragover={handleDragOver}
    on:drop={handleDrop}
    role="application"
>
    <CaptureAccentLine />

    <div
        class="content-wrapper"
        role="presentation"
        on:drop={(e) => {
            handleDrop(e);
        }}
        on:dragover={(e) => {
            e.preventDefault();
            handleDragOver(e);
        }}
        on:dragenter={(e) => {
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
