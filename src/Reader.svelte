<script>
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { getReaderIconComponent } from "./lib/reader-icons.js";

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
  let tabContextMenu = {
    open: false,
    x: 0,
    y: 0,
    tabIndex: -1,
  };

  let editorRef;
  let scrollRef;
  let paletteInputRef;
  let isComposing = false;
  let isRenderingContent = false;
  let activeParagraphEl = null;
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
    };
  }

  function getTabIcon(tab) {
    return getReaderIconComponent(tab?.icon);
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

  function preprocessContent(raw = "") {
    let text = normalizeNewlines(raw);

    strippedFrontmatter = "";
    codeblockMap = new Map();
    hiddenBlockMap = new Map();

    if (appSettings.reader_hide_frontmatter) {
      text = text.replace(
        /^---\n[\s\S]*?\n---[ \t]*(?:\n+)?/,
        (match) => {
          strippedFrontmatter = match.trimEnd();
          return "";
        },
      );
    }

    if (appSettings.reader_hide_obsidian_comments) {
      text = text.replace(/%%[\s\S]*?%%[ \t]*/g, (match) => {
        const hiddenId = `__HD_${hiddenBlockMap.size}__`;
        hiddenBlockMap.set(hiddenId, match);
        return `\u200B${hiddenId}`;
      });
    }

    if (appSettings.reader_hide_dataview) {
      text = replaceCodeblocks(text);
      console.log(
        "[CB] After replaceCodeblocks, text excerpt:",
        text.slice(0, 500),
      );
      console.log(
        "[CB] codeblockMap contents:",
        [...codeblockMap.entries()].map(
          ([k, v]) => `${k} => ${v.slice(0, 30)}`,
        ),
      );
    }

    return text.replace(/^\n+/, "");
  }

  function replaceCodeblocks(text = "") {
    const result = [];
    const regex =
      /^([ \t]*>[ \t]*)?(```([A-Za-z0-9_-]*)[ \t]*)\n([\s\S]*?)^([ \t]*>[ \t]*)?```[ \t]*$/gm;
    let lastIndex = 0;
    let match;

    while ((match = regex.exec(text)) !== null) {
      result.push(text.slice(lastIndex, match.index));

      const codeblockId = `__CB_${codeblockMap.size}__`;
      const label = (match[3] ?? "").trim() || "code";
      codeblockMap.set(codeblockId, match[0]);
      result.push(`\u200B${codeblockId}:${label}\u200B`);

      lastIndex = match.index + match[0].length;
    }

    result.push(text.slice(lastIndex));
    return result.join("");
  }

  function escHtml(text = "") {
    return text
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;");
  }

  function inlineMarkdown(text = "") {
    let html = escHtml(text);
    html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
    html = html.replace(/\*(.+?)\*/g, "<em>$1</em>");
    html = html.replace(/`(.+?)`/g, "<code>$1</code>");
    html = html.replace(/\[\[(.+?)\]\]/g, '<span class="wikilink">[[$1]]</span>');
    html = html.replace(/\[(.+?)\]\((.+?)\)/g, '<a href="$2">$1</a>');
    return html;
  }

  function markdownLineToHtml(line) {
    if (line === null || line === undefined) return "";
    if (line.includes("\u200B")) {
      console.log(
        "[CB] Sentinel line entering renderer:",
        JSON.stringify(line),
      );
    }

    const trimmed = line.trim();
    if (line.includes("\u200B")) {
      console.log(
        "[CB] trimmed starts with sentinel?",
        trimmed.startsWith("\u200B"),
        "first chars:",
        JSON.stringify(trimmed.slice(0, 20)),
      );
    }
    const codeblockMatch = trimmed.match(/\u200B(__CB_\d+__):([\w-]*)\u200B/);
    if (codeblockMatch) {
      const [, id, langValue] = codeblockMatch;
      const lang = langValue || "code";
      return `<div class="codeblock-pill" data-cbid="${id}" data-cblang="${escHtml(lang)}" contenteditable="false"><span class="codeblock-icon"></span><span class="codeblock-lang">${escHtml(lang)}</span></div>`;
    }

    const hiddenMatch = trimmed.match(/^\u200B(__HD_\d+__)$/);
    if (hiddenMatch) {
      return `<div class="hidden-marker" data-hidden-id="${hiddenMatch[1]}" contenteditable="false"></div>`;
    }

    if (/^###### /.test(line)) return `<h6>${escHtml(line.slice(7))}</h6>`;
    if (/^##### /.test(line)) return `<h5>${escHtml(line.slice(6))}</h5>`;
    if (/^#### /.test(line)) return `<h4>${escHtml(line.slice(5))}</h4>`;
    if (/^### /.test(line)) return `<h3>${escHtml(line.slice(4))}</h3>`;
    if (/^## /.test(line)) return `<h2>${escHtml(line.slice(3))}</h2>`;
    if (/^# /.test(line)) return `<h1>${escHtml(line.slice(2))}</h1>`;
    if (line.trim() === "") return "<p><br></p>";
    if (/^---+$/.test(line.trim())) return "<hr>";

    if (/^- \[ \] /.test(line)) {
      const label = line.slice(6);
      return `<p><input type="checkbox" class="md-checkbox" contenteditable="false"> ${inlineMarkdown(label)}</p>`;
    }

    if (/^- \[x\] /i.test(line)) {
      const label = line.slice(6);
      return `<p><input type="checkbox" class="md-checkbox" contenteditable="false" checked> ${inlineMarkdown(label)}</p>`;
    }

    if (/^> /.test(line)) {
      return `<blockquote>${inlineMarkdown(line.slice(2))}</blockquote>`;
    }

    if (/^- /.test(line)) {
      return `<p class="list-item">${inlineMarkdown(line.slice(2))}</p>`;
    }

    return `<p>${inlineMarkdown(line)}</p>`;
  }

  function markdownToHtml(text = "") {
    if (!text.trim()) return "";

    const lines = normalizeNewlines(text).split("\n");
    return lines.map((line) => markdownLineToHtml(line)).join("");
  }

  function elementInnerToMarkdown(el) {
    return serializeChildren(el);
  }

  function elementToMarkdownLine(el) {
    if (!el) return "";

    if (el.classList?.contains("raw-mode")) {
      return el.textContent ?? "";
    }

    if (el.classList?.contains("codeblock-pill")) {
      return null;
    }

    if (el.classList?.contains("hidden-marker")) {
      return el.dataset.hiddenId ?? "";
    }

    const tag = el.tagName?.toLowerCase();
    const inner = el.innerText ?? el.textContent ?? "";

    if (tag === "h1") return `# ${inner}`;
    if (tag === "h2") return `## ${inner}`;
    if (tag === "h3") return `### ${inner}`;
    if (tag === "h4") return `#### ${inner}`;
    if (tag === "h5") return `##### ${inner}`;
    if (tag === "h6") return `###### ${inner}`;
    if (tag === "hr") return "---";
    if (tag === "blockquote") return `> ${inner}`;

    if (tag === "p") {
      const checkbox = el.querySelector('input[type="checkbox"]');
      if (checkbox) {
        const text = inner.replace(/^\s*/, "");
        return `${checkbox.checked ? "- [x] " : "- [ ] "}${text}`;
      }

      if (el.classList.contains("list-item")) {
        return `- ${inner}`;
      }

      if (!inner.trim()) return "";
      return elementInnerToMarkdown(el);
    }

    return inner;
  }

  function placeCursorAtEnd(el) {
    if (!el) return;

    const range = document.createRange();
    const selection = window.getSelection();
    if (!selection) return;

    range.selectNodeContents(el);
    range.collapse(false);
    selection.removeAllRanges();
    selection.addRange(range);
  }

  function rerenderBlock(el) {
    if (!el || !editorRef?.contains(el)) return null;

    const raw = elementToMarkdownLine(el);
    if (raw === null) return el;

    const tempDiv = document.createElement("div");
    tempDiv.innerHTML = markdownLineToHtml(raw);
    const newEl = tempDiv.firstElementChild;
    if (!newEl) return el;

    el.replaceWith(newEl);
    return newEl;
  }

  function finalizeActiveBlock() {
    if (activeParagraphEl && editorRef?.contains(activeParagraphEl)) {
      rerenderBlock(activeParagraphEl);
    }
    activeParagraphEl = null;
  }

  function switchActiveBlock(newEl) {
    if (activeParagraphEl && editorRef?.contains(activeParagraphEl)) {
      rerenderBlock(activeParagraphEl);
      activeParagraphEl = null;
    }

    if (!newEl || !editorRef?.contains(newEl)) {
      activeParagraphEl = null;
      return;
    }

    const raw = elementToMarkdownLine(newEl);
    if (raw === null) {
      activeParagraphEl = null;
      return;
    }

    activeParagraphEl = newEl;
    activeParagraphEl.classList.add("raw-mode");
    activeParagraphEl.textContent = raw;
    placeCursorAtEnd(activeParagraphEl);
  }

  function handleSelectionChange() {
    if (isRenderingContent || !editorRef) return;

    const selection = window.getSelection();
    if (!selection || selection.rangeCount === 0) return;

    const node = selection.getRangeAt(0).startContainer;
    if (!editorRef.contains(node)) return;

    let el = node.nodeType === Node.TEXT_NODE ? node.parentElement : node;
    while (el && el !== editorRef && el.parentElement !== editorRef) {
      el = el.parentElement;
    }

    if (!el || el === editorRef || el === activeParagraphEl) return;
    if (el.classList?.contains("codeblock-pill")) return;

    switchActiveBlock(el);
  }

  function handleEditorBlur(event) {
    if (editorRef?.contains(event.relatedTarget)) return;
    finalizeActiveBlock();
  }

  function serializeInline(node) {
    if (node.nodeType === Node.TEXT_NODE) {
      return (node.textContent ?? "").replace(/\u00A0/g, " ");
    }

    if (node.nodeType !== Node.ELEMENT_NODE) {
      return "";
    }

    const element = node;
    const tag = element.tagName.toLowerCase();

    if (tag === "br") return "";
    if (tag === "input") return "";

    if (tag === "strong" || tag === "b") {
      return `**${serializeChildren(element)}**`;
    }

    if (tag === "em" || tag === "i") {
      return `*${serializeChildren(element)}*`;
    }

    if (tag === "code") {
      return `\`${serializeChildren(element)}\``;
    }

    if (tag === "a") {
      const href = element.getAttribute("href") ?? "";
      return `[${serializeChildren(element)}](${href})`;
    }

    if (tag === "span" && element.classList.contains("wikilink")) {
      return element.textContent ?? "";
    }

    return serializeChildren(element);
  }

  function serializeChildren(node, { skipCheckbox = false } = {}) {
    let result = "";

    node.childNodes.forEach((child) => {
      if (
        skipCheckbox &&
        child.nodeType === Node.ELEMENT_NODE &&
        child.tagName.toLowerCase() === "input"
      ) {
        return;
      }

      result += serializeInline(child);
    });

    return result.replace(/\u200B/g, "");
  }

  function htmlToMarkdown(el) {
    const lines = [];

    el.childNodes.forEach((child) => {
      if (child.nodeType === Node.TEXT_NODE) {
        const text = (child.textContent ?? "").trim();
        if (text) {
          lines.push(text);
        }
        return;
      }

      if (child.nodeType !== Node.ELEMENT_NODE) return;

      const element = child;
      const tag = element.tagName.toLowerCase();

      if (tag === "h1") {
        lines.push(`# ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "h2") {
        lines.push(`## ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "h3") {
        lines.push(`### ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "h4") {
        lines.push(`#### ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "h5") {
        lines.push(`##### ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "h6") {
        lines.push(`###### ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "hr") {
        lines.push("---");
        return;
      }

      if (tag === "blockquote") {
        lines.push(`> ${serializeChildren(element).trim()}`);
        return;
      }

      if (tag === "div" && element.classList.contains("codeblock-pill")) {
        const id = element.dataset.cbid;
        const lang = element.dataset.cblang ?? "code";
        if (id) {
          lines.push(`\u200B${id}:${lang}\u200B`);
        }
        return;
      }

      if (tag === "div" && element.classList.contains("hidden-marker")) {
        const hiddenId = element.dataset.hiddenId;
        if (hiddenId) {
          lines.push(hiddenId);
        }
        return;
      }

      if (element.classList.contains("raw-mode")) {
        lines.push(element.textContent ?? "");
        return;
      }

      if (tag === "p" || tag === "div") {
        const checkbox = element.querySelector('input[type="checkbox"]');
        if (checkbox) {
          const checked = checkbox.checked;
          const text = serializeChildren(element, { skipCheckbox: true }).trim();
          lines.push(`${checked ? "- [x] " : "- [ ] "}${text}`);
          return;
        }

        if (element.classList.contains("list-item")) {
          lines.push(`- ${serializeChildren(element).trim()}`);
          return;
        }

        const text = serializeChildren(element).trim();
        const hasOnlyBreak =
          element.childNodes.length === 1 &&
          element.firstChild?.nodeType === Node.ELEMENT_NODE &&
          element.firstChild?.tagName.toLowerCase() === "br";

        if (!text && hasOnlyBreak) {
          lines.push("");
          return;
        }

        if (!text && !element.textContent?.trim()) {
          lines.push("");
          return;
        }

        lines.push(text);
        return;
      }

      const fallbackText = serializeChildren(element).trim();
      if (fallbackText) {
        lines.push(fallbackText);
      }
    });

    return lines.join("\n");
  }

  function restoreCodeblocks(markdown = "") {
    let restored = markdown;
    restored = restored.replace(
      /\u200B(__CB_\d+__):[^\u200B]*\u200B/g,
      (_, id) => codeblockMap.get(id) ?? "",
    );

    return restored;
  }

  function restoreHiddenBlocks(markdown = "") {
    let restored = markdown;

    hiddenBlockMap.forEach((block, id) => {
      const escaped = id.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
      restored = restored.replace(new RegExp(`(?:\\u200B)?${escaped}`, "g"), block);
    });

    return restored;
  }

  function getCurrentMarkdown() {
    if (!editorRef) {
      return preprocessContent(rawContent);
    }

    return normalizeNewlines(htmlToMarkdown(editorRef));
  }

  function composeContentFromMarkdown(markdown = "") {
    const normalized = normalizeNewlines(markdown);
    blocks = parseRawBlocks(normalized);

    const withHiddenBlocks = restoreHiddenBlocks(normalized);
    const restored = restoreCodeblocks(withHiddenBlocks);
    const frontmatter = strippedFrontmatter.trim();

    if (frontmatter && restored.trim()) {
      return `${frontmatter}\n\n${restored}`;
    }

    if (frontmatter) {
      return frontmatter;
    }

    return restored;
  }

  function renderContentToEditor(raw = "") {
    const processed = preprocessContent(raw);
    blocks = parseRawBlocks(processed);
    activeParagraphEl = null;

    if (!editorRef) return;

    isRenderingContent = true;
    editorRef.innerHTML = markdownToHtml(processed);
    console.log(
      "[CB] Pills in DOM:",
      editorRef.querySelectorAll(".codeblock-pill").length,
    );
    console.log("[CB] innerHTML excerpt:", editorRef.innerHTML.slice(0, 300));
    isRenderingContent = false;
  }

  function loadEditorContent(raw = "") {
    finalizeActiveBlock();
    rawContent = normalizeNewlines(raw);
    renderContentToEditor(rawContent);
  }

  async function loadTab(index, forceReload = false) {
    const tab = tabs[index];
    if (!tab) return;

    if (!forceReload && tab.loaded) {
      loadEditorContent(tab.content);
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
        loadEditorContent(content);
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
        loadEditorContent("");
        missingFileMessage = missingMessage;
      }

      if (!isFileMissingError(error)) {
        showStatus(message, "error", 2200);
      }
    }
  }

  async function activateTab(index, forceReload = false) {
    if (index !== activeTabIndex) {
      finalizeActiveBlock();
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

      if (showConfirmation) {
        showStatus("Saved ✓", "success");
      }
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
    const content = composeContentFromMarkdown(markdown);
    rawContent = content;
    updateActiveTabContent(content);
    await saveTabByIndex(activeTabIndex, content, showConfirmation);
  }

  async function saveCurrentTab(showConfirmation = true) {
    await forceSave(showConfirmation);
  }

  function handleInput() {
    if (isComposing || isRenderingContent || !editorRef) return;

    const markdown = getCurrentMarkdown();
    const content = composeContentFromMarkdown(markdown);
    rawContent = content;
    updateActiveTabContent(content);
    scheduleSave(content);
  }

  function handleCompositionStart() {
    isComposing = true;
  }

  function handleCompositionEnd() {
    isComposing = false;
    handleInput();
  }

  async function handleKeydown(event) {
    const key = event.key.toLowerCase();

    if (event.key === "Escape") {
      event.preventDefault();
      event.stopPropagation();
      await hideReader();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "s") {
      event.preventDefault();
      event.stopPropagation();
      await forceSave();
      return;
    }

    if ((event.metaKey || event.ctrlKey) && key === "k") {
      event.preventDefault();
      event.stopPropagation();
      await openPalette();
    }
  }

  function closeTabContextMenu() {
    tabContextMenu = {
      open: false,
      x: 0,
      y: 0,
      tabIndex: -1,
    };
  }

  function openTabContextMenu(event, index) {
    const tab = tabs[index];
    if (!tab || tab.kind !== "opened") {
      closeTabContextMenu();
      return;
    }

    event.preventDefault();
    event.stopPropagation();

    tabContextMenu = {
      open: true,
      x: event.clientX,
      y: event.clientY,
      tabIndex: index,
    };
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

  async function closeTabByIndex(index) {
    const tab = tabs[index];
    if (!tab || tab.kind !== "opened") return;

    closeTabContextMenu();
    await flushPendingSave(false);

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
    finalizeActiveBlock();
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
    const key = event.key.toLowerCase();

    if (tabContextMenu.open && event.key === "Escape") {
      event.preventDefault();
      closeTabContextMenu();
      return;
    }

    if (showPalette) {
      handlePaletteKeydown(event);
      if (["ArrowDown", "ArrowUp", "Enter", "Escape"].includes(event.key)) {
        return;
      }
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

    loadEditorContent(currentTab.content);
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
        finalizeActiveBlock();
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
          finalizeActiveBlock();
          renderContentToEditor(rawContent);
        }
      });

      document.addEventListener("selectionchange", handleSelectionChange);
      window.addEventListener("keydown", handleGlobalKeydown);
    } catch (error) {
      showStatus(normalizeError(error), "error", 2400);
    }
  });

  onDestroy(() => {
    clearTimeout(saveTimeout);
    clearTimeout(statusTimeout);
    clearTimeout(savedIndicatorTimeout);
    document.removeEventListener("selectionchange", handleSelectionChange);
    window.removeEventListener("keydown", handleGlobalKeydown);
    unlistenShowReader?.();
    unlistenSettingsChanged?.();
  });
</script>

<div
  class="reader-container"
  class:palette-open={showPalette}
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
            on:contextmenu|stopPropagation={(event) =>
              openTabContextMenu(event, index)}
          >
            {#if getTabIcon(tab)}
              <span class="tab-icon" aria-hidden="true">
                <svelte:component
                  this={getTabIcon(tab)}
                  size={14}
                  strokeWidth={1.9}
                />
              </span>
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
        x
      </button>
    </div>
  </div>

  {#if tabContextMenu.open}
    <div
      class="tab-context-backdrop"
      role="presentation"
      on:click={closeTabContextMenu}
      on:contextmenu|preventDefault={closeTabContextMenu}
    >
      <div
        class="tab-context-menu"
        role="menu"
        style={`left: ${tabContextMenu.x}px; top: ${tabContextMenu.y}px;`}
      >
        <button
          class="tab-context-item"
          type="button"
          role="menuitem"
          on:click={() => closeTabByIndex(tabContextMenu.tabIndex)}
        >
          Schliessen
        </button>
      </div>
    </div>
  {/if}

  <div class="editor-scroll" bind:this={scrollRef}>
    {#if fileMissing}
      <div class="missing-file-banner">{missingFileMessage}</div>
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
      on:input={handleInput}
      on:keydown={handleKeydown}
      on:blur={handleEditorBlur}
      on:compositionstart={handleCompositionStart}
      on:compositionend={handleCompositionEnd}
    ></div>
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
    transform: translateZ(0);
    -webkit-transform: translateZ(0);
  }

  .accent-line,
  .reader-topbar,
  .editor-scroll,
  .status-toast {
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .reader-container.palette-open .accent-line,
  .reader-container.palette-open .reader-topbar,
  .reader-container.palette-open .editor-scroll,
  .reader-container.palette-open .status-toast {
    filter: blur(10px) saturate(0.82) brightness(0.62);
    opacity: 0.5;
    transform: scale(0.992);
    pointer-events: none;
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
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .tab-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-context-backdrop {
    position: absolute;
    inset: 0;
    z-index: 115;
  }

  .tab-context-menu {
    position: absolute;
    min-width: 140px;
    padding: 6px;
    border-radius: 12px;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) 82%,
      rgba(0, 0, 0, 0.22)
    );
    color: var(--app-text-color, #ffffff);
    border: 0.5px solid rgba(255, 255, 255, 0.08);
    box-shadow:
      0 18px 40px rgba(0, 0, 0, 0.28),
      0 6px 16px rgba(0, 0, 0, 0.18);
    backdrop-filter: blur(20px) saturate(130%);
    -webkit-backdrop-filter: blur(20px) saturate(130%);
  }

  .tab-context-item {
    width: 100%;
    padding: 9px 10px;
    border-radius: 8px;
    text-align: left;
    cursor: pointer;
    color: inherit;
    font: inherit;
    border: 0;
    background: transparent;
  }

  .tab-context-item:hover {
    background: rgba(255, 255, 255, 0.08);
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
  .palette-item:hover {
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
    background: transparent;
  }

  .missing-file-banner {
    margin: 12px 16px 0;
    padding: 10px 12px;
    border-radius: 12px;
    background: rgba(255, 255, 255, 0.05);
    color: rgba(255, 255, 255, 0.72);
    font-size: 12px;
    border: 0.5px solid rgba(255, 255, 255, 0.08);
  }

  .editor-body {
    min-height: 100%;
    padding: 16px 20px 40px;
    outline: none;
    font-family: inherit;
    font-size: inherit;
    color: inherit;
    line-height: 1.7;
    word-break: break-word;
    white-space: pre-wrap;
    caret-color: var(--accent-color);
    -webkit-user-modify: read-write;
  }

  .editor-body[data-placeholder]:empty::before {
    content: attr(data-placeholder);
    color: var(--placeholder-color);
    pointer-events: none;
  }

  .editor-body :global(p) {
    margin: 0;
    min-height: 1.7em;
  }

  .editor-body :global(h1) {
    margin: 8px 0 4px;
    font-size: 1.5em;
    font-weight: 700;
    line-height: 1.25;
  }

  .editor-body :global(h2) {
    margin: 6px 0 3px;
    font-size: 1.25em;
    font-weight: 600;
    line-height: 1.3;
  }

  .editor-body :global(h3) {
    margin: 4px 0 2px;
    font-size: 1.1em;
    font-weight: 600;
    line-height: 1.35;
  }

  .editor-body :global(h4) {
    margin: 3px 0 2px;
    font-size: 1em;
    font-weight: 600;
    line-height: 1.4;
  }

  .editor-body :global(h5) {
    margin: 2px 0 1px;
    font-size: 0.94em;
    font-weight: 600;
    line-height: 1.45;
    letter-spacing: 0.01em;
  }

  .editor-body :global(h6) {
    margin: 2px 0 1px;
    font-size: 0.88em;
    font-weight: 600;
    line-height: 1.45;
    letter-spacing: 0.02em;
    color: var(--text-secondary);
  }

  .editor-body :global(blockquote) {
    margin: 2px 0;
    padding-left: 12px;
    border-left: 3px solid var(--accent-color);
    color: var(--text-secondary);
  }

  .editor-body :global(code) {
    padding: 1px 5px;
    border-radius: 3px;
    background: rgba(0, 0, 0, 0.08);
    font-family: "SF Mono", Monaco, monospace;
    font-size: 0.88em;
  }

  .editor-body :global(strong) {
    font-weight: 600;
  }

  .editor-body :global(em) {
    font-style: italic;
  }

  .editor-body :global(a) {
    color: var(--accent-color);
    text-decoration: none;
  }

  .editor-body :global(hr) {
    margin: 8px 0;
    border: none;
    border-top: 1px solid var(--border-color);
  }

  .editor-body :global(p.list-item)::before {
    content: "•";
    margin-right: 8px;
    color: var(--text-secondary);
  }

  .editor-body :global(.md-checkbox) {
    margin-right: 6px;
    accent-color: var(--accent-color);
    cursor: pointer;
  }

  .editor-body :global(.wikilink) {
    color: var(--accent-color);
    opacity: 0.85;
  }

  .editor-body :global(.codeblock-pill) {
    display: flex;
    align-items: center;
    width: fit-content;
    gap: 6px;
    margin: 4px 0 4px auto;
    padding: 4px 8px;
    border-radius: 999px;
    background: rgba(255, 255, 255, 0.06);
    border: 0.5px solid rgba(255, 255, 255, 0.1);
    color: rgba(255, 255, 255, 0.78);
    cursor: default;
  }

  .editor-body :global(.codeblock-pill):hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .editor-body :global(.codeblock-icon) {
    width: 6px;
    height: 6px;
    border-radius: 999px;
    background: var(--accent-color);
    opacity: 0.85;
  }

  .editor-body :global(.codeblock-lang) {
    font-size: 11px;
    font-weight: 600;
    text-transform: lowercase;
  }

  .editor-body :global(.hidden-marker) {
    display: none;
  }

  .editor-body :global(.raw-mode) {
    margin-left: -4px;
    padding-left: 4px;
    border-radius: 3px;
    background: rgba(139, 92, 246, 0.04);
  }

  .palette-backdrop {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding: 52px 14px 14px;
    z-index: 120;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) 56%,
      rgba(0, 0, 0, 0.58)
    );
    backdrop-filter: blur(28px) saturate(135%) brightness(0.74);
    -webkit-backdrop-filter: blur(28px) saturate(135%) brightness(0.74);
  }

  .palette-backdrop::before {
    content: "";
    position: absolute;
    inset: 0;
    background:
      linear-gradient(
        180deg,
        rgba(255, 255, 255, 0.04) 0%,
        rgba(0, 0, 0, 0.08) 100%
      ),
      rgba(0, 0, 0, 0.26);
    pointer-events: none;
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
      0 18px 48px rgba(0, 0, 0, 0.24),
      0 6px 18px rgba(0, 0, 0, 0.14);
    transform: translateZ(0);
    -webkit-transform: translateZ(0);
    z-index: 1;
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
