<script>
  import { run, preventDefault } from 'svelte/legacy';

  import { ENABLE_DEMO_FAKE_BG } from "./lib/demo-fake-bg.js";
  const DEMO_FAKE_BG = import.meta.env.DEV && ENABLE_DEMO_FAKE_BG;

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
  import { imagePathCache } from "./lib/reader/contentProcessing.js";
  import * as editorBridge from "./lib/reader/editorBridge.js";
  import * as tabLifecycle from "./lib/reader/tabLifecycle.js";
  import * as tabSettingsSync from "./lib/reader/tabSettingsSync.js";
  import { setupListeners } from "./lib/reader/lifecycleSetup.js";
  import * as navigationGlue from "./lib/reader/navigationGlue.js";
  import { ensureVaultNotes } from "./lib/reader/navigation.js";
  import { createDebouncedJob } from "./lib/reader/saveLoadPipeline.js";
  import { computeBrightnessFilter } from "./lib/reader/settingsBridge.js";
  import {
    applySearchHighlights,
    clearSearchHighlights,
    runSearch as collectSearchMatches,
    stepSearch as getNextSearchIndex,
  } from "./lib/reader/searchLogic.js";
  import { getDailyNotePath } from "./lib/reader/tabIO.js";
  import { filterPaletteNotes } from "./lib/reader/paletteLogic.js";
  import { defaultSettings } from "./lib/stores.js";
  import { normalizeError } from "./lib/reader/errorHandling.js";

  let tabs = $state([]);
  let activeTabIndex = $state(0);
  let showPalette = $state(false);
  let paletteQuery = $state("");
  let vaultNotes = $state([]);
  let isSaving = false;
  let rawContent = $state("");
  let strippedFrontmatter = $state("");
  let codeblockMap = new Map();
  let hiddenBlockMap = new Map();
  const scrollPositions = new Map();
  let appSettings = $state({ ...defaultSettings });
  let statusMessage = $state("");
  let statusType = $state("");
  let missingFileMessage = $state("");
  let selectedPaletteIndex = $state(0);
  // (showSavedIndicator removed)
  let showAutocomplete = $state(false);
  let autocompleteIndex = $state(0);
  let autocompleteResults = $state([]);
  let autocompleteRange = $state(null);
  let showSearch = $state(false);
  let searchQuery = $state("");
  let searchMatches = $state([]);
  let searchIndex = $state(0);
  let searchInputRef = $state();
  let tabContextMenu = $state({
    open: false,
    x: 0,
    y: 0,
    tabIndex: -1,
  });

  let editorComponent = $state();
  let paletteInputRef = $state();
  let statusTimeout;
  let unlistenShowReader;
  let unlistenSettingsChanged;
  let cleanupGlobalListeners = () => {};
  const saveScheduler = createDebouncedJob(600);

  let activeTab = $derived(tabs[activeTabIndex] ?? null);
  let fileMissing = $derived(missingFileMessage.trim() !== "");
  let showNotePaths = $derived(appSettings?.show_note_paths ?? true);
  let autocompleteResultLimit = $derived(appSettings?.autocomplete_results ?? 20);
  let filteredVaultNotes = $derived(filterPaletteNotes(
    vaultNotes,
    paletteQuery,
    autocompleteResultLimit,
  ));
  run(() => {
    if (selectedPaletteIndex >= filteredVaultNotes.length) {
      selectedPaletteIndex = Math.max(filteredVaultNotes.length - 1, 0);
    }
  });
  let brightnessFilter = $derived(computeBrightnessFilter(appSettings.window_brightness));

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

  function editorBridgeBag() {
    return {
      getEditorComponent: () => editorComponent,
      getRawContent: () => rawContent,
      setRawContent: (value) => { rawContent = value; },
      getAppSettings: () => appSettings,
      codeblockMap,
      hiddenBlockMap,
      getStrippedFrontmatter: () => strippedFrontmatter,
      setStrippedFrontmatter: (value) => { strippedFrontmatter = value; },
      closeAutocomplete,
      closeSearch,
      getVaultNotes: () => vaultNotes,
      getAutocompleteResultLimit: () => autocompleteResultLimit,
      setShowAutocomplete: (value) => { showAutocomplete = value; },
      setAutocompleteResults: (value) => { autocompleteResults = value; },
      setAutocompleteRange: (value) => { autocompleteRange = value; },
      setAutocompleteIndex: (value) => { autocompleteIndex = value; },
      updateActiveTabContent: (content) => {
        if (!activeTab) return;
        tabs = tabLifecycle.applyTabUpdate(tabs, activeTabIndex, { content, loaded: true });
      },
      scheduleSave: (content) => tabLifecycle.scheduleSave(content, tabLifecycleBag()),
      forceSave: () => tabLifecycle.forceSave(true, tabLifecycleBag()),
    };
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
    editorComponent?.focus();
  }

  async function openSearch() {
    showSearch = true;
    await tick();
    searchInputRef?.focus();
    searchInputRef?.select();
  }

  function saveScrollPosition() {
    tabLifecycle.saveScrollPosition({
      getActiveTab: () => activeTab,
      getEditorComponent: () => editorComponent,
      scrollPositions,
    });
  }

  function handleEditorChange() {
    editorBridge.handleEditorChange(editorBridgeBag());
  }

  async function handleEditorSaveRequest() {
    await editorBridge.handleEditorSaveRequest(editorBridgeBag());
  }

  function handleEditorAutocompleteChange(detail) {
    editorBridge.handleEditorAutocompleteChange({ detail }, editorBridgeBag());
  }

  async function openExternalUrl(url) {
    try {
      await invoke("open_external_url", { url });
    } catch (error) {
      showStatus("Failed to open link", "error", 2200);
    }
  }

  function tabLifecycleBag() {
    return {
      getTabs: () => tabs,
      setTabs: (value) => { tabs = value; },
      getActiveTab: () => activeTab,
      getActiveTabIndex: () => activeTabIndex,
      setActiveTabIndex: (value) => { activeTabIndex = value; },
      getEditorComponent: () => editorComponent,
      scrollPositions,
      closeSearch,
      closeTabContextMenu,
      loadEditorContent: (raw) => editorBridge.loadEditorContent(raw, editorBridgeBag()),
      renderContentToEditor: (raw) => editorBridge.renderContentToEditor(raw, editorBridgeBag()),
      getCurrentSaveContent: () => editorBridge.getCurrentSaveContent(editorBridgeBag()),
      setRawContent: (value) => { rawContent = value; },
      setMissingFileMessage: (value) => { missingFileMessage = value; },
      setIsSaving: (value) => { isSaving = value; },
      showStatus,
      saveScheduler,
    };
  }

  function tabSettingsSyncBag() {
    return {
      getTabs: () => tabs,
      setTabs: (value) => { tabs = value; },
      getActiveTab: () => activeTab,
      getActiveTabIndex: () => activeTabIndex,
      setActiveTabIndex: (value) => { activeTabIndex = value; },
      getAppSettings: () => appSettings,
      setAppSettings: (value) => { appSettings = value; },
      getRawContent: () => rawContent,
      getEditorComponent: () => editorComponent,
      loadTab: (index, force) => tabLifecycle.loadTab(index, force, tabLifecycleBag()),
      loadEditorContent: (raw) => editorBridge.loadEditorContent(raw, editorBridgeBag()),
      renderContentToEditor: (raw) => editorBridge.renderContentToEditor(raw, editorBridgeBag()),
      flushPendingSave: (confirm) => tabLifecycle.flushPendingSave(confirm, tabLifecycleBag()),
      setMissingFileMessage: (value) => { missingFileMessage = value; },
      invoke,
    };
  }

  function navigationGlueBag() {
    return {
      getTabs: () => tabs,
      setTabs: (value) => { tabs = value; },
      getActiveTab: () => activeTab,
      getActiveTabIndex: () => activeTabIndex,
      getVaultNotes: () => vaultNotes,
      setVaultNotes: (value) => { vaultNotes = value; },
      getAppSettings: () => appSettings,
      activateTab: (index, force) => tabLifecycle.activateTab(index, force, tabLifecycleBag()),
      loadTab: (index, force) => tabLifecycle.loadTab(index, force, tabLifecycleBag()),
      forceSave: (confirm) => tabLifecycle.forceSave(confirm, tabLifecycleBag()),
      saveScrollPosition,
      setShowPalette: (value) => { showPalette = value; },
      setPaletteQuery: (value) => { paletteQuery = value; },
      setSelectedPaletteIndex: (value) => { selectedPaletteIndex = value; },
      focusPaletteInput: () => paletteInputRef?.focus(),
      focusEditor: () => editorComponent?.focus(),
      showStatus,
    };
  }

  async function syncTabWithDisk(index) {
    await tabLifecycle.syncTabWithDisk(index, tabLifecycleBag());
  }

  async function activateTab(index, forceReload = false) {
    await tabLifecycle.activateTab(index, forceReload, tabLifecycleBag());
  }

  async function flushPendingSave(showConfirmation = false) {
    await tabLifecycle.flushPendingSave(showConfirmation, tabLifecycleBag());
  }

  async function forceSave(showConfirmation = true) {
    await tabLifecycle.forceSave(showConfirmation, tabLifecycleBag());
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
    await tabLifecycle.closeActiveTab(tabLifecycleBag());
  }

  async function closeTabByIndex(index) {
    await tabLifecycle.closeTabByIndex(index, tabLifecycleBag());
  }

  async function hideReader() {
    saveScrollPosition();
    editorComponent?.finalizeBlock?.();
    closeAutocomplete();
    closeSearch();
    await flushPendingSave(false);

    try {
      await invoke("hide_reader");
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    }

    imagePathCache.clear();
  }

  async function openSettings() {
    try {
      await invoke("open_settings");
    } catch (e) {
      console.error("Failed to open settings:", e);
    }
  }

  async function handleOpenInObsidian() {
    await navigationGlue.handleOpenInObsidian(navigationGlueBag());
  }

  function canCreateMissingDailyNote() {
    const tab = tabs[activeTabIndex];
    return (
      tab?.kind === "daily" &&
      tab.missing &&
      appSettings?.daily_note_create_if_missing
    );
  }

  async function handleCreateDailyNote() {
    if (!canCreateMissingDailyNote()) return;
    if (isSaving) return;

    isSaving = true;

    try {
      await invoke("append_to_daily_note", { text: "" });
      await syncTabWithDisk(activeTabIndex);
      showStatus("Daily note created", "success", 1600);
    } catch (error) {
      showStatus(normalizeError(error), "error", 2400);
    } finally {
      isSaving = false;
    }
  }

  async function handleNavigateToWikilink(target, forceNewTab = false) {
    await navigationGlue.handleNavigateToWikilink(target, forceNewTab, navigationGlueBag());
  }

  async function navigateBack() {
    await navigationGlue.navigateBack(navigationGlueBag());
  }

  function openPalette() {
    navigationGlue.openPalette(navigationGlueBag());
  }

  function closePalette() {
    navigationGlue.closePalette(navigationGlueBag());
  }

  async function openVaultNote(note) {
    await navigationGlue.openVaultNote(note, navigationGlueBag());
  }

  async function handleShowReader() {
    saveScrollPosition();
    editorComponent?.finalizeBlock?.();
    await flushPendingSave(false);
    try {
      const currentDailyPath = await getDailyNotePath(appSettings);
      const dailyTab = tabs.find((tab) => tab.kind === "daily");
      if (dailyTab && currentDailyPath !== dailyTab.path) {
        await applyTabSettings(appSettings, {
          preserveOpened: true,
          forceReloadActive: false,
        });
      }
    } catch (error) {
      console.error("Failed to check daily note rollover:", error);
    }
    if (tabs[activeTabIndex]) {
      await syncTabWithDisk(activeTabIndex);
    }
  }

  async function applyTabSettings(settings, options = {}) {
    await tabSettingsSync.applyTabSettings(settings, options, tabSettingsSyncBag());
  }

  async function handleSettingsChanged(settings) {
    await tabSettingsSync.handleSettingsChanged(settings, tabSettingsSyncBag());
  }

  async function buildInitialTabs() {
    await tabSettingsSync.buildInitialTabs(tabSettingsSyncBag());
  }

  // TODO: move to ReaderEditor once window-wide drag/drop can be preserved cleanly.

  onMount(async () => {
    try {
      await buildInitialTabs();
      ensureVaultNotes(vaultNotes)
        .then((notes) => {
          vaultNotes = notes;
        })
        .catch((error) => {
          showStatus(normalizeError(error), "error", 2400);
        });

      unlistenShowReader = await listen("show_reader", handleShowReader);

      unlistenSettingsChanged = await listen(
        "settings_changed",
        (event) => handleSettingsChanged(event.payload),
      );

      cleanupGlobalListeners = setupListeners(
        {
          isTabContextMenuOpen: () => tabContextMenu.open,
          isAutocompleteOpen: () => showAutocomplete,
          isSearchOpen: () => showSearch,
          isPaletteOpen: () => showPalette,
          hasTabAtIndex: (index) => Boolean(tabs[index]),
          onCloseTabContextMenu: closeTabContextMenu,
          onCloseActiveTab: closeActiveTab,
          onFocusSearch: () => searchInputRef?.focus(),
          onSearch: openSearch,
          onSave: forceSave,
          onActivateTab: activateTab,
          onCloseAutocomplete: closeAutocomplete,
          onCloseSearch: closeSearch,
          onClosePalette: closePalette,
          onCloseReader: hideReader,
          onCreateDailyNote: handleCreateDailyNote,
        },
        [
          {
            getShortcut: () => appSettings.save_to_daily_shortcut,
            shouldHandle: canCreateMissingDailyNote,
            onMatch: handleCreateDailyNote,
          },
          {
            getShortcut: () => appSettings.reader_open_in_obsidian_shortcut,
            onMatch: handleOpenInObsidian,
          },
          {
            getShortcut: () => appSettings.reader_navigate_back_shortcut,
            onMatch: navigateBack,
          },
          {
            getShortcut: () => appSettings.reader_command_palette_shortcut,
            onMatch: openPalette,
          },
        ],
      );
    } catch (error) {
      showStatus(normalizeError(error), "error", 2400);
    }
  });

  onDestroy(() => {
    saveScheduler.clear();
    clearTimeout(statusTimeout);
    clearHighlights();
    cleanupGlobalListeners?.();
    unlistenShowReader?.();
    unlistenSettingsChanged?.();
  });
</script>

<div
  class="reader-container"
  class:demo-fake-bg={DEMO_FAKE_BG}
  class:palette-open={showPalette}
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
  role="application"
>
  <ReaderTopBar
    {tabs}
    {activeTabIndex}
    canOpenInObsidian={Boolean(tabs[activeTabIndex]?.path)}
    on:activateTab={(event) => activateTab(event.detail)}
    on:newTab={openPalette}
    on:openInObsidian={handleOpenInObsidian}
    on:closeReader={hideReader}
    on:closeTab={(event) => closeTabByIndex(event.detail)}
    on:tabContextMenu={(event) => openTabContextMenu(event.detail)}
  />

  <SearchBar
    open={showSearch}
    query={searchQuery}
    matchCount={searchMatches.length}
    activeIndex={searchIndex}
    bind:inputRef={searchInputRef}
    onQueryChange={(event) => runSearch(event)}
    onStep={(direction) => stepSearch(direction)}
    onClose={closeSearch}
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
    activeTabKind={activeTab?.kind ?? ""}
    {autocompleteResults}
    {autocompleteIndex}
    missingFileMessage={fileMissing ? missingFileMessage : ""}
    {codeblockMap}
    {hiddenBlockMap}
    setStrippedFrontmatter={(value) => {
      strippedFrontmatter = value;
    }}
    onChange={handleEditorChange}
    onSaveRequest={handleEditorSaveRequest}
    onOpenSearchRequest={openSearch}
    onCloseSearchRequest={closeSearch}
    onOpenSettingsRequest={openSettings}
    onCloseRequest={hideReader}
    onNavigateWikilink={(detail) =>
      handleNavigateToWikilink(detail.target, detail.newTab)}
    onOpenExternalLink={(href) => openExternalUrl(href)}
    onAutocompleteChange={handleEditorAutocompleteChange}
    onAutocompleteIndexChange={(index) => {
      autocompleteIndex = index;
    }}
    onScroll={saveScrollPosition}
  />

  <CommandPalette
    open={showPalette}
    query={paletteQuery}
    notes={filteredVaultNotes}
    selectedIndex={selectedPaletteIndex}
    showPaths={showNotePaths}
    bind:inputRef={paletteInputRef}
    onQueryChange={(event) => {
      paletteQuery = event.currentTarget.value;
    }}
    onSelectIndex={(index) => {
      selectedPaletteIndex = index;
    }}
    onOpenNote={(note) => openVaultNote(note)}
    onClose={closePalette}
  />

  <!-- Bottom bar disabled. Re-enable when back navigation is needed. -->
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
        onmousedown={preventDefault(() =>
          editorComponent?.insertAutocompleteResult?.(note))}
        onmouseenter={() => (autocompleteIndex = index)}
      >
        <span class="autocomplete-name">{note.name}</span>
        {#if showNotePaths}
          <span class="autocomplete-path">{note.relative_path}</span>
        {/if}
      </button>
    {/each}
  </div>
{/if}

<style>
  .reader-container.demo-fake-bg {
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
