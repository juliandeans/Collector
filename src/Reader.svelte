<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";

  let tabs = [];
  let activeTabIndex = 0;
  let lines = [""];
  let focusedLine = null;
  let showPalette = false;
  let paletteQuery = "";
  let vaultNotes = [];
  let isSaving = false;
  let lastSaved = null;
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
  };
  let statusMessage = "";
  let statusType = "";
  let missingFileMessage = "";
  let selectedPaletteIndex = 0;
  let showSavedIndicator = false;

  let paletteInputRef;
  let lineInputRefs = [];
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

  function splitContent(content = "") {
    const normalized = content.replace(/\r\n/g, "\n");
    return normalized.length > 0 ? normalized.split("\n") : [""];
  }

  function joinLines(nextLines) {
    return nextLines.join("\n");
  }

  function fileLabel(path) {
    const filename = path.split("/").pop() || path;
    return filename.replace(/\.md$/i, "");
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
      background_color: settings.background_color ?? appSettings.background_color,
      font_family: settings.font_family ?? appSettings.font_family,
      font_size: settings.font_size ?? appSettings.font_size,
      border_radius: settings.border_radius ?? appSettings.border_radius,
      window_transparency:
        settings.window_transparency ?? appSettings.window_transparency,
      window_blur: settings.window_blur ?? appSettings.window_blur,
      window_saturation: settings.window_saturation ?? appSettings.window_saturation,
      window_brightness: settings.window_brightness ?? appSettings.window_brightness,
      text_color: settings.text_color ?? appSettings.text_color,
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

  function updateLines(nextLines, shouldScheduleSave = true) {
    lines = nextLines.length > 0 ? nextLines : [""];
    updateActiveTabContent(joinLines(lines));
    if (shouldScheduleSave) {
      scheduleSave();
    }
  }

  async function loadTab(index, forceReload = false) {
    const tab = tabs[index];
    if (!tab) return;

    focusedLine = null;
    lineInputRefs = [];

    if (!forceReload && tab.loaded) {
      lines = splitContent(tab.content);
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
        lines = splitContent(content);
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
        lines = [""];
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

    const content = joinLines(lines);
    updateActiveTabContent(content);
    await saveTabByIndex(activeTabIndex, content, showConfirmation);
  }

  function scheduleSave() {
    clearTimeout(saveTimeout);
    pendingSave = {
      index: activeTabIndex,
      content: joinLines(lines),
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

  function focusLine(index) {
    focusedLine = index;
    setTimeout(() => {
      const input = lineInputRefs[index];
      if (input) {
        input.focus();
        input.setSelectionRange(input.value.length, input.value.length);
      }
    }, 0);
  }

  function blurLine() {
    focusedLine = null;
  }

  function handleLineInput(index, value) {
    const nextLines = [...lines];
    nextLines[index] = value;
    updateLines(nextLines);
  }

  function insertLineAfter(index) {
    const nextLines = [...lines];
    nextLines.splice(index + 1, 0, "");
    updateLines(nextLines);
    focusLine(index + 1);
  }

  function removeLine(index) {
    if (lines.length === 1) {
      updateLines([""]);
      focusLine(0);
      return;
    }

    const nextLines = [...lines];
    nextLines.splice(index, 1);
    updateLines(nextLines);
    focusLine(Math.max(index - 1, 0));
  }

  function handleLineKeydown(event, index) {
    if (event.key === "Enter") {
      event.preventDefault();
      insertLineAfter(index);
      return;
    }

    if (event.key === "ArrowUp" && index > 0) {
      event.preventDefault();
      focusLine(index - 1);
      return;
    }

    if (event.key === "ArrowDown" && index < lines.length - 1) {
      event.preventDefault();
      focusLine(index + 1);
      return;
    }

    if (event.key === "Backspace" && lines[index] === "") {
      event.preventDefault();
      removeLine(index);
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

  async function toggleCheckbox(index) {
    const checkbox = parseCheckbox(lines[index]);
    if (!checkbox) return;

    const marker = checkbox.checked ? " " : "x";
    const nextLines = [...lines];
    nextLines[index] = `- [${marker}] ${checkbox.text}`;
    updateLines(nextLines, false);
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
      {
        label: fileLabel(note.path),
        path: note.path,
        content: "",
        loaded: false,
        missing: false,
        missingMessage: "",
      },
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

  async function buildInitialTabs() {
    const settings = await invoke("load_settings");
    applySettings(settings);

    const dailyPath = await invoke("get_daily_note_path");
    const pinnedTabs = (settings.pinned_notes || []).map((path) => ({
      label: fileLabel(path),
      path,
      content: "",
      loaded: false,
      missing: false,
      missingMessage: "",
    }));

    tabs = [
      {
        label: "Daily",
        path: dailyPath,
        content: "",
        loaded: false,
        missing: false,
        missingMessage: "",
      },
      ...pinnedTabs,
    ];

    await activateTab(0, true);
  }

  onMount(async () => {
    try {
      await buildInitialTabs();

      unlistenShowReader = await listen("show_reader", async () => {
        await flushPendingSave(false);
        await reloadCurrentTab();
      });

      unlistenSettingsChanged = await listen("settings_changed", (event) => {
        applySettings(event.payload);
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
      <div class="drag-handle" aria-hidden="true">≡</div>
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
            <span>{tab.label}</span>
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
        focusLine(lines.length - 1);
      }
    }}
    on:keydown={(event) => {
      if (
        event.target === event.currentTarget &&
        (event.key === "Enter" || event.key === " ")
      ) {
        event.preventDefault();
        focusLine(lines.length - 1);
      }
    }}
  >
    {#if missingFileMessage}
      <div class="missing-file-banner">{missingFileMessage}</div>
    {/if}

    {#each lines as line, index}
      <div class="line-row">
        {#if focusedLine === index}
          <input
            bind:this={lineInputRefs[index]}
            class="line-input"
            type="text"
            value={line}
            spellcheck="false"
            on:blur={blurLine}
            on:input={(event) => handleLineInput(index, event.currentTarget.value)}
            on:keydown={(event) => handleLineKeydown(event, index)}
          />
        {:else}
          {@const heading = parseHeading(line)}
          {@const checkbox = parseCheckbox(line)}

          {#if line.trim() === ""}
            <button
              class="line-display blank"
              type="button"
              on:click={() => focusLine(index)}
            >
              <span class="line-spacer"></span>
            </button>
          {:else if checkbox}
            <div class="line-display checkbox-line">
              <button
                class="checkbox-toggle"
                class:checked={checkbox.checked}
                type="button"
                on:click={() => toggleCheckbox(index)}
              ></button>
              <button
                class="checkbox-text"
                type="button"
                on:click={() => focusLine(index)}
              >
                <span
                  class:checked={checkbox.checked}
                  >{@html renderInlineMarkdown(checkbox.text || "Task")}</span
                >
              </button>
            </div>
          {:else if heading}
            <button
              class="line-display heading"
              class:level-1={heading.level === 1}
              class:level-2={heading.level === 2}
              class:level-3={heading.level === 3}
              type="button"
              on:click={() => focusLine(index)}
            >
              <span>{@html renderInlineMarkdown(heading.text)}</span>
            </button>
          {:else}
            <button
              class="line-display"
              type="button"
              on:click={() => focusLine(index)}
            >
              <span>{@html renderInlineMarkdown(line)}</span>
            </button>
          {/if}
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
      <div
        class="palette"
        role="dialog"
        aria-modal="true"
      >
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
    inset: 0;
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
      inset 0 0 0 0.5px rgba(255, 255, 255, 0.1);
    overflow: hidden;
  }

  .accent-line {
    height: 2px;
    width: 100%;
    background: linear-gradient(
      90deg,
      transparent,
      var(--accent-color),
      transparent
    );
    opacity: 0.85;
  }

  .reader-topbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    min-height: 40px;
    padding: 6px 10px;
    gap: 10px;
    background: linear-gradient(
      180deg,
      rgba(255, 255, 255, 0.08),
      rgba(255, 255, 255, 0.02)
    );
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
  }

  .tab-strip {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .drag-handle {
    flex: 0 0 auto;
    color: rgba(255, 255, 255, 0.45);
    letter-spacing: 0.08em;
    user-select: none;
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
  .line-display,
  .checkbox-text,
  .palette-item {
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
  }

  .tab-button {
    position: relative;
    flex: 0 0 auto;
    max-width: 120px;
    padding: 8px 10px 10px;
    border-radius: 10px 10px 0 0;
    color: rgba(255, 255, 255, 0.72);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-button.active {
    color: var(--app-text-color, #ffffff);
    background: rgba(255, 255, 255, 0.08);
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
    background: rgba(255, 255, 255, 0.08);
    cursor: pointer;
    transition:
      background var(--transition-fast),
      transform var(--transition-fast);
  }

  .tab-action:hover,
  .tab-button:hover,
  .line-display:hover,
  .checkbox-text:hover,
  .palette-item:hover,
  .checkbox-toggle:hover {
    background: rgba(255, 255, 255, 0.12);
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
    padding: 16px 14px 18px;
  }

  .missing-file-banner {
    margin-bottom: 12px;
    padding: 10px 12px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.06);
    color: rgba(255, 255, 255, 0.72);
    font-size: 12px;
  }

  .line-row + .line-row {
    margin-top: 2px;
  }

  .line-display,
  .line-input {
    width: 100%;
    min-height: 32px;
    padding: 7px 10px;
    border-radius: 10px;
    text-align: left;
  }

  .line-display {
    cursor: text;
    color: var(--app-text-color, #ffffff);
  }

  .line-display.blank {
    padding: 6px 10px;
  }

  .line-spacer {
    display: block;
    height: 18px;
    border-radius: 999px;
    background: transparent;
  }

  .line-input {
    border: 1px solid rgba(255, 255, 255, 0.16);
    background: rgba(10, 10, 18, 0.42);
    color: var(--app-text-color, #ffffff);
    outline: none;
  }

  .line-display.heading {
    font-weight: 600;
  }

  .line-display.level-1 {
    font-size: 1.55rem;
    line-height: 1.2;
  }

  .line-display.level-2 {
    font-size: 1.25rem;
    line-height: 1.25;
  }

  .line-display.level-3 {
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

  .checkbox-text {
    flex: 1;
    min-height: 32px;
    padding: 7px 10px;
    border-radius: 10px;
    text-align: left;
    cursor: text;
  }

  .checkbox-text .checked {
    color: rgba(255, 255, 255, 0.52);
    text-decoration: line-through;
  }

  :global(.reader-container a) {
    color: #9bd0ff;
    text-decoration: none;
  }

  :global(.reader-container code) {
    padding: 2px 6px;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
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
    background: rgba(10, 12, 18, 0.22);
    backdrop-filter: blur(10px);
    -webkit-backdrop-filter: blur(10px);
  }

  .palette {
    width: 100%;
    max-width: 520px;
    max-height: min(70vh, 520px);
    overflow: hidden;
    border-radius: 18px;
    background: rgba(20, 20, 30, 0.82);
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: 0 24px 60px rgba(0, 0, 0, 0.28);
  }

  .palette-input {
    width: 100%;
    padding: 14px 16px;
    border: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    background: transparent;
    color: #fff;
    font: inherit;
    outline: none;
  }

  .palette-results {
    max-height: 420px;
    overflow-y: auto;
    padding: 8px;
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
    background: rgba(255, 255, 255, 0.12);
  }

  .palette-name {
    color: #fff;
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
    right: 12px;
    bottom: 12px;
    padding: 9px 12px;
    border-radius: 999px;
    background: rgba(24, 26, 34, 0.82);
    color: #fff;
    font-size: 12px;
    backdrop-filter: blur(20px);
    -webkit-backdrop-filter: blur(20px);
    box-shadow: 0 12px 24px rgba(0, 0, 0, 0.18);
  }

  .status-toast.error {
    background: rgba(143, 35, 35, 0.88);
  }
</style>
