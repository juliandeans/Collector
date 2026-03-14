<script>
  import "./lib/reader/reader-shell.css";
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy, tick } from "svelte";
  import CommandPalette from "./lib/reader/CommandPalette.svelte";
  import ReaderEditor from "./lib/reader/ReaderEditor.svelte";
  import ReaderTopBar from "./lib/reader/ReaderTopBar.svelte";
  import SearchBar from "./lib/reader/SearchBar.svelte";
  import StatusToast from "./lib/reader/StatusToast.svelte";
  import TabContextMenu from "./lib/reader/TabContextMenu.svelte";
  import {
    imagePathCache,
    normalizeNewlines,
    preprocessContent,
  } from "./lib/reader/contentProcessing.js";
  import { getAutocompleteResults } from "./lib/reader/autocomplete.js";
  import { composeContentFromMarkdown } from "./lib/reader/editorSerialization.js";
  import {
    createImportPlaceholder,
    processDroppedFiles,
    processDroppedPaths,
  } from "./lib/reader/imageImport.js";
  import { setupListeners } from "./lib/reader/lifecycleSetup.js";
  import {
    ensureVaultNotes,
    navigateToWikilink,
    openInObsidian,
  } from "./lib/reader/navigation.js";
  import { createDebouncedJob } from "./lib/reader/saveLoadPipeline.js";
  import {
    applyColorSettings,
    applySettings,
    computeBrightnessFilter,
    getReaderFilterSettings,
    haveReaderFilterChanges,
  } from "./lib/reader/settingsBridge.js";
  import {
    applySearchHighlights,
    clearSearchHighlights,
    runSearch as collectSearchMatches,
    stepSearch as getNextSearchIndex,
  } from "./lib/reader/searchLogic.js";
  import {
    createTab,
    fileLabel,
    getPinnedNotesSignature,
    rebuildTabsFromSettings,
  } from "./lib/reader/tabState.js";
  import {
    getDailyNotePath,
    loadTabContent,
    saveTabContent,
  } from "./lib/reader/tabIO.js";
  import {
    filterPaletteNotes,
    getOpenVaultNoteIntent,
  } from "./lib/reader/paletteLogic.js";

  let tabs = [];
  let activeTabIndex = 0;
  let showPalette = false;
  let paletteQuery = "";
  let vaultNotes = [];
  let isSaving = false;
  let rawContent = "";
  let strippedFrontmatter = "";
  let codeblockMap = new Map();
  let hiddenBlockMap = new Map();
  const scrollPositions = new Map();
  let appSettings = {
    vault_name: "Vault",
    vault_path: "",
    background_color: "#1e1e2e",
    font_family: "-apple-system, BlinkMacSystemFont, SF Pro Display",
    font_size: 15,
    border_radius: 12,
    window_transparency: 55,
    window_blur: 80,
    window_saturation: 200,
    window_brightness: 0,
    text_color: "#ffffff",
    accent_color: "#8b5cf6",
    internal_link_color: "#a78bfa",
    external_link_color: "#60a5fa",
    pinned_notes: [],
    reader_hide_frontmatter: true,
    reader_hide_dataview: true,
    reader_hide_obsidian_comments: true,
    reader_hide_inline_fields: true,
    reader_hide_html: true,
  };
  let statusMessage = "";
  let statusType = "";
  let missingFileMessage = "";
  let selectedPaletteIndex = 0;
  let showSavedIndicator = false;
  let showAutocomplete = false;
  let autocompleteIndex = 0;
  let autocompleteResults = [];
  let autocompleteRange = null;
  let showSearch = false;
  let searchQuery = "";
  let searchMatches = [];
  let searchIndex = 0;
  let searchInputRef;
  let isDragging = false;
  let dragCounter = 0;
  let isImportingImages = false;
  let tabContextMenu = {
    open: false,
    x: 0,
    y: 0,
    tabIndex: -1,
  };

  let editorComponent;
  let paletteInputRef;
  let statusTimeout;
  let savedIndicatorTimeout;
  let unlistenShowReader;
  let unlistenSettingsChanged;
  let cleanupGlobalListeners = () => {};
  const saveScheduler = createDebouncedJob(600);

  $: activeTab = tabs[activeTabIndex] ?? null;
  $: fileMissing = missingFileMessage.trim() !== "";
  $: filteredVaultNotes = filterPaletteNotes(vaultNotes, paletteQuery);
  $: if (selectedPaletteIndex >= filteredVaultNotes.length) {
    selectedPaletteIndex = Math.max(filteredVaultNotes.length - 1, 0);
  }
  $: brightnessFilter = computeBrightnessFilter(appSettings.window_brightness);

  function isFileDrag(event) {
    const types = event.dataTransfer?.types;
    if (types && Array.from(types).includes("Files")) return true;
    const items = event.dataTransfer?.items;
    return !!items && items.length > 0;
  }

  function normalizeError(error) {
    return typeof error === "string"
      ? error
      : error?.message || error?.toString?.() || "Unknown error";
  }

  function isFileMissingError(error) {
    const text = normalizeError(error).toLowerCase();
    return (
      text.includes("no such file") ||
      text.includes("not found") ||
      text.includes("cannot find")
    );
  }

  function showStatus(message, type = "success", duration = 1500) {
    clearTimeout(statusTimeout);
    statusMessage = message;
    statusType = type;
    // Hide the transient status toast after its display period.
    statusTimeout = setTimeout(() => {
      statusMessage = "";
      statusType = "";
    }, duration);
  }

  function closeAutocomplete() {
    showAutocomplete = false;
    autocompleteIndex = 0;
    autocompleteRange = null;
    autocompleteResults = [];
    editorComponent?.clearAutocomplete?.();
  }

  function clearHighlights() {
    clearSearchHighlights();
  }

  function highlightMatches() {
    applySearchHighlights(searchMatches, searchIndex);
  }

  function scrollToMatch(index) {
    const range = searchMatches[index];
    const scrollEl = editorComponent?.getScrollElement?.();
    if (!range || !scrollEl) return;

    const rect = range.getClientRects()[0] ?? range.getBoundingClientRect();
    const scrollRect = scrollEl.getBoundingClientRect();
    if (!rect) return;

    if (rect.top < scrollRect.top || rect.bottom > scrollRect.bottom) {
      const targetTop =
        scrollEl.scrollTop +
        (rect.top - scrollRect.top) -
        scrollEl.clientHeight / 2 +
        rect.height / 2;

      scrollEl.scrollTo({
        top: Math.max(targetTop, 0),
        behavior: "smooth",
      });
    }
  }

  function runSearch(event) {
    if (event?.currentTarget?.value !== undefined) {
      searchQuery = event.currentTarget.value;
    }

    clearHighlights();
    searchMatches = [];
    searchIndex = 0;

    const editorEl = editorComponent?.getEditorElement?.();
    searchMatches = collectSearchMatches(searchQuery, editorEl);
    highlightMatches();

    if (searchMatches.length > 0) {
      scrollToMatch(0);
    }
  }

  function stepSearch(direction) {
    if (searchMatches.length === 0) return;

    searchIndex = getNextSearchIndex(searchMatches, searchIndex, direction);
    highlightMatches();
    scrollToMatch(searchIndex);
  }

  function closeSearch() {
    showSearch = false;
    searchQuery = "";
    clearHighlights();
    searchMatches = [];
    searchIndex = 0;
  }

  async function openSearch() {
    showSearch = true;
    await tick();
    searchInputRef?.focus();
    searchInputRef?.select();
  }

  function insertImportedMarkdown(
    markdownLinks = [],
    { syncEditor = true } = {},
  ) {
    const editorRef = editorComponent?.getEditorElement?.();
    if (!editorRef || markdownLinks.length === 0) return;

    editorRef.focus();

    const content = markdownLinks
      .filter((entry) => entry?.trim())
      .map((entry) => `\n${entry}\n`)
      .join("\n");

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) {
      editorRef.append(document.createTextNode(content));
      if (syncEditor) {
        handleEditorChange();
      }
      return;
    }

    const range = selection.getRangeAt(0);
    if (!editorRef.contains(range.commonAncestorContainer)) {
      editorRef.append(document.createTextNode(content));
      if (syncEditor) {
        handleEditorChange();
      }
      return;
    }

    range.deleteContents();
    const textNode = document.createTextNode(content);
    range.insertNode(textNode);

    const afterRange = document.createRange();
    afterRange.setStartAfter(textNode);
    afterRange.collapse(true);
    selection.removeAllRanges();
    selection.addRange(afterRange);

    if (syncEditor) {
      handleEditorChange();
    }
  }

  async function finalizeImportedImages(replacements) {
    const markdown = getCurrentMarkdown();
    let nextContent = composeContentFromMarkdown(markdown, {
      strippedFrontmatter,
      hiddenBlockMap,
      codeblockMap,
    });

    replacements.forEach(({ placeholder, markdown }) => {
      nextContent = nextContent.replace(placeholder, markdown);
    });

    rawContent = normalizeNewlines(nextContent);
    updateActiveTabContent(rawContent);
    imagePathCache.clear();
    await renderContentToEditor(rawContent);
    scheduleSave(rawContent);
  }

  function buildImportReplacements(results, jobs) {
    return results.map((result, index) => {
      if (result.status === "fulfilled") {
        return {
          placeholder: jobs[index].placeholder,
          markdown: result.value,
        };
      }

      showStatus(normalizeError(result.reason), "error", 2200);
      return { placeholder: jobs[index].placeholder, markdown: "" };
    });
  }

  async function resolveImportJobs(jobs, importer) {
    insertImportedMarkdown(
      jobs.map((job) => job.placeholder),
      { syncEditor: false },
    );

    const results = await Promise.allSettled(jobs.map(importer));
    await finalizeImportedImages(buildImportReplacements(results, jobs));
  }

  async function handleImportedImages(detail) {
    if (detail?.type === "drop") {
      isDragging = false;
      dragCounter = 0;
      const items = Array.from(detail.items || []);
      const files = Array.from(detail.files || []);
      if (files.length === 0) return;

      try {
        isImportingImages = true;

        const jobs = files.map((file, index) => {
          const item = items[index];
          const itemFile =
            item?.kind === "file" ? (item.getAsFile?.() ?? null) : null;
          const fallbackPath =
            file.path ||
            file.webkitRelativePath ||
            itemFile?.path ||
            itemFile?.webkitRelativePath ||
            null;
          const placeholder = createImportPlaceholder(file.name || fallbackPath);

          return {
            file,
            fallbackPath,
            placeholder,
          };
        });

        await resolveImportJobs(jobs, async (job) => {
          if (job.fallbackPath) {
            const [result] = await processDroppedPaths(
              [job.fallbackPath],
              appSettings,
            );
            return result?.markdown ?? "";
          }

          const [result] = await processDroppedFiles([job.file], appSettings);
          return result?.markdown ?? "";
        });
      } catch (error) {
        showStatus(normalizeError(error), "error", 2200);
      } finally {
        isImportingImages = false;
      }
      return;
    }

    if (detail?.type === "paste") {
      const imageItems = Array.from(detail.items || []);
      if (imageItems.length === 0) return;

      try {
        isImportingImages = true;

        const jobs = imageItems
          .map((item) => item.getAsFile?.())
          .filter(Boolean)
          .map((file) => ({
            file,
            placeholder: createImportPlaceholder(file.name),
          }));

        await resolveImportJobs(jobs, async (job) => {
          const [result] = await processDroppedFiles([job.file], appSettings);
          return result?.markdown ?? "";
        });
      } catch (error) {
        showStatus(normalizeError(error), "error", 2200);
      } finally {
        isImportingImages = false;
      }
    }
  }

  async function handleEditorDrop(event) {
    if (!event.dataTransfer) return;

    const items = Array.from(event.dataTransfer.items || []);
    const files = Array.from(event.dataTransfer.files || []);
    if (files.length === 0) return;

    event.preventDefault();
    event.stopPropagation();
    isDragging = false;
    dragCounter = 0;
    await handleImportedImages({
      type: "drop",
      items,
      files,
    });
  }

  function handleDragEnter(event) {
    if (!isFileDrag(event)) return;
    event.preventDefault();
    dragCounter += 1;
    if (dragCounter === 1) {
      isDragging = true;
    }
  }

  function handleDragLeave(event) {
    if (!isFileDrag(event)) return;
    event.preventDefault();

    const rect = event.currentTarget.getBoundingClientRect();
    const x = event.clientX;
    const y = event.clientY;
    const isLeaving =
      x < rect.left || x > rect.right || y < rect.top || y > rect.bottom;

    if (!isLeaving) return;

    dragCounter = Math.max(dragCounter - 1, 0);
    if (dragCounter <= 0) {
      dragCounter = 0;
      isDragging = false;
    }
  }

  function handleDragOver(event) {
    if (!isFileDrag(event)) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = "copy";
    isDragging = true;
  }

  function replaceTab(index, updates) {
    tabs = tabs.map((tab, tabIndex) =>
      tabIndex === index ? { ...tab, ...updates } : tab,
    );
  }

  function updateActiveTabContent(nextContent) {
    if (!activeTab) return;
    replaceTab(activeTabIndex, {
      content: nextContent,
      loaded: true,
    });
  }

  function getCurrentMarkdown() {
    if (!editorComponent) {
      return preprocessContent(rawContent, {
        appSettings,
        codeblockMap,
        hiddenBlockMap,
        setStrippedFrontmatter: (value) => {
          strippedFrontmatter = value;
        },
      });
    }

    return editorComponent.getMarkdown();
  }

  async function renderContentToEditor(raw = "") {
    preprocessContent(raw, {
      appSettings,
      codeblockMap,
      hiddenBlockMap,
      setStrippedFrontmatter: (value) => {
        strippedFrontmatter = value;
      },
    });

    if (!editorComponent) return;
    await editorComponent.renderContent(raw);
  }

  // TODO: keep this in Reader until editor load/reset state can be passed via a dedicated pipeline object.
  async function loadEditorContent(raw = "") {
    editorComponent?.finalizeBlock?.();
    closeAutocomplete();
    closeSearch();
    rawContent = normalizeNewlines(raw);
    await renderContentToEditor(rawContent);
  }

  function saveScrollPosition() {
    const currentTab = tabs[activeTabIndex];
    if (!currentTab || !editorComponent) return;
    scrollPositions.set(currentTab.path, editorComponent.getScrollTop());
  }

  async function restoreScrollPosition(path) {
    if (!editorComponent) return;
    await editorComponent.restoreScroll(path, scrollPositions);
  }

  function handleEditorChange() {
    const markdown = getCurrentMarkdown();
    const content = composeContentFromMarkdown(markdown, {
      strippedFrontmatter,
      hiddenBlockMap,
      codeblockMap,
    });
    rawContent = content;
    updateActiveTabContent(content);
    scheduleSave(content);
  }

  async function handleEditorSaveRequest() {
    await forceSave();
  }

  function handleEditorAutocompleteChange(event) {
    showAutocomplete = event.detail.open;
    autocompleteResults =
      event.detail.results ??
      getAutocompleteResults(event.detail.query, vaultNotes);
    autocompleteRange = event.detail.range;
    autocompleteIndex = event.detail.index ?? 0;
  }

  async function openExternalUrl(url) {
    try {
      await invoke("open_external_url", { url });
    } catch (error) {
      showStatus("Failed to open link", "error", 2200);
    }
  }

  // TODO: extract the tab load/save orchestration once Reader tab state is isolated from editor state.
  function getMissingState(error) {
    const message = normalizeError(error);
    const missing = isFileMissingError(error);
    return {
      message,
      missing,
      missingMessage: missing ? "File not found - will be created on first save" : message,
    };
  }

  function setLoadedTabState(index, content) {
    replaceTab(index, {
      content,
      loaded: true,
      missing: false,
      missingMessage: "",
    });
  }

  function setMissingTabState(index, missingState) {
    replaceTab(index, {
      content: "",
      loaded: true,
      missing: missingState.missing,
      missingMessage: missingState.missingMessage,
    });
  }

  async function syncActiveTabView(index, path, content, missingMessage = "") {
    if (index !== activeTabIndex) return;
    await loadEditorContent(content);
    missingFileMessage = missingMessage;
    await restoreScrollPosition(path);
  }

  async function loadTab(index, forceReload = false) {
    const tab = tabs[index];
    if (!tab) return;

    if (!forceReload && tab.loaded) {
      await syncActiveTabView(
        index,
        tab.path,
        tab.content,
        tab.missing ? tab.missingMessage || "" : "",
      );
      return;
    }

    try {
      const content = await loadTabContent(tab.path);
      setLoadedTabState(index, content);
      await syncActiveTabView(index, tab.path, content);
    } catch (error) {
      const missingState = getMissingState(error);
      setMissingTabState(index, missingState);
      await syncActiveTabView(index, tab.path, "", missingState.missingMessage);
      if (!missingState.missing) showStatus(missingState.message, "error", 2200);
    }
  }

  async function syncTabWithDisk(index) {
    const tab = tabs[index];
    if (!tab) return;

    if (!tab.loaded) {
      await loadTab(index, true);
      return;
    }

    try {
      const content = normalizeNewlines(await loadTabContent(tab.path));
      const currentContent = normalizeNewlines(tab.content ?? "");

      if (!tab.missing && content === currentContent) {
        if (index === activeTabIndex) {
          missingFileMessage = "";
        }
        return;
      }

      setLoadedTabState(index, content);
      await syncActiveTabView(index, tab.path, content);
    } catch (error) {
      const missingState = getMissingState(error);
      const isUnchangedMissingState =
        missingState.missing &&
        tab.missing &&
        normalizeNewlines(tab.content ?? "") === "" &&
        (tab.missingMessage || "") === missingState.missingMessage;

      if (isUnchangedMissingState) {
        if (index === activeTabIndex) missingFileMessage = missingState.missingMessage;
        return;
      }

      setMissingTabState(index, missingState);
      await syncActiveTabView(index, tab.path, "", missingState.missingMessage);
      if (!missingState.missing) showStatus(missingState.message, "error", 2200);
    }
  }

  async function activateTab(index, forceReload = false) {
    saveScrollPosition();
    closeSearch();
    if (index !== activeTabIndex) {
      editorComponent?.finalizeBlock?.();
      await flushPendingSave(false);
    }

    activeTabIndex = index;
    await loadTab(index, forceReload);
  }

  async function saveTabByIndex(index, content, showConfirmation = true) {
    const tab = tabs[index];
    if (!tab) return;

    isSaving = true;

    try {
      await saveTabContent(tab.path, content);

      replaceTab(index, {
        content,
        missing: false,
        missingMessage: "",
        loaded: true,
      });

      if (index === activeTabIndex) {
        missingFileMessage = "";
      }

      showSavedIndicator = true;
      clearTimeout(savedIndicatorTimeout);
      // Keep the saved indicator visible briefly so the save feedback is noticeable.
      savedIndicatorTimeout = setTimeout(() => {
        showSavedIndicator = false;
      }, 1500);
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    } finally {
      isSaving = false;
    }
  }

  function scheduleSave(content = rawContent) {
    saveScheduler.schedule(
      {
        index: activeTabIndex,
        content,
      },
      (job) => {
        saveTabByIndex(job.index, job.content);
      },
    );
  }

  async function flushPendingSave(showConfirmation = false) {
    const job = saveScheduler.flush();
    if (!job) return;
    await saveTabByIndex(job.index, job.content, showConfirmation);
  }

  async function forceSave(showConfirmation = true) {
    if (!activeTab) return;

    saveScheduler.clear();

    const markdown = getCurrentMarkdown();
    const content = composeContentFromMarkdown(markdown, {
      strippedFrontmatter,
      hiddenBlockMap,
      codeblockMap,
    });
    rawContent = content;
    updateActiveTabContent(content);
    await saveTabByIndex(activeTabIndex, content, showConfirmation);
  }

  function closeTabContextMenu() {
    tabContextMenu = {
      open: false,
      x: 0,
      y: 0,
      tabIndex: -1,
    };
  }

  function openTabContextMenu(eventOrDetail, index) {
    const detail =
      typeof index === "number"
        ? { index, x: eventOrDetail.clientX, y: eventOrDetail.clientY }
        : eventOrDetail;
    const tab = tabs[detail.index];
    if (!tab || tab.kind !== "opened") {
      closeTabContextMenu();
      return;
    }

    eventOrDetail?.preventDefault?.();
    eventOrDetail?.stopPropagation?.();

    tabContextMenu = {
      open: true,
      x: detail.x,
      y: detail.y,
      tabIndex: detail.index,
    };
  }

  async function closeActiveTab() {
    if (!activeTab || activeTab.kind !== "opened" || activeTab.isPinned) return;

    saveScrollPosition();
    await flushPendingSave(false);
    scrollPositions.delete(activeTab.path);

    const nextTabs = tabs.filter((_, index) => index !== activeTabIndex);
    const nextIndex = Math.max(activeTabIndex - 1, 0);
    tabs = nextTabs;
    activeTabIndex = nextIndex;
    await loadTab(nextIndex);
  }

  async function closeTabByIndex(index) {
    const tab = tabs[index];
    if (!tab || tab.kind !== "opened") return;

    closeTabContextMenu();
    saveScrollPosition();
    await flushPendingSave(false);
    scrollPositions.delete(tab.path);

    const nextTabs = tabs.filter((_, tabIndex) => tabIndex !== index);
    const nextIndex =
      activeTabIndex > index
        ? activeTabIndex - 1
        : Math.min(activeTabIndex, nextTabs.length - 1);

    tabs = nextTabs;
    activeTabIndex = Math.max(nextIndex, 0);

    if (tabs[activeTabIndex]) {
      await loadTab(activeTabIndex);
    }
  }

  async function hideReader() {
    saveScrollPosition();
    editorComponent?.finalizeBlock?.();
    closeAutocomplete();
    closeSearch();
    isDragging = false;
    dragCounter = 0;
    await flushPendingSave(false);

    try {
      await invoke("hide_reader");
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    }
  }

  async function handleOpenInObsidian() {
    const tab = tabs[activeTabIndex];
    if (!tab?.path) return;

    try {
      await openInObsidian(
        tab.path,
        appSettings.vault_name ?? "Vault",
        appSettings.vault_path ?? "",
      );
    } catch (error) {
      showStatus("Failed to open in Obsidian", "error", 2200);
    }
  }

  function createOpenedNoteTab(note, history = []) {
    return createTab({
      kind: "opened",
      path: note.path,
      label: note.name,
      isPinned: false,
      history,
    });
  }

  async function applyOpenVaultNoteIntent(intent) {
    if (intent.action === "activateExisting") {
      await activateTab(intent.index, true);
      return true;
    }

    if (intent.action === "newTab") {
      tabs = [...tabs, createOpenedNoteTab(intent.note)];
      await activateTab(tabs.length - 1, true);
      return true;
    }

    return false;
  }

  async function handleNavigateToWikilink(target, forceNewTab = false) {
    try {
      vaultNotes = await ensureVaultNotes(vaultNotes);
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
      return;
    }

    const intent = navigateToWikilink(target, tabs, activeTabIndex, {
      vaultNotes,
      forceNewTab,
    });

    if (intent.action === "notFound") {
      showStatus(`Note not found: [[${target}]]`, "error", 2200);
      return;
    }

    if (intent.action === "openInObsidian") {
      showStatus("Opening in Obsidian...", "success", 1200);

      try {
        await openInObsidian(
          intent.note.path,
          appSettings.vault_name ?? "Vault",
          appSettings.vault_path ?? "",
        );
      } catch (error) {
        showStatus("Failed to open in Obsidian", "error", 2200);
      }

      return;
    }

    if (intent.action === "noop") {
      return;
    }

    saveScrollPosition();

    if (await applyOpenVaultNoteIntent(intent)) {
      return;
    }

    if (intent.action === "replaceCurrent") {
      await forceSave(false);
      replaceTab(activeTabIndex, {
        path: intent.note.path,
        label: intent.note.name,
        content: "",
        loaded: false,
        missing: false,
        missingMessage: "",
        history: intent.history,
        isPinned: false,
      });
      await loadTab(activeTabIndex, true);
    }
  }

  async function navigateBack() {
    const currentTab = tabs[activeTabIndex];
    if (!currentTab?.history?.length) return;

    saveScrollPosition();
    await forceSave(false);
    const previousPath = currentTab.history[currentTab.history.length - 1];
    const note = vaultNotes.find((entry) => entry.path === previousPath);

    replaceTab(activeTabIndex, {
      path: previousPath,
      label: note?.name ?? fileLabel(previousPath),
      content: "",
      loaded: false,
      missing: false,
      missingMessage: "",
      history: currentTab.history.slice(0, -1),
      isPinned: false,
    });
    await loadTab(activeTabIndex, true);
  }

  async function openPalette() {
    try {
      vaultNotes = await ensureVaultNotes(vaultNotes);
      showPalette = true;
      paletteQuery = "";
      selectedPaletteIndex = 0;
      await tick();
      paletteInputRef?.focus();
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    }
  }

  function closePalette() {
    showPalette = false;
    paletteQuery = "";
    selectedPaletteIndex = 0;
  }

  async function openVaultNote(note) {
    closePalette();
    await applyOpenVaultNoteIntent(getOpenVaultNoteIntent(note, tabs));
  }

  async function handleShowReader() {
    saveScrollPosition();
    editorComponent?.finalizeBlock?.();
    isDragging = false;
    dragCounter = 0;
    await flushPendingSave(false);
    if (tabs[activeTabIndex]) {
      await syncTabWithDisk(activeTabIndex);
    }
  }

  async function handleSettingsChanged(settings) {
    const previousFilters = getReaderFilterSettings(appSettings);
    const previousPinnedNotes = getPinnedNotesSignature(appSettings.pinned_notes);
    appSettings = applySettings(appSettings, settings);
    applyColorSettings(appSettings);
    const nextFilters = getReaderFilterSettings(appSettings);
    const nextPinnedNotes = getPinnedNotesSignature(appSettings.pinned_notes);
    const filtersChanged = haveReaderFilterChanges(previousFilters, nextFilters);

    if (previousPinnedNotes !== nextPinnedNotes) {
      await flushPendingSave(false);
      await applyTabSettings(settings, {
        preserveOpened: true,
        forceReloadActive: false,
      });
      return;
    }

    if (filtersChanged) {
      editorComponent?.finalizeBlock?.();
      imagePathCache.clear();
      await renderContentToEditor(rawContent);
    }
  }

  async function applyTabSettings(
    settings,
    { preserveOpened = true, forceReloadActive = false } = {},
  ) {
    const dailyPath = await getDailyNotePath(settings);
    const nextState = rebuildTabsFromSettings(settings, tabs, {
      preserveOpened,
      previousActivePath: activeTab?.path ?? null,
      dailyPath,
    });

    tabs = nextState.tabs;
    activeTabIndex = nextState.activeTabIndex;

    const currentTab = tabs[activeTabIndex];
    if (!currentTab) return;

    if (forceReloadActive || !currentTab.loaded) {
      await loadTab(activeTabIndex, true);
      return;
    }

    await loadEditorContent(currentTab.content);
    missingFileMessage = currentTab.missing
      ? currentTab.missingMessage || ""
      : "";
  }

  async function buildInitialTabs() {
    const settings = await invoke("load_settings");
    appSettings = applySettings(appSettings, settings);
    applyColorSettings(appSettings);
    await applyTabSettings(settings, {
      preserveOpened: false,
      forceReloadActive: true,
    });
  }

  // TODO: move to ReaderEditor once window-wide drag/drop can be preserved cleanly.

  onMount(async () => {
    try {
      await buildInitialTabs();
      vaultNotes = await ensureVaultNotes(vaultNotes);

      unlistenShowReader = await listen("show_reader", handleShowReader);

      unlistenSettingsChanged = await listen(
        "settings_changed",
        (event) => handleSettingsChanged(event.payload),
      );

      cleanupGlobalListeners = setupListeners({
        isTabContextMenuOpen: () => tabContextMenu.open,
        isAutocompleteOpen: () => showAutocomplete,
        isSearchOpen: () => showSearch,
        isPaletteOpen: () => showPalette,
        hasTabAtIndex: (index) => Boolean(tabs[index]),
        onCloseTabContextMenu: closeTabContextMenu,
        onCloseActiveTab: closeActiveTab,
        onOpenPalette: openPalette,
        onFocusSearch: () => searchInputRef?.focus(),
        onSearch: openSearch,
        onSave: forceSave,
        onActivateTab: activateTab,
        onCloseAutocomplete: closeAutocomplete,
        onCloseSearch: closeSearch,
        onClosePalette: closePalette,
        onCloseReader: hideReader,
      });
    } catch (error) {
      showStatus(normalizeError(error), "error", 2400);
    }
  });

  onDestroy(() => {
    saveScheduler.clear();
    clearTimeout(statusTimeout);
    clearTimeout(savedIndicatorTimeout);
    clearHighlights();
    cleanupGlobalListeners?.();
    unlistenShowReader?.();
    unlistenSettingsChanged?.();
  });
</script>

<div
  class="reader-container"
  class:palette-open={showPalette}
  class:dragging={isDragging}
  style="
    --app-background: {appSettings.background_color};
    --app-font-family: {appSettings.font_family};
    --app-font-size: {appSettings.font_size}px;
    --app-border-radius: {appSettings.border_radius}px;
    --app-transparency: {appSettings.window_transparency}%;
    --app-blur: {appSettings.window_blur}px;
    --app-saturation: {appSettings.window_saturation}%;
    --app-text-color: {appSettings.text_color};
    --app-brightness-filter: {brightnessFilter};
  "
  on:dragenter={handleDragEnter}
  on:dragleave={handleDragLeave}
  on:dragover={handleDragOver}
  on:drop={handleEditorDrop}
  role="application"
>
  <ReaderTopBar
    {tabs}
    {activeTabIndex}
    {isSaving}
    {isImportingImages}
    {showSavedIndicator}
    canGoBack={tabs[activeTabIndex]?.history?.length > 0}
    canOpenInObsidian={Boolean(tabs[activeTabIndex]?.path)}
    on:activateTab={(event) => activateTab(event.detail)}
    on:newTab={openPalette}
    on:goBack={navigateBack}
    on:openInObsidian={handleOpenInObsidian}
    on:closeReader={hideReader}
    on:tabContextMenu={(event) => openTabContextMenu(event.detail)}
  />

  <SearchBar
    open={showSearch}
    query={searchQuery}
    matchCount={searchMatches.length}
    activeIndex={searchIndex}
    bind:inputRef={searchInputRef}
    on:queryChange={(event) => runSearch(event.detail)}
    on:step={(event) => stepSearch(event.detail)}
    on:close={closeSearch}
  />

  <TabContextMenu
    open={tabContextMenu.open}
    x={tabContextMenu.x}
    y={tabContextMenu.y}
    on:close={closeTabContextMenu}
    on:closeTab={() => closeTabByIndex(tabContextMenu.tabIndex)}
  />

  <ReaderEditor
    bind:this={editorComponent}
    {rawContent}
    {appSettings}
    {vaultNotes}
    {showSearch}
    {showAutocomplete}
    {autocompleteResults}
    {autocompleteIndex}
    missingFileMessage={fileMissing ? missingFileMessage : ""}
    {codeblockMap}
    {hiddenBlockMap}
    setStrippedFrontmatter={(value) => {
      strippedFrontmatter = value;
    }}
    on:change={handleEditorChange}
    on:saveRequest={handleEditorSaveRequest}
    on:openSearchRequest={openSearch}
    on:closeSearchRequest={closeSearch}
    on:openPaletteRequest={openPalette}
    on:closeRequest={hideReader}
    on:navigateWikilink={(event) =>
      handleNavigateToWikilink(event.detail.target, event.detail.newTab)}
    on:openExternalLink={(event) => openExternalUrl(event.detail)}
    on:importImages={(event) => handleImportedImages(event.detail)}
    on:autocompleteChange={handleEditorAutocompleteChange}
    on:autocompleteIndexChange={(event) => {
      autocompleteIndex = event.detail;
    }}
    on:scroll={saveScrollPosition}
  />

  {#if isDragging}
    <div class="drop-overlay"></div>
  {/if}

  <CommandPalette
    open={showPalette}
    query={paletteQuery}
    notes={filteredVaultNotes}
    selectedIndex={selectedPaletteIndex}
    bind:inputRef={paletteInputRef}
    on:queryChange={(event) => {
      paletteQuery = event.detail.currentTarget.value;
    }}
    on:selectIndex={(event) => {
      selectedPaletteIndex = event.detail;
    }}
    on:openNote={(event) => openVaultNote(event.detail)}
    on:close={closePalette}
  />

  <StatusToast message={statusMessage} type={statusType} />
</div>

{#if showAutocomplete && autocompleteResults.length > 0}
  {@const rect = autocompleteRange?.getBoundingClientRect()}
  <div
    class="autocomplete-dropdown"
    style={`left: ${rect?.left ?? 0}px; top: ${(rect?.bottom ?? 0) + 4}px;`}
  >
    {#each autocompleteResults as note, index (note.path)}
      <button
        class="autocomplete-item"
        class:selected={index === autocompleteIndex}
        type="button"
        on:mousedown|preventDefault={() =>
          editorComponent?.insertAutocompleteResult?.(note)}
        on:mouseenter={() => (autocompleteIndex = index)}
      >
        <span class="autocomplete-name">{note.name}</span>
        <span class="autocomplete-path">{note.relative_path}</span>
      </button>
    {/each}
  </div>
{/if}
