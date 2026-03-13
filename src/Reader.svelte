<script>
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
    parseRawBlocks,
    preprocessContent,
  } from "./lib/reader/contentProcessing.js";
  import { composeContentFromMarkdown } from "./lib/reader/editorSerialization.js";

  let tabs = [];
  let activeTabIndex = 0;
  let blocks = [""];
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
  let autocompleteQuery = "";
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
  let saveTimeout;
  let pendingSave = null;
  let statusTimeout;
  let savedIndicatorTimeout;
  let unlistenShowReader;
  let unlistenSettingsChanged;

  $: activeTab = tabs[activeTabIndex] ?? null;
  $: fileMissing = missingFileMessage.trim() !== "";
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

  function isFileDrag(event) {
    const types = event.dataTransfer?.types;
    if (types && Array.from(types).includes("Files")) return true;
    const items = event.dataTransfer?.items;
    return !!items && items.length > 0;
  }

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
            label: fileLabel(entry),
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
    isPinned = kind === "daily" || kind === "pinned",
    history = [],
    existingTab = null,
  }) {
    const fallbackLabel =
      kind === "daily" ? "Daily" : kind === "pinned" ? "" : fileLabel(path);

    return {
      kind,
      path,
      label: label.trim() || fallbackLabel,
      icon: icon.trim(),
      content: existingTab?.content ?? "",
      loaded: existingTab?.loaded ?? false,
      missing: existingTab?.missing ?? false,
      missingMessage: existingTab?.missingMessage ?? "",
      isPinned: existingTab?.isPinned ?? isPinned,
      history: [...(existingTab?.history ?? history)],
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

  function filterVaultNotes(query) {
    if (!query) return vaultNotes.slice(0, 10);

    const lower = query.toLowerCase();
    return vaultNotes
      .filter(
        (note) =>
          note.name.toLowerCase().includes(lower) ||
          note.relative_path.toLowerCase().includes(lower),
      )
      .slice(0, 8);
  }

  function closeAutocomplete() {
    showAutocomplete = false;
    autocompleteQuery = "";
    autocompleteIndex = 0;
    autocompleteRange = null;
    autocompleteResults = [];
    editorComponent?.clearAutocomplete?.();
  }

  function clearHighlights() {
    if (typeof CSS !== "undefined" && CSS.highlights) {
      CSS.highlights.clear();
    }
  }

  function highlightMatches() {
    clearHighlights();

    if (
      typeof CSS === "undefined" ||
      !CSS.highlights ||
      typeof Highlight === "undefined" ||
      searchMatches.length === 0
    ) {
      return;
    }

    const allHighlight = new Highlight(...searchMatches);
    CSS.highlights.set("search-result", allHighlight);

    if (searchMatches[searchIndex]) {
      const activeHighlight = new Highlight(searchMatches[searchIndex]);
      CSS.highlights.set("search-active", activeHighlight);
    }
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
    if (!searchQuery.trim() || !editorEl) return;

    const walker = document.createTreeWalker(
      editorEl,
      NodeFilter.SHOW_TEXT,
      null,
    );

    const query = searchQuery.toLowerCase();
    const ranges = [];
    let node;

    while ((node = walker.nextNode())) {
      const text = node.textContent ?? "";
      const lower = text.toLowerCase();
      let position = 0;

      while (true) {
        const matchIndex = lower.indexOf(query, position);
        if (matchIndex === -1) break;

        const range = document.createRange();
        range.setStart(node, matchIndex);
        range.setEnd(node, matchIndex + query.length);
        ranges.push(range);
        position = matchIndex + 1;
      }
    }

    searchMatches = ranges;
    highlightMatches();

    if (ranges.length > 0) {
      scrollToMatch(0);
    }
  }

  function stepSearch(direction) {
    if (searchMatches.length === 0) return;

    searchIndex =
      (searchIndex + direction + searchMatches.length) % searchMatches.length;
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

  function openSearch() {
    showSearch = true;
    tick().then(() => {
      searchInputRef?.focus();
      searchInputRef?.select();
    });
  }

  function createImportPlaceholder(filename = "image") {
    const label = filename.replace(/\s+/g, " ").trim() || "image";
    return `[Importing image: ${label} · ${Date.now()}-${Math.random().toString(36).slice(2, 8)}]`;
  }

  function normalizeImportedImageResult(result) {
    if (typeof result === "string") {
      return { markdown: result };
    }

    return {
      markdown: result?.markdown ?? "",
    };
  }

  function fileExtension(name = "") {
    return name.split(".").pop()?.toLowerCase() ?? "";
  }

  async function importImageFile(file, fallbackPath = null) {
    const ext = fileExtension(file?.name ?? fallbackPath ?? "");
    if (!["png", "jpg", "jpeg", "webp", "gif"].includes(ext)) {
      throw new Error(
        `Unsupported image: ${file?.name ?? fallbackPath ?? "file"}`,
      );
    }

    const candidatePath =
      fallbackPath || file?.path || file?.webkitRelativePath || null;

    if (candidatePath) {
      const result = await invoke("save_image", {
        filePath: candidatePath,
      });
      return normalizeImportedImageResult(result).markdown;
    }

    if (!file) {
      throw new Error("Image file data not available");
    }

    const base64 = await new Promise((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        const result = reader.result;
        const base64String =
          typeof result === "string" ? result.split(",")[1] || result : "";
        resolve(base64String);
      };
      reader.onerror = reject;
      reader.readAsDataURL(file);
    });

    const result = await invoke("save_image_from_bytes", {
      bytesBase64: base64,
      filename: file.name || "clipboard-image.png",
    });

    return normalizeImportedImageResult(result).markdown;
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
    blocks = parseRawBlocks(normalizeNewlines(markdown));
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

        insertImportedMarkdown(
          jobs.map((job) => job.placeholder),
          { syncEditor: false },
        );

        const results = await Promise.allSettled(
          jobs.map((job) => importImageFile(job.file, job.fallbackPath)),
        );

        const replacements = results.map((result, index) => {
          if (result.status === "fulfilled") {
            return {
              placeholder: jobs[index].placeholder,
              markdown: result.value,
            };
          }

          showStatus(normalizeError(result.reason), "error", 2200);
          return {
            placeholder: jobs[index].placeholder,
            markdown: "",
          };
        });

        await finalizeImportedImages(replacements);
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

        insertImportedMarkdown(
          jobs.map((job) => job.placeholder),
          { syncEditor: false },
        );

        const results = await Promise.allSettled(
          jobs.map((job) => importImageFile(job.file, null)),
        );

        const replacements = results.map((result, index) => {
          if (result.status === "fulfilled") {
            return {
              placeholder: jobs[index].placeholder,
              markdown: result.value,
            };
          }

          showStatus(normalizeError(result.reason), "error", 2200);
          return {
            placeholder: jobs[index].placeholder,
            markdown: "",
          };
        });

        await finalizeImportedImages(replacements);
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

    const rect = event.currentTarget?.getBoundingClientRect?.();
    const x = event.clientX;
    const y = event.clientY;
    const isLeaving =
      rect &&
      (x < rect.left || x > rect.right || y < rect.top || y > rect.bottom);

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

  function applySettings(settings) {
    appSettings = {
      ...appSettings,
      vault_name: settings.vault_name ?? appSettings.vault_name,
      vault_path: settings.vault_path ?? appSettings.vault_path,
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
      accent_color: settings.accent_color ?? appSettings.accent_color,
      internal_link_color:
        settings.internal_link_color ?? appSettings.internal_link_color,
      external_link_color:
        settings.external_link_color ?? appSettings.external_link_color,
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
      reader_hide_inline_fields:
        settings.reader_hide_inline_fields ??
        appSettings.reader_hide_inline_fields,
      reader_hide_html:
        settings.reader_hide_html ?? appSettings.reader_hide_html,
    };
    applyColorSettings(appSettings);
  }

  function getReaderFilterSettings(settings = appSettings) {
    return {
      reader_hide_frontmatter: settings.reader_hide_frontmatter,
      reader_hide_dataview: settings.reader_hide_dataview,
      reader_hide_obsidian_comments: settings.reader_hide_obsidian_comments,
      reader_hide_inline_fields: settings.reader_hide_inline_fields,
      reader_hide_html: settings.reader_hide_html,
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
    const processed = preprocessContent(raw, {
      appSettings,
      codeblockMap,
      hiddenBlockMap,
      setStrippedFrontmatter: (value) => {
        strippedFrontmatter = value;
      },
    });
    blocks = parseRawBlocks(processed);

    if (!editorComponent) return;
    await editorComponent.renderContent(raw);
  }

  async function loadEditorContent(raw = "") {
    editorComponent?.finalizeBlock?.();
    closeAutocomplete();
    closeSearch();
    rawContent = normalizeNewlines(raw);
    imagePathCache.clear();
    await renderContentToEditor(rawContent);
  }

  function saveScrollPosition() {
    const currentTab = tabs[activeTabIndex];
    if (!currentTab || !editorComponent) return;
    scrollPositions.set(
      currentTab.path,
      editorComponent.getScrollTop?.() ?? 0,
    );
  }

  async function restoreScrollPosition(path) {
    await editorComponent?.restoreScroll?.(path, scrollPositions);
  }

  function handleEditorChange() {
    const markdown = getCurrentMarkdown();
    blocks = parseRawBlocks(normalizeNewlines(markdown));
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
    autocompleteQuery = event.detail.query;
    autocompleteResults = event.detail.results;
    autocompleteRange = event.detail.range;
    autocompleteIndex = event.detail.index ?? 0;
  }

  async function openExternalUrl(url) {
    try {
      await invoke("open_external_url", { url });
    } catch (error) {
      console.error("Failed to open external URL:", error);
      showStatus("Failed to open link", "error", 2200);
    }
  }

  async function loadTab(index, forceReload = false) {
    const tab = tabs[index];
    if (!tab) return;

    if (!forceReload && tab.loaded) {
      await loadEditorContent(tab.content);
      missingFileMessage = tab.missing ? tab.missingMessage || "" : "";
      if (index === activeTabIndex) {
        await restoreScrollPosition(tab.path);
      }
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
        await loadEditorContent(content);
        missingFileMessage = "";
        await restoreScrollPosition(tab.path);
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
        await loadEditorContent("");
        missingFileMessage = missingMessage;
        await restoreScrollPosition(tab.path);
      }

      if (!isFileMissingError(error)) {
        showStatus(message, "error", 2200);
      }
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
      const content = normalizeNewlines(
        await invoke("read_note_file", { path: tab.path }),
      );
      const currentContent = normalizeNewlines(tab.content ?? "");

      if (!tab.missing && content === currentContent) {
        if (index === activeTabIndex) {
          missingFileMessage = "";
        }
        return;
      }

      replaceTab(index, {
        content,
        loaded: true,
        missing: false,
        missingMessage: "",
      });

      if (index === activeTabIndex) {
        await loadEditorContent(content);
        missingFileMessage = "";
        await restoreScrollPosition(tab.path);
      }
    } catch (error) {
      const message = normalizeError(error);
      const missing = isFileMissingError(error);
      const missingMessage = missing
        ? "File not found - will be created on first save"
        : message;
      const isUnchangedMissingState =
        missing &&
        tab.missing &&
        normalizeNewlines(tab.content ?? "") === "" &&
        (tab.missingMessage || "") === missingMessage;

      if (isUnchangedMissingState) {
        if (index === activeTabIndex) {
          missingFileMessage = missingMessage;
        }
        return;
      }

      replaceTab(index, {
        content: "",
        loaded: true,
        missing,
        missingMessage,
      });

      if (index === activeTabIndex) {
        await loadEditorContent("");
        missingFileMessage = missingMessage;
        await restoreScrollPosition(tab.path);
      }

      if (!missing) {
        showStatus(message, "error", 2200);
      }
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

      showSavedIndicator = true;
      clearTimeout(savedIndicatorTimeout);
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
    clearTimeout(saveTimeout);
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

  async function forceSave(showConfirmation = true) {
    if (!activeTab) return;

    clearTimeout(saveTimeout);
    saveTimeout = null;
    pendingSave = null;

    const markdown = getCurrentMarkdown();
    blocks = parseRawBlocks(normalizeNewlines(markdown));
    const content = composeContentFromMarkdown(markdown, {
      strippedFrontmatter,
      hiddenBlockMap,
      codeblockMap,
    });
    rawContent = content;
    updateActiveTabContent(content);
    await saveTabByIndex(activeTabIndex, content, showConfirmation);
  }

  async function saveCurrentTab(showConfirmation = true) {
    await forceSave(showConfirmation);
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

  async function openInObsidian() {
    const tab = tabs[activeTabIndex];
    if (!tab?.path) return;

    try {
      const settings = await invoke("load_settings");
      const vaultName = settings.vault_name ?? "Vault";
      const vaultPath = (settings.vault_path ?? "").replace(/[\\/]$/, "");
      let relativePath = tab.path.replace(/\\/g, "/");

      if (vaultPath) {
        const normalizedVaultPath = vaultPath.replace(/\\/g, "/");
        if (relativePath.startsWith(normalizedVaultPath)) {
          relativePath = relativePath
            .slice(normalizedVaultPath.length)
            .replace(/^\/+/, "");
        }
      }

      const noteRef = relativePath.replace(/\.md$/i, "");
      const encodedVault = encodeURIComponent(vaultName);
      const encodedNote = encodeURIComponent(noteRef).replace(/%2F/g, "/");
      const obsidianUrl = `obsidian://open?vault=${encodedVault}&file=${encodedNote}`;
      await invoke("open_external_url", { url: obsidianUrl });
    } catch (error) {
      console.error("Failed to open in Obsidian:", error);
      showStatus("Failed to open in Obsidian", "error", 2200);
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

  async function resolveWikilink(target) {
    const withoutDisplay = target.split("|")[0].trim();
    const cleanTarget = withoutDisplay.split("#")[0].trim();
    if (!cleanTarget) return null;

    await ensureVaultNotes();

    const normalizedTarget = cleanTarget.toLowerCase().replace(/\\/g, "/");
    const withExtension = normalizedTarget.endsWith(".md")
      ? normalizedTarget
      : `${normalizedTarget}.md`;

    let found = vaultNotes.find((note) => {
      const relativePath = note.relative_path.toLowerCase().replace(/\\/g, "/");
      return (
        note.name.toLowerCase() === normalizedTarget ||
        relativePath === normalizedTarget ||
        relativePath === withExtension
      );
    });

    if (!found) {
      found = vaultNotes.find((note) =>
        note.name.toLowerCase().includes(normalizedTarget),
      );
    }

    return found ?? null;
  }

  async function navigateToWikilink(target, forceNewTab = false) {
    const note = await resolveWikilink(target);
    if (!note) {
      showStatus(`Note not found: [[${target}]]`, "error", 2200);
      return;
    }

    const currentTab = tabs[activeTabIndex];
    if (!currentTab) return;

    saveScrollPosition();
    const openNewTab = forceNewTab || currentTab.isPinned;

    if (openNewTab) {
      const existingIndex = tabs.findIndex((tab) => tab.path === note.path);
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
          isPinned: false,
          history: [],
        }),
      ];
      await activateTab(tabs.length - 1, true);
      return;
    }

    await forceSave(false);
    replaceTab(activeTabIndex, {
      path: note.path,
      label: note.name,
      content: "",
      loaded: false,
      missing: false,
      missingMessage: "",
      history: [...(currentTab.history ?? []), currentTab.path],
      isPinned: false,
    });
    await loadTab(activeTabIndex, true);
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
        isPinned: false,
        history: [],
      }),
    ];

    await activateTab(tabs.length - 1, true);
  }

  async function handleGlobalKeydown(event) {
    const key = event.key.toLowerCase();

    if (tabContextMenu.open && event.key === "Escape") {
      event.preventDefault();
      closeTabContextMenu();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "w") {
      event.preventDefault();
      await closeActiveTab();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "k") {
      event.preventDefault();
      await openPalette();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "f") {
      event.preventDefault();
      if (showSearch) {
        searchInputRef?.focus();
      } else {
        openSearch();
      }
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "p") {
      event.preventDefault();
      await openPalette();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "s") {
      event.preventDefault();
      await forceSave();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && /^[1-9]$/.test(event.key)) {
      event.preventDefault();
      const tabIndex = Number(event.key) - 1;
      if (tabs[tabIndex]) {
        await activateTab(tabIndex);
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      if (showAutocomplete) {
        closeAutocomplete();
        return;
      }
      if (showSearch) {
        closeSearch();
        return;
      }
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
        isPinned: true,
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
          isPinned: true,
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

    await loadEditorContent(currentTab.content);
    missingFileMessage = currentTab.missing
      ? currentTab.missingMessage || ""
      : "";
  }

  async function buildInitialTabs() {
    const settings = await invoke("load_settings");
    applySettings(settings);
    await rebuildTabsFromSettings(settings, {
      preserveOpened: false,
      forceReloadActive: true,
    });
  }

  // TODO: move to ReaderEditor once window-wide drag/drop can be preserved cleanly.

  onMount(async () => {
    try {
      await buildInitialTabs();
      await ensureVaultNotes();

      unlistenShowReader = await listen("show_reader", async () => {
        saveScrollPosition();
        editorComponent?.finalizeBlock?.();
        isDragging = false;
        dragCounter = 0;
        await flushPendingSave(false);
        if (tabs[activeTabIndex]) {
          await syncTabWithDisk(activeTabIndex);
        }
      });

      unlistenSettingsChanged = await listen(
        "settings_changed",
        async (event) => {
          const previousFilters = getReaderFilterSettings();
          const previousPinnedNotes = getPinnedNotesSignature(
            appSettings.pinned_notes,
          );
          applySettings(event.payload);
          const nextFilters = getReaderFilterSettings();
          const nextPinnedNotes = getPinnedNotesSignature(
            appSettings.pinned_notes,
          );
          const filtersChanged =
            previousFilters.reader_hide_frontmatter !==
              nextFilters.reader_hide_frontmatter ||
            previousFilters.reader_hide_dataview !==
              nextFilters.reader_hide_dataview ||
            previousFilters.reader_hide_obsidian_comments !==
              nextFilters.reader_hide_obsidian_comments ||
            previousFilters.reader_hide_inline_fields !==
              nextFilters.reader_hide_inline_fields ||
            previousFilters.reader_hide_html !== nextFilters.reader_hide_html;
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
            editorComponent?.finalizeBlock?.();
            imagePathCache.clear();
            await renderContentToEditor(rawContent);
          }
        },
      );

      window.addEventListener("keydown", handleGlobalKeydown);
    } catch (error) {
      showStatus(normalizeError(error), "error", 2400);
    }
  });

  onDestroy(() => {
    clearTimeout(saveTimeout);
    clearTimeout(statusTimeout);
    clearTimeout(savedIndicatorTimeout);
    clearHighlights();
    window.removeEventListener("keydown", handleGlobalKeydown);
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
    on:openInObsidian={openInObsidian}
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
      navigateToWikilink(event.detail.target, event.detail.newTab)}
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
    transform: translateZ(0);
    -webkit-transform: translateZ(0);
  }

  .reader-container.dragging {
    background: rgba(255, 255, 255, 0.7);
    border-color: rgba(255, 255, 255, 0.7);
    border-width: 2px;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.08),
      0 2px 8px rgba(0, 0, 0, 0.04);
  }

  :global(.accent-line),
  :global(.reader-topbar),
  :global(.search-bar),
  :global(.editor-scroll),
  :global(.status-toast) {
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .drop-overlay {
    position: absolute;
    inset: 2px;
    background: none;
    border: 2px dashed rgba(255, 255, 255, 0.7);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    pointer-events: none;
    z-index: 30;
  }

  .reader-container.palette-open :global(.accent-line),
  .reader-container.palette-open :global(.reader-topbar),
  .reader-container.palette-open :global(.search-bar),
  .reader-container.palette-open :global(.editor-scroll),
  .reader-container.palette-open :global(.status-toast) {
    filter: blur(4px) brightness(0.85);
    opacity: 0.7;
    transform: scale(0.996);
    pointer-events: none;
    transition:
      filter 0.2s,
      opacity 0.2s,
      transform 0.2s;
  }

  :global(.callout-blue) {
    background: rgba(59, 130, 246, 0.1);
    border-color: rgba(59, 130, 246, 0.6);
    color: #3b82f6;
  }

  :global(.callout-green) {
    background: rgba(34, 197, 94, 0.1);
    border-color: rgba(34, 197, 94, 0.6);
    color: #22c55e;
  }

  :global(.callout-yellow) {
    background: rgba(234, 179, 8, 0.1);
    border-color: rgba(234, 179, 8, 0.6);
    color: #eab308;
  }

  :global(.callout-red) {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.6);
    color: #ef4444;
  }

  :global(.callout-purple) {
    background: rgba(139, 92, 246, 0.1);
    border-color: rgba(139, 92, 246, 0.6);
    color: #8b5cf6;
  }

  :global(.callout-orange) {
    background: rgba(249, 115, 22, 0.1);
    border-color: rgba(249, 115, 22, 0.6);
    color: #f97316;
  }

  :global(.callout-gray) {
    background: rgba(107, 114, 128, 0.1);
    border-color: rgba(107, 114, 128, 0.6);
    color: #6b7280;
  }

  :global(.callout-blue .callout-content),
  :global(.callout-green .callout-content),
  :global(.callout-yellow .callout-content),
  :global(.callout-red .callout-content),
  :global(.callout-purple .callout-content),
  :global(.callout-orange .callout-content),
  :global(.callout-gray .callout-content) {
    color: var(--app-text-color, #ffffff);
  }

  .autocomplete-dropdown {
    position: fixed;
    z-index: 200;
    min-width: 220px;
    max-width: 320px;
    overflow: hidden;
    border-radius: 8px;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) var(--app-transparency, 55%),
      transparent
    );
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow:
      0 18px 40px rgba(0, 0, 0, 0.22),
      0 6px 16px rgba(0, 0, 0, 0.14);
    backdrop-filter: blur(20px) saturate(130%);
    -webkit-backdrop-filter: blur(20px) saturate(130%);
  }

  .autocomplete-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 7px 12px;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    text-align: left;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .autocomplete-item:hover,
  .autocomplete-item.selected {
    background: color-mix(
      in srgb,
      var(--accent-color, #8b5cf6) 14%,
      transparent
    );
  }

  .autocomplete-name {
    color: var(--app-text-color, #ffffff);
    font-size: 13px;
    font-weight: 500;
  }

  .autocomplete-path {
    overflow: hidden;
    color: var(--text-secondary);
    font-size: 10px;
    white-space: nowrap;
    text-overflow: ellipsis;
  }

  :global(::highlight(search-result)) {
    background-color: rgba(234, 179, 8, 0.35);
    color: inherit;
  }

  :global(::highlight(search-active)) {
    background-color: rgba(234, 179, 8, 0.8);
    color: #1a1a1a;
  }
</style>
