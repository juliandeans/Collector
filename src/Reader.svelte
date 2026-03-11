<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  let tabs = [];
  let activeTabIndex = 0;
  let blocks = [""];
  let focusedBlock = null;
  let showPalette = false;
  let paletteQuery = "";
  let vaultNotes = [];
  let isSaving = false;
  let lastSaved = null;
  let rawContent = "";
  let visibleBlocks = [];
  let appSettings = {
    background_color: "#1e1e2e",
    font_family: "-apple-system, BlinkMacSystemFont, SF Pro Display",
    font_size: 15,
    border_radius: 12,
    window_transparency: 55,
    window_blur: 80,
    window_saturation: 200,
    window_brightness: 0,
    text_color: "#ffffff",
    pinned_notes: [],
    reader_hide_frontmatter: true,
    reader_hide_dataview: true,
    reader_hide_obsidian_comments: true,
  };
  let statusMessage = "";
  let statusType = "";
  let missingFileMessage = "";
  let selectedPaletteIndex = 0;
  let showSavedIndicator = false;

  let paletteInputRef;
  let blockInputRefs = [];
  let saveTimeout;
  let pendingSave = null;
  let statusTimeout;
  let savedIndicatorTimeout;
  let unlistenShowReader;
  let unlistenSettingsChanged;

  $: activeTab = tabs[activeTabIndex] ?? null;
  $: filteredVaultNotes = vaultNotes.filter((note) => {
    const query = paletteQuery.trim().toLowerCase();
    if (!query) return true;
    return (
      note.name.toLowerCase().includes(query) ||
      note.relative_path.toLowerCase().includes(query)
    );
  });
  $: if (selectedPaletteIndex >= filteredVaultNotes.length) {
    selectedPaletteIndex = Math.max(filteredVaultNotes.length - 1, 0);
  }
  $: brightnessFilter = (() => {
    const b = appSettings.window_brightness;
    if (b === 0) return "";
    if (b > 0) {
      const brightnessValue = 1 + (b / 100) * 0.6;
      const contrastValue = 1 - (b / 100) * 0.25;
      return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
    }
    const brightnessValue = 1 + (b / 100) * 0.7;
    const contrastValue = 1 + (-b / 100) * 0.3;
    return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
  })();

  function normalizeNewlines(content = "") {
    return content.replace(/\r\n/g, "\n");
  }

  function parseRawBlocks(content = "") {
    const normalized = normalizeNewlines(content);
    if (!normalized) return [""];

    const lines = normalized.split("\n");
    const parsedBlocks = [];
    let current = [];
    let inFrontmatter = false;
    let inCodeBlock = false;
    let inObsidianComment = false;

    const pushCurrent = () => {
      if (current.length === 0) return;
      parsedBlocks.push(current.join("\n"));
      current = [];
    };

    for (let index = 0; index < lines.length; index += 1) {
      const line = lines[index];
      const trimmed = line.trim();

      if (index === 0 && trimmed === "---") {
        pushCurrent();
        current.push(line);
        inFrontmatter = true;
        continue;
      }

      if (inFrontmatter) {
        current.push(line);
        if (trimmed === "---" && current.length > 1) {
          pushCurrent();
          inFrontmatter = false;
        }
        continue;
      }

      if (!inCodeBlock && !inObsidianComment && trimmed === "") {
        pushCurrent();
        continue;
      }

      current.push(line);

      if (!inObsidianComment && trimmed.startsWith("```")) {
        inCodeBlock = !inCodeBlock;
        continue;
      }

      if (!inCodeBlock && trimmed.startsWith("%%")) {
        if (inObsidianComment) {
          inObsidianComment = false;
        } else if (!trimmed.endsWith("%%") || trimmed === "%%") {
          inObsidianComment = true;
        }
        continue;
      }

      if (inObsidianComment && trimmed.endsWith("%%")) {
        inObsidianComment = false;
      }
    }

    pushCurrent();
    return parsedBlocks.length > 0 ? parsedBlocks : [""];
  }

  function isFrontmatterBlock(block, index) {
    const trimmed = block.trim();
    return index === 0 && trimmed.startsWith("---") && trimmed.endsWith("---");
  }

  function isCodeBlock(block) {
    const trimmed = block.trim();
    return trimmed.startsWith("```") && trimmed.endsWith("```");
  }

  function isObsidianComment(block) {
    const trimmed = block.trim();
    return trimmed.startsWith("%%") && trimmed.endsWith("%%");
  }

  function getVisibleBlocksFromRaw(content = rawContent) {
    const allBlocks = parseRawBlocks(content);
    const nextVisibleBlocks = [];

    allBlocks.forEach((block, originalIndex) => {
      const isHidden =
        (appSettings.reader_hide_frontmatter &&
          isFrontmatterBlock(block, originalIndex)) ||
        (appSettings.reader_hide_dataview && isCodeBlock(block)) ||
        (appSettings.reader_hide_obsidian_comments && isObsidianComment(block));

      if (!isHidden) {
        nextVisibleBlocks.push({ content: block, originalIndex });
      }
    });

    return { allBlocks, nextVisibleBlocks };
  }

  function getContentForSave(nextBlocks = blocks) {
    const allBlocks = parseRawBlocks(rawContent);

    visibleBlocks.forEach((visibleBlock, index) => {
      const replacement = nextBlocks[index] ?? "";
      if (visibleBlock.originalIndex >= allBlocks.length) {
        allBlocks.push(replacement);
      } else {
        allBlocks[visibleBlock.originalIndex] = replacement;
      }
    });

    return allBlocks.join("\n\n");
  }

  function loadContent(raw = "") {
    rawContent = normalizeNewlines(raw);
    const { allBlocks, nextVisibleBlocks } =
      getVisibleBlocksFromRaw(rawContent);

    visibleBlocks =
      nextVisibleBlocks.length > 0
        ? nextVisibleBlocks
        : [{ content: "", originalIndex: allBlocks.length }];

    blocks = visibleBlocks.map((block) => block.content);
    focusedBlock = null;
    blockInputRefs = [];
  }

  function autosize(node) {
    const resize = () => {
      node.style.height = "auto";
      node.style.height = `${node.scrollHeight}px`;
    };

    node.addEventListener("input", resize);
    setTimeout(resize, 0);

    return {
      update: resize,
      destroy() {
        node.removeEventListener("input", resize);
      },
    };
  }

  function fileLabel(path) {
    const filename = path.split("/").pop() || path;
    return filename.replace(/\.md$/i, "");
  }

  function normalizePinnedNotes(pinnedNotes = []) {
    return pinnedNotes
      .map((entry) => {
        if (typeof entry === "string") {
          return {
            path: entry,
            label: "",
            icon: "",
          };
        }

        return {
          path: entry?.path ?? "",
          label: entry?.label ?? "",
          icon: entry?.icon ?? "",
        };
      })
      .filter((entry) => entry.path.trim() !== "")
      .map((entry) => ({
        path: entry.path,
        label: entry.label.trim(),
        icon: entry.icon.trim(),
      }));
  }

  function getPinnedNotesSignature(pinnedNotes = []) {
    return JSON.stringify(
      normalizePinnedNotes(pinnedNotes).map((note) => ({
        path: note.path,
        label: note.label,
        icon: note.icon,
      })),
    );
  }

  function createTab({
    kind = "opened",
    path,
    label = "",
    icon = "",
    existingTab = null,
  }) {
    const fallbackLabel = kind === "daily" ? "Daily" : fileLabel(path);

    return {
      kind,
      path,
      label: label.trim() || fallbackLabel,
      icon: icon.trim(),
      content: existingTab?.content ?? "",
      loaded: existingTab?.loaded ?? false,
      missing: existingTab?.missing ?? false,
      missingMessage: existingTab?.missingMessage ?? "",
    };
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
    statusTimeout = setTimeout(() => {
      statusMessage = "";
      statusType = "";
    }, duration);
  }

  function applySettings(settings) {
    appSettings = {
      ...appSettings,
      background_color:
        settings.background_color ?? appSettings.background_color,
      font_family: settings.font_family ?? appSettings.font_family,
      font_size: settings.font_size ?? appSettings.font_size,
      border_radius: settings.border_radius ?? appSettings.border_radius,
      window_transparency:
        settings.window_transparency ?? appSettings.window_transparency,
      window_blur: settings.window_blur ?? appSettings.window_blur,
      window_saturation:
        settings.window_saturation ?? appSettings.window_saturation,
      window_brightness:
        settings.window_brightness ?? appSettings.window_brightness,
      text_color: settings.text_color ?? appSettings.text_color,
      pinned_notes: normalizePinnedNotes(
        settings.pinned_notes ?? appSettings.pinned_notes,
      ),
      reader_hide_frontmatter:
        settings.reader_hide_frontmatter ?? appSettings.reader_hide_frontmatter,
      reader_hide_dataview:
        settings.reader_hide_dataview ?? appSettings.reader_hide_dataview,
      reader_hide_obsidian_comments:
        settings.reader_hide_obsidian_comments ??
        appSettings.reader_hide_obsidian_comments,
    };
  }

  function getReaderFilterSettings(settings = appSettings) {
    return {
      reader_hide_frontmatter: settings.reader_hide_frontmatter,
      reader_hide_dataview: settings.reader_hide_dataview,
      reader_hide_obsidian_comments: settings.reader_hide_obsidian_comments,
    };
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

  function updateBlocks(nextBlocks, shouldScheduleSave = true) {
    blocks = nextBlocks.length > 0 ? nextBlocks : [""];
    rawContent = getContentForSave(blocks);
    updateActiveTabContent(rawContent);
    if (shouldScheduleSave) {
      scheduleSave();
    }
  }

  async function loadTab(index, forceReload = false) {
    const tab = tabs[index];
    if (!tab) return;

    focusedBlock = null;
    blockInputRefs = [];

    if (!forceReload && tab.loaded) {
      loadContent(tab.content);
      missingFileMessage = tab.missing ? tab.missingMessage || "" : "";
      return;
    }

    try {
      const content = await invoke("read_note_file", { path: tab.path });
      replaceTab(index, {
        content,
        loaded: true,
        missing: false,
        missingMessage: "",
      });
      if (index === activeTabIndex) {
        loadContent(content);
        missingFileMessage = "";
      }
    } catch (error) {
      const message = normalizeError(error);
      const missingMessage = isFileMissingError(error)
        ? "File not found - will be created on first save"
        : message;

      replaceTab(index, {
        content: "",
        loaded: true,
        missing: isFileMissingError(error),
        missingMessage,
      });

      if (index === activeTabIndex) {
        loadContent("");
        missingFileMessage = missingMessage;
      }

      if (!isFileMissingError(error)) {
        showStatus(message, "error", 2200);
      }
    }
  }

  async function activateTab(index, forceReload = false) {
    if (index !== activeTabIndex) {
      await flushPendingSave(false);
    }
    activeTabIndex = index;
    await loadTab(index, forceReload);
  }

  async function reloadCurrentTab() {
    if (!activeTab) return;
    await loadTab(activeTabIndex, true);
  }

  async function saveTabByIndex(index, content, showConfirmation = true) {
    const tab = tabs[index];
    if (!tab) return;

    isSaving = true;

    try {
      await invoke("write_note_file", {
        path: tab.path,
        content,
      });

      replaceTab(index, {
        content,
        missing: false,
        missingMessage: "",
        loaded: true,
      });
      if (index === activeTabIndex) {
        missingFileMessage = "";
      }
      lastSaved = new Date();
      showSavedIndicator = true;
      clearTimeout(savedIndicatorTimeout);
      savedIndicatorTimeout = setTimeout(() => {
        showSavedIndicator = false;
      }, 1500);
      if (showConfirmation) {
        showStatus("Saved ✓", "success");
      }
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    } finally {
      isSaving = false;
    }
  }

  async function saveCurrentTab(showConfirmation = true) {
    if (!activeTab) return;

    clearTimeout(saveTimeout);
    saveTimeout = null;
    pendingSave = null;

    const content = getContentForSave();
    updateActiveTabContent(content);
    rawContent = content;
    await saveTabByIndex(activeTabIndex, content, showConfirmation);
  }

  function scheduleSave() {
    clearTimeout(saveTimeout);
    const content = getContentForSave();
    rawContent = content;
    pendingSave = {
      index: activeTabIndex,
      content,
    };
    saveTimeout = setTimeout(() => {
      const job = pendingSave;
      saveTimeout = null;
      pendingSave = null;
      if (job) {
        saveTabByIndex(job.index, job.content);
      }
    }, 600);
  }

  async function flushPendingSave(showConfirmation = false) {
    if (!saveTimeout || !pendingSave) return;
    clearTimeout(saveTimeout);
    saveTimeout = null;
    const job = pendingSave;
    pendingSave = null;
    await saveTabByIndex(job.index, job.content, showConfirmation);
  }

  function focusBlock(index) {
    focusedBlock = index;
    setTimeout(() => {
      const input = blockInputRefs[index];
      if (input) {
        input.focus();
        input.setSelectionRange(input.value.length, input.value.length);
      }
    }, 0);
  }

  function blurBlock() {
    focusedBlock = null;
  }

  function handleBlockInput(index) {
    blocks = [...blocks];
    rawContent = getContentForSave(blocks);
    updateActiveTabContent(rawContent);
    scheduleSave();
  }

  function handleBlockKeydown(event, index) {
    if (event.key === "Escape") {
      event.preventDefault();
      focusedBlock = null;
      return;
    }

    if (event.metaKey && event.key.toLowerCase() === "s") {
      event.preventDefault();
      event.stopPropagation();
      saveCurrentTab();
    }
  }

  function escapeHtml(text = "") {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#39;");
  }

  function renderInlineMarkdown(text = "") {
    let html = escapeHtml(text);
    html = html.replace(/`([^`]+)`/g, "<code>$1</code>");
    html = html.replace(
      /\[([^\]]+)\]\((https?:\/\/[^\s)]+)\)/g,
      '<a href="$2" target="_blank" rel="noopener noreferrer">$1</a>',
    );
    html = html.replace(/\*\*([^*]+)\*\*/g, "<strong>$1</strong>");
    html = html.replace(/(^|[^*])\*([^*]+)\*/g, "$1<em>$2</em>");
    return html;
  }

  function parseHeading(line) {
    const match = line.match(/^(#{1,3})\s+(.*)$/);
    if (!match) return null;
    return {
      level: match[1].length,
      text: match[2],
    };
  }

  function parseCheckbox(line) {
    const match = line.match(/^- \[( |x|X)\]\s?(.*)$/);
    if (!match) return null;
    return {
      checked: match[1].toLowerCase() === "x",
      text: match[2],
    };
  }

  function isSpacerBlock(block) {
    return block.trim() === "";
  }

  function getBlockLines(block) {
    return block.split("\n").map((line) => {
      const heading = parseHeading(line);
      const checkbox = parseCheckbox(line);

      if (line.trim() === "") {
        return { type: "spacer" };
      }

      if (checkbox) {
        return {
          type: "checkbox",
          checked: checkbox.checked,
          html: renderInlineMarkdown(checkbox.text || "Task"),
        };
      }

      if (heading) {
        return {
          type: "heading",
          level: heading.level,
          html: renderInlineMarkdown(heading.text),
        };
      }

      return {
        type: "text",
        html: renderInlineMarkdown(line),
      };
    });
  }

  async function toggleCheckbox(blockIndex, lineIndex) {
    const nextBlocks = [...blocks];
    const blockLines = nextBlocks[blockIndex].split("\n");
    const checkbox = parseCheckbox(blockLines[lineIndex]);
    if (!checkbox) return;

    const marker = checkbox.checked ? " " : "x";
    blockLines[lineIndex] = `- [${marker}] ${checkbox.text}`;
    nextBlocks[blockIndex] = blockLines.join("\n");
    updateBlocks(nextBlocks, false);
    await saveCurrentTab();
  }

  async function closeActiveTab() {
    if (activeTabIndex === 0 || !activeTab) return;

    await flushPendingSave(false);

    const nextTabs = tabs.filter((_, index) => index !== activeTabIndex);
    const nextIndex = Math.max(activeTabIndex - 1, 0);
    tabs = nextTabs;
    activeTabIndex = nextIndex;
    await loadTab(nextIndex);
  }

  async function hideReader() {
    await flushPendingSave(false);

    try {
      await invoke("hide_reader");
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    }
  }

  async function ensureVaultNotes() {
    if (vaultNotes.length > 0) return;

    try {
      vaultNotes = await invoke("list_vault_notes");
    } catch (error) {
      showStatus(normalizeError(error), "error", 2200);
    }
  }

  async function openPalette() {
    await ensureVaultNotes();
    showPalette = true;
    paletteQuery = "";
    selectedPaletteIndex = 0;
    setTimeout(() => paletteInputRef?.focus(), 0);
  }

  function closePalette() {
    showPalette = false;
    paletteQuery = "";
    selectedPaletteIndex = 0;
  }

  async function openVaultNote(note) {
    const existingIndex = tabs.findIndex((tab) => tab.path === note.path);
    closePalette();

    if (existingIndex >= 0) {
      await activateTab(existingIndex, true);
      return;
    }

    tabs = [
      ...tabs,
      createTab({
        kind: "opened",
        path: note.path,
        label: note.name,
      }),
    ];

    await activateTab(tabs.length - 1, true);
  }

  function handlePaletteKeydown(event) {
    if (!showPalette) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (filteredVaultNotes.length > 0) {
        selectedPaletteIndex = Math.min(
          selectedPaletteIndex + 1,
          filteredVaultNotes.length - 1,
        );
      }
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      selectedPaletteIndex = Math.max(selectedPaletteIndex - 1, 0);
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      const note = filteredVaultNotes[selectedPaletteIndex];
      if (note) {
        openVaultNote(note);
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      closePalette();
    }
  }

  async function handleGlobalKeydown(event) {
    if (showPalette) {
      handlePaletteKeydown(event);
      if (["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(event.key)) {
        return;
      }
    }

    if (event.metaKey && event.key.toLowerCase() === "k") {
      event.preventDefault();
      await openPalette();
      return;
    }

    if (event.metaKey && event.key.toLowerCase() === "w") {
      event.preventDefault();
      await closeActiveTab();
      return;
    }

    if (event.metaKey && event.key.toLowerCase() === "s") {
      event.preventDefault();
      await saveCurrentTab();
      return;
    }

    if (event.metaKey && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      const tabIndex = Number(event.key) - 1;
      if (tabs[tabIndex]) {
        await activateTab(tabIndex);
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      if (showPalette) {
        closePalette();
      } else {
        await hideReader();
      }
    }
  }

  async function rebuildTabsFromSettings(
    settings,
    { preserveOpened = true, forceReloadActive = false } = {},
  ) {
    const dailyPath = await invoke("get_daily_note_path");
    const pinnedNotes = normalizePinnedNotes(settings.pinned_notes);
    const previousTabs = tabs;
    const previousActivePath = activeTab?.path ?? null;
    const existingByPath = new Map(previousTabs.map((tab) => [tab.path, tab]));

    const nextTabs = [
      createTab({
        kind: "daily",
        path: dailyPath,
        label: "Daily",
        existingTab:
          previousTabs.find((tab) => tab.kind === "daily") ??
          existingByPath.get(dailyPath),
      }),
      ...pinnedNotes.map((note) =>
        createTab({
          kind: "pinned",
          path: note.path,
          label: note.label,
          icon: note.icon,
          existingTab: existingByPath.get(note.path),
        }),
      ),
    ];

    if (preserveOpened) {
      const reservedPaths = new Set(nextTabs.map((tab) => tab.path));
      previousTabs
        .filter((tab) => tab.kind === "opened" && !reservedPaths.has(tab.path))
        .forEach((tab) => {
          nextTabs.push({ ...tab });
        });
    }

    tabs = nextTabs;

    const nextActiveIndex = previousActivePath
      ? nextTabs.findIndex((tab) => tab.path === previousActivePath)
      : 0;

    activeTabIndex = nextActiveIndex >= 0 ? nextActiveIndex : 0;

    const currentTab = tabs[activeTabIndex];
    if (!currentTab) return;

    if (forceReloadActive || !currentTab.loaded) {
      await loadTab(activeTabIndex, true);
      return;
    }

    loadContent(currentTab.content);
    missingFileMessage = currentTab.missing ? currentTab.missingMessage || "" : "";
  }

  async function buildInitialTabs() {
    const settings = await invoke("load_settings");
    applySettings(settings);
    await rebuildTabsFromSettings(settings, {
      preserveOpened: false,
      forceReloadActive: true,
    });
  }

  onMount(async () => {
    try {
      await buildInitialTabs();

      unlistenShowReader = await listen("show_reader", async () => {
        await flushPendingSave(false);
        await reloadCurrentTab();
      });

      unlistenSettingsChanged = await listen("settings_changed", async (event) => {
        const previousFilters = getReaderFilterSettings();
        const previousPinnedNotes = getPinnedNotesSignature(
          appSettings.pinned_notes,
        );
        applySettings(event.payload);
        const nextFilters = getReaderFilterSettings();
        const nextPinnedNotes = getPinnedNotesSignature(appSettings.pinned_notes);
        const filtersChanged =
          previousFilters.reader_hide_frontmatter !==
            nextFilters.reader_hide_frontmatter ||
          previousFilters.reader_hide_dataview !==
            nextFilters.reader_hide_dataview ||
          previousFilters.reader_hide_obsidian_comments !==
            nextFilters.reader_hide_obsidian_comments;
        const pinnedNotesChanged = previousPinnedNotes !== nextPinnedNotes;

        if (pinnedNotesChanged) {
          await flushPendingSave(false);
          await rebuildTabsFromSettings(event.payload, {
            preserveOpened: true,
            forceReloadActive: false,
          });
          return;
        }

        if (filtersChanged) {
          loadContent(rawContent);
        }
      });

      window.addEventListener("keydown", handleGlobalKeydown);
    } catch (error) {
      showStatus(normalizeError(error), "error", 2400);
    }
  });

  onDestroy(() => {
    clearTimeout(saveTimeout);
    clearTimeout(statusTimeout);
    clearTimeout(savedIndicatorTimeout);
    window.removeEventListener("keydown", handleGlobalKeydown);
    unlistenShowReader?.();
    unlistenSettingsChanged?.();
  });
</script>

<div
  class="reader-container"
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
  role="application"
>
  <div class="accent-line" role="presentation"></div>

  <div class="reader-topbar" data-tauri-drag-region>
    <div class="tab-strip">
      <!-- <div class="drag-handle" aria-hidden="true">≡</div> -->
      <button
        class="tab-action"
        type="button"
        title="Open Command Palette"
        on:mousedown|stopPropagation
        on:click|stopPropagation={openPalette}
      >
        +
      </button>

      <div class="tab-list">
        {#each tabs as tab, index (tab.path)}
          <button
            class="tab-button"
            class:active={index === activeTabIndex}
            type="button"
            title={tab.path}
            on:mousedown|stopPropagation
            on:click|stopPropagation={() => activateTab(index)}
          >
            {#if tab.icon}
              <span class="tab-icon">{tab.icon}</span>
            {/if}
            <span class="tab-label">{tab.label}</span>
          </button>
        {/each}
      </div>
    </div>

    <div class="topbar-actions">
      {#if isSaving}
        <span class="save-indicator busy">Saving...</span>
      {:else if showSavedIndicator}
        <span class="save-indicator">Saved ✓</span>
      {/if}

      <button
        class="tab-action close"
        type="button"
        title="Close Reader"
        on:mousedown|stopPropagation
        on:click|stopPropagation={hideReader}
      >
        ✕
      </button>
    </div>
  </div>

  <div
    class="editor-scroll"
    role="button"
    tabindex="0"
    on:click={(event) => {
      if (event.target === event.currentTarget) {
        focusBlock(blocks.length - 1);
      }
    }}
    on:keydown={(event) => {
      if (
        event.target === event.currentTarget &&
        (event.key === "Enter" || event.key === " ")
      ) {
        event.preventDefault();
        focusBlock(blocks.length - 1);
      }
    }}
  >
    {#if missingFileMessage}
      <div class="missing-file-banner">{missingFileMessage}</div>
    {/if}

    {#each blocks as block, index}
      <div class="block-row">
        {#if focusedBlock === index}
          <textarea
            bind:this={blockInputRefs[index]}
            bind:value={blocks[index]}
            class="block-input"
            spellcheck="false"
            rows={blocks[index].split("\n").length || 1}
            style="resize: none; overflow: hidden;"
            use:autosize
            on:blur={blurBlock}
            on:input={() => handleBlockInput(index)}
            on:keydown={(event) => handleBlockKeydown(event, index)}
          ></textarea>
        {:else if isSpacerBlock(block)}
          <div class="editor-spacer" aria-hidden="true"></div>
        {:else}
          <button
            class="block-display"
            type="button"
            on:click={() => focusBlock(index)}
          >
            {#each getBlockLines(block) as blockLine, lineIndex}
              {#if blockLine.type === "spacer"}
                <span class="block-line-spacer"></span>
              {:else if blockLine.type === "checkbox"}
                <span class="block-line checkbox-line">
                  <button
                    class="checkbox-toggle"
                    class:checked={blockLine.checked}
                    type="button"
                    on:click|stopPropagation={() =>
                      toggleCheckbox(index, lineIndex)}
                  ></button>
                  <span class:checked={blockLine.checked}
                    >{@html blockLine.html}</span
                  >
                </span>
              {:else if blockLine.type === "heading"}
                <span
                  class="block-line heading"
                  class:level-1={blockLine.level === 1}
                  class:level-2={blockLine.level === 2}
                  class:level-3={blockLine.level === 3}
                >
                  {@html blockLine.html}
                </span>
              {:else}
                <span class="block-line">{@html blockLine.html}</span>
              {/if}
            {/each}
          </button>
        {/if}
      </div>
    {/each}
  </div>

  {#if showPalette}
    <div
      class="palette-backdrop"
      role="button"
      tabindex="0"
      on:click|self={closePalette}
      on:keydown={(event) => {
        if (["Escape", "Enter", " "].includes(event.key)) {
          event.preventDefault();
          closePalette();
        }
      }}
    >
      <div class="palette" role="dialog" aria-modal="true">
        <input
          bind:this={paletteInputRef}
          bind:value={paletteQuery}
          class="palette-input"
          placeholder="Search vault notes..."
          spellcheck="false"
          on:keydown={(event) => {
            event.stopPropagation();
            handlePaletteKeydown(event);
          }}
        />

        <div class="palette-results">
          {#if filteredVaultNotes.length === 0}
            <div class="palette-empty">No matching notes</div>
          {:else}
            {#each filteredVaultNotes as note, index (note.path)}
              <button
                class="palette-item"
                class:selected={index === selectedPaletteIndex}
                type="button"
                on:mouseenter={() => (selectedPaletteIndex = index)}
                on:click={() => openVaultNote(note)}
              >
                <span class="palette-name">{note.name}</span>
                <span class="palette-path">{note.relative_path}</span>
              </button>
            {/each}
          {/if}
        </div>
      </div>
    </div>
  {/if}

  {#if statusMessage}
    <div class="status-toast" class:error={statusType === "error"}>
      {statusMessage}
    </div>
  {/if}
</div>

<style>
  :global(*) {
    box-sizing: border-box;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
  }

  .reader-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) var(--app-transparency, 55%),
      transparent
    );
    color: var(--app-text-color, #ffffff);
    font-family: var(--app-font-family, var(--font-family));
    font-size: var(--app-font-size, 15px);
    backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    -webkit-backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    border-radius: var(--app-border-radius, 12px);
    border: 0.5px solid rgba(0, 0, 0, 0.08);
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.08),
      0 2px 8px rgba(0, 0, 0, 0.04);
    overflow: clip;
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

  .accent-line {
    height: 2px;
    background: linear-gradient(
      90deg,
      rgba(139, 92, 246, 0.6),
      rgba(139, 92, 246, 0.3),
      rgba(139, 92, 246, 0.6)
    );
    background-size: 200% 100%;
    animation: shimmer 3s linear infinite;
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  .reader-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 40px;
    padding: 8px 12px 6px;
    gap: 10px;
    background: transparent;
  }

  .tab-strip {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .tab-list {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .tab-list::-webkit-scrollbar {
    display: none;
  }

  .tab-button,
  .tab-action,
  .block-display,
  .palette-item {
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
  }

  .tab-button {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    flex: 0 0 auto;
    max-width: 120px;
    padding: 8px 10px 10px;
    border-radius: 10px 10px 0 0;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-icon {
    flex: 0 0 auto;
    line-height: 1;
  }

  .tab-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-button.active {
    color: var(--app-text-color, #ffffff);
    background: rgba(255, 255, 255, 0.04);
  }

  .tab-button.active::after {
    content: "";
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: 0;
    height: 2px;
    border-radius: 999px;
    background: var(--accent-color);
  }

  .tab-action {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    cursor: pointer;
    transition:
      background var(--transition-fast),
      transform var(--transition-fast);
  }

  .tab-action:hover,
  .tab-button:hover,
  .block-display:hover,
  .palette-item:hover,
  .checkbox-toggle:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .tab-action:active {
    transform: translateY(1px);
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex: 0 0 auto;
  }

  .save-indicator {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.72);
  }

  .save-indicator.busy {
    color: rgba(255, 255, 255, 0.9);
  }

  .editor-scroll {
    position: relative;
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px 16px;
    background: transparent;
  }

  .missing-file-banner {
    margin-bottom: 12px;
    padding: 10px 12px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.72);
    font-size: 12px;
    border: 0.5px solid rgba(255, 255, 255, 0.08);
  }

  .block-row + .block-row {
    margin-top: 8px;
  }

  .block-display,
  .block-input {
    width: 100%;
    padding: 7px 10px;
    border-radius: 10px;
    text-align: left;
  }

  .block-display {
    display: flex;
    flex-direction: column;
    gap: 4px;
    cursor: text;
    color: var(--app-text-color, #ffffff);
  }

  .editor-spacer {
    height: 0.8em;
  }

  .block-line-spacer {
    display: block;
    height: 0.8em;
  }

  .block-input {
    width: 100%;
    min-height: 32px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(255, 255, 255, 0.04);
    color: var(--app-text-color, #ffffff);
    outline: none;
    resize: none;
    font: inherit;
    line-height: 1.6;
    overflow: hidden;
    field-sizing: content;
  }

  .block-line {
    display: block;
    min-height: 1.5em;
    line-height: 1.6;
    white-space: pre-wrap;
    word-break: break-word;
  }

  .block-line.heading {
    font-weight: 600;
  }

  .block-line.level-1 {
    font-size: 1.55rem;
    line-height: 1.2;
  }

  .block-line.level-2 {
    font-size: 1.25rem;
    line-height: 1.25;
  }

  .block-line.level-3 {
    font-size: 1.05rem;
    line-height: 1.3;
    letter-spacing: 0.01em;
  }

  .checkbox-line {
    display: flex;
    align-items: flex-start;
    gap: 10px;
  }

  .checkbox-toggle {
    width: 18px;
    height: 18px;
    margin-top: 7px;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.22);
    background: rgba(255, 255, 255, 0.04);
    cursor: pointer;
    flex: 0 0 auto;
  }

  .checkbox-toggle.checked {
    background: var(--accent-color);
    border-color: var(--accent-color);
    box-shadow: 0 0 0 1px rgba(0, 0, 0, 0.08);
  }

  .checkbox-toggle.checked::after {
    content: "✓";
    display: block;
    color: #101218;
    font-size: 11px;
    line-height: 16px;
    text-align: center;
    font-weight: 700;
  }

  .checkbox-line .checked {
    color: rgba(255, 255, 255, 0.52);
    text-decoration: line-through;
  }

  :global(.reader-container a) {
    color: var(--accent-color);
    text-decoration: none;
  }

  :global(.reader-container code) {
    padding: 2px 6px;
    border-radius: 6px;
    background: rgba(0, 0, 0, 0.12);
    font-family: "SF Mono", Monaco, monospace;
    font-size: 0.92em;
  }

  .palette-backdrop {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 52px 14px 14px;
    background: rgba(0, 0, 0, 0.12);
    backdrop-filter: blur(14px) saturate(120%);
    -webkit-backdrop-filter: blur(14px) saturate(120%);
  }

  .palette {
    position: relative;
    width: 100%;
    max-width: 520px;
    max-height: min(70vh, 520px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) var(--app-transparency, 55%),
      transparent
    );
    backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    -webkit-backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    color: var(--app-text-color, #ffffff);
    font-family: var(
      --app-font-family,
      -apple-system,
      BlinkMacSystemFont,
      "SF Pro Display",
      sans-serif
    );
    border: 0.5px solid rgba(0, 0, 0, 0.08);
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.08),
      0 2px 8px rgba(0, 0, 0, 0.04);
    transform: translateZ(0);
    -webkit-transform: translateZ(0);
  }

  .palette::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(
      90deg,
      rgba(139, 92, 246, 0.6),
      rgba(139, 92, 246, 0.3),
      rgba(139, 92, 246, 0.6)
    );
    background-size: 200% 100%;
    animation: shimmer 3s linear infinite;
    z-index: 1;
  }

  .palette-input {
    width: 100%;
    padding: 14px 16px 12px;
    border: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: transparent;
    color: var(--app-text-color, #ffffff);
    font: inherit;
    outline: none;
    position: relative;
    z-index: 1;
  }

  .palette-input::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }

  .palette-results {
    max-height: 420px;
    overflow-y: auto;
    padding: 10px 8px 8px;
  }

  .palette-item {
    display: flex;
    flex-direction: column;
    width: 100%;
    padding: 10px 12px;
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
  }

  .palette-item.selected {
    background: rgba(255, 255, 255, 0.08);
  }

  .palette-name {
    color: var(--app-text-color, #ffffff);
    font-weight: 600;
  }

  .palette-path {
    margin-top: 2px;
    color: rgba(255, 255, 255, 0.58);
    font-size: 12px;
  }

  .palette-empty {
    padding: 18px 12px;
    color: rgba(255, 255, 255, 0.58);
    text-align: center;
  }

  .status-toast {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    padding: 8px 16px;
    background: rgba(52, 199, 89, 0.12);
    backdrop-filter: blur(20px);
    border: 0.5px solid rgba(52, 199, 89, 0.3);
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    color: #34c759;
    animation: fadeInUp 0.2s ease-out;
    white-space: nowrap;
    z-index: 100;
  }

  .status-toast.error {
    background: rgba(255, 59, 48, 0.12);
    border-color: rgba(255, 59, 48, 0.3);
    color: #ff3b30;
  }

  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(10px);
    }
    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }

  .save-indicator {
    font-size: 11px;
    color: rgba(255, 255, 255, 0.72);
  }

  .save-indicator.busy {
    color: rgba(255, 255, 255, 0.9);
  }

  .editor-scroll::-webkit-scrollbar,
  .palette-results::-webkit-scrollbar {
    width: 6px;
  }

  .editor-scroll::-webkit-scrollbar-track,
  .palette-results::-webkit-scrollbar-track {
    background: transparent;
  }

  .editor-scroll::-webkit-scrollbar-thumb,
  .palette-results::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.12);
    border-radius: 3px;
  }
</style>
