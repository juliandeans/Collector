<script>
    import { onMount, onDestroy, tick } from "svelte";
    import {
        getCachedImageSrc,
        markdownToHtml,
        normalizeNewlines,
        preprocessContent,
        warmImagesInText,
    } from "./contentProcessing.js";
    import { htmlToMarkdown } from "./editorSerialization.js";
    import { createBlockManager } from "./editorBlockManager.js";
    import { createEditorAutocompleteManager } from "./editorAutocompleteManager.js";
    import {
        handleEditorMouseDown as onEditorMouseDown,
        handleEditorPaste as onEditorPaste,
        insertPlainTextAtSelection,
    } from "./editorMouseAndPaste.js";

    import "./reader-editor.css";

    /**
     * @typedef {Object} Props
     * @property {string} [rawContent]
     * @property {any} [appSettings]
     * @property {any} [vaultNotes]
     * @property {any} [vaultTags]
     * @property {string} [missingFileMessage]
     * @property {string} [activeTabKind]
     * @property {boolean} [showSearch]
     * @property {boolean} [showAutocomplete]
     * @property {any} [autocompleteResults]
     * @property {number} [autocompleteIndex]
     * @property {any} [codeblockMap]
     * @property {any} [hiddenBlockMap]
     * @property {any} [setStrippedFrontmatter]
     * @property {any} [onChange]
     * @property {any} [onAutocompleteChange]
     * @property {any} [onAutocompleteIndexChange]
     * @property {any} [onOpenSearchRequest]
     * @property {any} [onOpenSettingsRequest]
     * @property {any} [onCloseSearchRequest]
     * @property {any} [onCloseRequest]
     * @property {any} [onSaveRequest]
     * @property {any} [onNavigateWikilink]
     * @property {any} [onOpenExternalLink]
     * @property {any} [onSelectionChange]
     * @property {any} [onScroll]
     */

    /** @type {Props} */
    let {
        rawContent = "",
        appSettings = {},
        vaultNotes = [],
        vaultTags = [],
        missingFileMessage = "",
        activeTabKind = "",
        showSearch = false,
        showAutocomplete = false,
        autocompleteResults = [],
        autocompleteIndex = 0,
        codeblockMap = new Map(),
        hiddenBlockMap = new Map(),
        setStrippedFrontmatter = () => {},
        onChange,
        onAutocompleteChange,
        onAutocompleteIndexChange,
        onOpenSearchRequest,
        onOpenSettingsRequest,
        onCloseSearchRequest,
        onCloseRequest,
        onSaveRequest,
        onNavigateWikilink,
        onOpenExternalLink,
        onSelectionChange,
        onScroll,
    } = $props();

    let editorRef = $state();
    let scrollRef = $state();
    let isComposing = false;
    let isRenderingContent = false;
    let renderRequestId = 0;

    function formatShortcutLabel(shortcut = "") {
        return String(shortcut ?? "")
            .split("+")
            .map((part) => part.trim())
            .filter(Boolean)
            .join("+");
    }

    let dailyNoteShortcutLabel = $derived(
        formatShortcutLabel(appSettings?.save_to_daily_shortcut),
    );
    let missingFileBannerMessage = $derived(
        missingFileMessage.trim() &&
            activeTabKind === "daily" &&
            appSettings?.daily_note_create_if_missing &&
            dailyNoteShortcutLabel
            ? `File not found — create with ${dailyNoteShortcutLabel}`
            : missingFileMessage,
    );

    const autocompleteManager = createEditorAutocompleteManager({
        getEditorRef: () => editorRef,
        getVaultNotes: () => vaultNotes,
        getVaultTags: () => vaultTags,
        getAutocompleteLimit: () => appSettings?.autocomplete_results ?? 20,
        getShowAutocomplete: () => showAutocomplete,
        getAutocompleteIndex: () => autocompleteIndex,
        getAutocompleteResultsList: () => autocompleteResults,
        onAutocompleteChange: (...args) => onAutocompleteChange?.(...args),
        onAutocompleteIndexChange: (...args) =>
            onAutocompleteIndexChange?.(...args),
        onChange: (...args) => onChange?.(...args),
    });

    const blockManager = createBlockManager({
        getEditorRef: () => editorRef,
        getIsRendering: () => isRenderingContent,
        onSelectionChange: (...args) => onSelectionChange?.(...args),
        checkAutocomplete: autocompleteManager.checkAutocomplete,
    });

    export function clearAutocomplete() {
        autocompleteManager.clearAutocomplete();
    }

    export function insertAutocompleteResult(note) {
        autocompleteManager.insertAutocompleteResult(note);
    }

    function patchResolvedImages() {
        if (!editorRef) return;

        const blankImages = editorRef.querySelectorAll('img[src=""]');
        for (const image of blankImages) {
            const path =
                image.getAttribute("data-path") || image.getAttribute("alt");
            if (!path) continue;

            const cachedSrc = getCachedImageSrc(path);
            if (cachedSrc) {
                image.src = cachedSrc;
            }
        }
    }

    export function getMarkdown() {
        if (!editorRef) {
            return preprocessContent(rawContent, {
                appSettings,
                codeblockMap,
                hiddenBlockMap,
                setStrippedFrontmatter,
            });
        }

        return normalizeNewlines(htmlToMarkdown(editorRef));
    }

    export async function renderContent(raw = "") {
        const processed = preprocessContent(raw, {
            appSettings,
            codeblockMap,
            hiddenBlockMap,
            setStrippedFrontmatter,
        });
        blockManager.clearActive();

        if (!editorRef) return;
        if (scrollRef) scrollRef.scrollTop = 0;

        const requestId = ++renderRequestId;
        isRenderingContent = true;
        editorRef.innerHTML = markdownToHtml(processed, appSettings);
        isRenderingContent = false;

        const loadedAnyMissingImages = await warmImagesInText(processed);
        if (!loadedAnyMissingImages) {
            return;
        }

        if (requestId !== renderRequestId || !editorRef) {
            return;
        }

        patchResolvedImages();
    }

    export async function restoreScroll(path, scrollPositions) {
        await tick();
        await new Promise((resolve) => {
            requestAnimationFrame(() => {
                if (scrollRef) {
                    scrollRef.scrollTop = scrollPositions.get(path) ?? 0;
                }
                resolve();
            });
        });
    }

    export function finalizeBlock() {
        blockManager.finalizeActiveBlock();
    }

    export function focus() {
        editorRef?.focus();
    }

    export function getEditorElement() {
        return editorRef;
    }

    export function getScrollElement() {
        return scrollRef;
    }

    export function getScrollTop() {
        return scrollRef?.scrollTop ?? 0;
    }

    function handleInput() {
        if (isComposing || isRenderingContent || !editorRef) return;
        onChange?.();
        autocompleteManager.checkAutocomplete();
    }

    function handleCompositionStart() {
        isComposing = true;
    }

    function handleCompositionEnd() {
        isComposing = false;
        onChange?.();
        autocompleteManager.checkAutocomplete();
    }

    function handleKeydown(event) {
        const key = event.key.toLowerCase();

        if (autocompleteManager.handleAutocompleteKeydown(event)) {
            event.stopPropagation();
            return;
        }

        if ((event.metaKey || event.ctrlKey) && key === "f") {
            event.preventDefault();
            event.stopPropagation();
            onOpenSearchRequest?.();
            return;
        }

        if ((event.metaKey || event.ctrlKey) && key === ",") {
            event.preventDefault();
            event.stopPropagation();
            onOpenSettingsRequest?.();
            return;
        }

        if (event.key === "Escape") {
            event.preventDefault();
            event.stopPropagation();
            if (showAutocomplete) {
                autocompleteManager.closeAutocomplete();
                return;
            }
            if (showSearch) {
                onCloseSearchRequest?.();
                return;
            }
            onCloseRequest?.();
            return;
        }

        if ((event.metaKey || event.ctrlKey) && key === "s") {
            event.preventDefault();
            event.stopPropagation();
            onSaveRequest?.();
            return;
        }
    }

    export function insertPlainText(text) {
        if (!insertPlainTextAtSelection(text, editorRef)) return false;
        onChange?.();
        autocompleteManager.checkAutocomplete();
        return true;
    }

    function handleEditorPaste(event) {
        onEditorPaste(event, editorRef, {
            onChange: (...args) => onChange?.(...args),
            checkAutocomplete: autocompleteManager.checkAutocomplete,
        });
    }

    function handleEditorMouseDown(event) {
        onEditorMouseDown(event, {
            onNavigateWikilink: (...args) => onNavigateWikilink?.(...args),
            onOpenExternalLink: (...args) => onOpenExternalLink?.(...args),
        });
    }

    onMount(() => {
        document.addEventListener(
            "selectionchange",
            blockManager.handleSelectionChange,
        );
    });

    onDestroy(() => {
        document.removeEventListener(
            "selectionchange",
            blockManager.handleSelectionChange,
        );
    });
</script>

<div
    class="editor-scroll"
    bind:this={scrollRef}
    onscroll={() => onScroll?.()}
>
    {#if missingFileBannerMessage.trim()}
        <div class="missing-file-banner">{missingFileBannerMessage}</div>
    {/if}
    <div
        class="editor-body"
        contenteditable="true"
        bind:this={editorRef}
        spellcheck="true"
        role="textbox"
        aria-multiline="true"
        tabindex="0"
        data-placeholder="Start writing..."
        oninput={handleInput}
        onkeydown={handleKeydown}
        onpaste={handleEditorPaste}
        onmousedown={handleEditorMouseDown}
        onblur={blockManager.handleEditorBlur}
        oncompositionstart={handleCompositionStart}
        oncompositionend={handleCompositionEnd}
    ></div>
</div>
