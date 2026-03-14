import { invoke } from "@tauri-apps/api/core";

export const imagePathCache = new Map();

const INLINE_FIELD_LINE_PATTERN = /^\s*\w[\w\s-]*::\s*.*$/;
const INLINE_FIELD_SEGMENT_PATTERN =
  /(^|[\s([{;,:-])(\[?\w[\w\s-]*::\s*(?:[^\s[\](){}<>]+(?:\s+(?!\w[\w\s-]*::)[^\s[\](){}<>]+)*)?\]?)(?=$|[\s)\],;:.!?-])/g;
const HTML_TAG_NAMES = [
  "a",
  "abbr",
  "article",
  "aside",
  "b",
  "blockquote",
  "br",
  "center",
  "code",
  "details",
  "div",
  "em",
  "figcaption",
  "figure",
  "footer",
  "h1",
  "h2",
  "h3",
  "h4",
  "h5",
  "h6",
  "header",
  "hr",
  "i",
  "iframe",
  "img",
  "li",
  "main",
  "mark",
  "ol",
  "p",
  "pre",
  "script",
  "section",
  "small",
  "span",
  "strong",
  "style",
  "sub",
  "summary",
  "sup",
  "table",
  "tbody",
  "td",
  "tfoot",
  "th",
  "thead",
  "tr",
  "u",
  "ul",
];
const HTML_TAG_PATTERN = new RegExp(
  `<\\/?(?:${HTML_TAG_NAMES.join("|")})(?=[\\s>/])[^>]*>`,
  "gi",
);

export function normalizeNewlines(content = "") {
  return content.replace(/\r\n/g, "\n");
}

function stashHiddenBlock(block = "", hiddenBlockMap = new Map()) {
  const hiddenId = `__HD_${hiddenBlockMap.size}__`;
  hiddenBlockMap.set(hiddenId, block);
  return `\u200B${hiddenId}`;
}

export function getCachedImageSrc(path) {
  return imagePathCache.get(path) ?? "";
}

export function setCachedImageSrc(path, src) {
  if (!path || !src) return;
  imagePathCache.set(path, src);
}

export function parseRawBlocks(content = "") {
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

export function stripInlineFields(text = "", hiddenBlockMap = new Map()) {
  return text
    .split("\n")
    .map((line) => {
      if (INLINE_FIELD_LINE_PATTERN.test(line)) {
        return stashHiddenBlock(line, hiddenBlockMap);
      }

      const stripped = line.replace(
        INLINE_FIELD_SEGMENT_PATTERN,
        (_, prefix, field) =>
          `${prefix}${stashHiddenBlock(field, hiddenBlockMap)}`,
      );

      return stripped;
    })
    .join("\n");
}

export function stripHtmlTags(text = "", hiddenBlockMap = new Map()) {
  return text
    .split("\n")
    .map((line) => {
      HTML_TAG_PATTERN.lastIndex = 0;
      if (!HTML_TAG_PATTERN.test(line)) {
        return line;
      }

      HTML_TAG_PATTERN.lastIndex = 0;
      const withoutTags = line.replace(HTML_TAG_PATTERN, (match) =>
        stashHiddenBlock(match, hiddenBlockMap),
      );
      const visibleContent = withoutTags
        .replace(/(?:\u200B)?__HD_\d+__/g, "")
        .trim();

      if (visibleContent) {
        return withoutTags;
      }

      return stashHiddenBlock(line, hiddenBlockMap);
    })
    .join("\n");
}

export function preprocessContent(
  raw = "",
  {
    appSettings = {},
    codeblockMap = new Map(),
    hiddenBlockMap = new Map(),
    setStrippedFrontmatter = () => {},
  } = {},
) {
  let text = normalizeNewlines(raw);

  setStrippedFrontmatter("");
  codeblockMap.clear();
  hiddenBlockMap.clear();

  if (appSettings.reader_hide_frontmatter) {
    text = text.replace(/^---\n[\s\S]*?\n---[ \t]*(?:\n+)?/, (match) => {
      setStrippedFrontmatter(match.trimEnd());
      return "";
    });
  }

  if (appSettings.reader_hide_obsidian_comments) {
    text = text.replace(/%%[\s\S]*?%%[ \t]*/g, (match) => {
      return stashHiddenBlock(match, hiddenBlockMap);
    });
  }

  if (appSettings.reader_hide_dataview) {
    text = replaceCodeblocks(text, codeblockMap);
  }

  if (appSettings.reader_hide_inline_fields) {
    text = stripInlineFields(text, hiddenBlockMap);
  }

  if (appSettings.reader_hide_html) {
    text = stripHtmlTags(text, hiddenBlockMap);
  }

  return text.replace(/^\n+/, "");
}

export function replaceCodeblocks(text = "", codeblockMap = new Map()) {
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

function escAttr(text = "") {
  return escHtml(text).replace(/"/g, "&quot;");
}

function normalizeImageWidth(rawWidth = "") {
  const trimmed = rawWidth.trim();
  if (!trimmed) return null;

  const width = Number.parseInt(trimmed, 10);
  if (!Number.isFinite(width) || width <= 0 || width > 4000) {
    return null;
  }

  return `${width}px`;
}

function sanitizeExternalHref(rawHref = "") {
  const trimmed = rawHref.trim();
  if (!trimmed) return null;

  if (/^https?:\/\//i.test(trimmed) || /^obsidian:\/\//i.test(trimmed)) {
    return trimmed;
  }

  return null;
}

function splitTrailingUrlPunctuation(rawUrl = "") {
  let url = rawUrl;
  let trailing = "";

  while (/[),.!?;:]$/.test(url)) {
    trailing = `${url.slice(-1)}${trailing}`;
    url = url.slice(0, -1);
  }

  return { url, trailing };
}

export async function resolveImagePath(rawPath = "") {
  const cleanPath = rawPath.split("|")[0]?.trim() ?? "";
  if (!cleanPath) return "";

  const cached = getCachedImageSrc(cleanPath);
  if (cached) {
    return cached;
  }

  try {
    const src = await invoke("load_image_data_url", {
      path: cleanPath,
    });
    setCachedImageSrc(cleanPath, src);
    return src;
  } catch (error) {
    console.warn("Could not resolve image path:", cleanPath, error);
    return "";
  }
}

export function collectImagePaths(text = "") {
  const wikiImageRegex = /!\[\[([^\]]+)\]\]/g;
  const mdImageRegex = /!\[([^\]]*)\]\(([^)]+)\)/g;
  const imagePaths = new Set();
  let match;

  while ((match = wikiImageRegex.exec(text)) !== null) {
    imagePaths.add(match[1].split("|")[0].trim());
  }

  while ((match = mdImageRegex.exec(text)) !== null) {
    imagePaths.add(match[2].trim());
  }

  return [...imagePaths].filter((path) => path);
}

export async function warmImagesInText(text = "") {
  const missingPaths = collectImagePaths(text).filter(
    (path) => !getCachedImageSrc(path),
  );

  if (missingPaths.length === 0) {
    return false;
  }

  await Promise.allSettled(missingPaths.map((path) => resolveImagePath(path)));

  return true;
}

export function inlineMarkdown(text = "") {
  const imageTokens = [];
  const linkTokens = [];
  const hiddenTokens = [];
  let html = text;

  html = html.replace(/!\[\[([^\]]+)\]\]/g, (_, inner) => {
    const [rawPath = "", rawWidth = ""] = inner.split("|");
    const cleanPath = rawPath.trim();
    const widthValue = normalizeImageWidth(rawWidth);
    const src = getCachedImageSrc(cleanPath) ?? "";
    const style = widthValue
      ? `width:${widthValue};max-width:100%;min-height:20px;`
      : "max-width:100%;min-height:20px;";
    const imageTag = `<img src="${escAttr(src)}" alt="${escAttr(cleanPath)}" data-path="${escAttr(cleanPath)}" style="${escAttr(style)}" class="md-image" loading="lazy">`;
    imageTokens.push(imageTag);
    return `\u0000IMG${imageTokens.length - 1}\u0000`;
  });

  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_, alt, path) => {
    const cleanPath = path.trim();
    const src = getCachedImageSrc(cleanPath) ?? "";
    const imageTag = `<img src="${escAttr(src)}" alt="${escAttr(alt)}" data-path="${escAttr(cleanPath)}" style="max-width:100%;min-height:20px;" class="md-image" loading="lazy">`;
    imageTokens.push(imageTag);
    return `\u0000IMG${imageTokens.length - 1}\u0000`;
  });

  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, (fullMatch, label, href) => {
    const safeHref = sanitizeExternalHref(href);
    if (!safeHref) {
      return fullMatch;
    }

    const linkTag = `<a href="${escAttr(safeHref)}" target="_blank" rel="noopener noreferrer">${escHtml(label)}</a>`;
    linkTokens.push(linkTag);
    return `\u0000LNK${linkTokens.length - 1}\u0000`;
  });

  html = html.replace(/\bhttps?:\/\/[^\s<]+/gi, (rawUrl) => {
    const { url, trailing } = splitTrailingUrlPunctuation(rawUrl);
    const safeHref = sanitizeExternalHref(url);
    if (!safeHref) {
      return rawUrl;
    }

    const linkTag = `<a href="${escAttr(safeHref)}" target="_blank" rel="noopener noreferrer">${escHtml(url)}</a>`;
    linkTokens.push(linkTag);
    return `\u0000LNK${linkTokens.length - 1}\u0000${trailing}`;
  });

  html = html.replace(/(?:\u200B)?(__HD_\d+__)/g, (_, hiddenId) => {
    hiddenTokens.push(
      `<span class="hidden-inline-marker" data-hidden-id="${hiddenId}" contenteditable="false"></span>`,
    );
    return `\u0000HDN${hiddenTokens.length - 1}\u0000`;
  });

  html = escHtml(html);
  html = html.replace(/\*\*(.+?)\*\*/g, "<strong>$1</strong>");
  html = html.replace(/\*(.+?)\*/g, "<em>$1</em>");
  html = html.replace(/`(.+?)`/g, "<code>$1</code>");
  html = html.replace(/\[\[([^\]]+)\]\]/g, (_, inner) => {
    const [rawTarget = "", rawDisplay = ""] = inner.split("|");
    const target = rawTarget.trim();
    const display = (rawDisplay || rawTarget).trim();
    return `<span class="wikilink" data-target="${escAttr(target)}">[[${escHtml(display)}]]</span>`;
  });
  html = html.replace(/\u0000LNK(\d+)\u0000/g, (_, index) => {
    return linkTokens[Number(index)] ?? "";
  });
  html = html.replace(/\u0000IMG(\d+)\u0000/g, (_, index) => {
    return imageTokens[Number(index)] ?? "";
  });
  html = html.replace(/\u0000HDN(\d+)\u0000/g, (_, index) => {
    return hiddenTokens[Number(index)] ?? "";
  });
  return html;
}

function capitalize(text = "") {
  return text ? `${text.charAt(0).toUpperCase()}${text.slice(1)}` : "";
}

function calloutIcon(type) {
  const icons = {
    note: "ℹ",
    info: "ℹ",
    tip: "💡",
    hint: "💡",
    warning: "⚠",
    caution: "⚠",
    attention: "⚠",
    danger: "🔥",
    error: "✗",
    bug: "🐛",
    success: "✓",
    check: "✓",
    done: "✓",
    question: "?",
    help: "?",
    faq: "?",
    quote: '"',
    cite: '"',
    abstract: "◻",
    summary: "◻",
    tldr: "◻",
    example: "◈",
    important: "★",
  };

  return icons[type] ?? "ℹ";
}

function calloutColorClass(type) {
  const map = {
    note: "blue",
    info: "blue",
    abstract: "blue",
    summary: "blue",
    tip: "green",
    hint: "green",
    success: "green",
    check: "green",
    done: "green",
    warning: "yellow",
    caution: "yellow",
    attention: "yellow",
    danger: "red",
    error: "red",
    bug: "red",
    question: "purple",
    help: "purple",
    faq: "purple",
    quote: "gray",
    cite: "gray",
    example: "purple",
    important: "orange",
  };

  return map[type] ?? "blue";
}

function processCallout(lines = []) {
  const firstLine = lines[0] ?? "";
  const calloutMatch = firstLine.match(/^>\s*\[!([\w]+)\]\s*(.*)/i);
  if (!calloutMatch) return null;

  const type = calloutMatch[1].toLowerCase();
  const title = calloutMatch[2].trim() || capitalize(type);
  const contentLines = lines.slice(1).map((line) => line.replace(/^>\s?/, ""));

  while (contentLines.length > 0 && !contentLines[0].trim()) {
    contentLines.shift();
  }

  while (
    contentLines.length > 0 &&
    !contentLines[contentLines.length - 1].trim()
  ) {
    contentLines.pop();
  }

  const content = contentLines.length
    ? contentLines.map((line) => inlineMarkdown(line)).join("<br>")
    : "";
  const icon = calloutIcon(type);
  const colorClass = calloutColorClass(type);
  const raw = lines.join("\n");

  return `<div class="callout callout-${colorClass}" data-raw="${escAttr(raw)}"><div class="callout-title"><span class="callout-icon">${icon}</span><span class="callout-label">${escHtml(title)}</span></div>${content ? `<div class="callout-content">${content}</div>` : ""}</div>`;
}

export function markdownLineToHtml(line) {
  if (line === null || line === undefined) return "";
  const trimmed = line.trim();
  const codeblockMatch = trimmed.match(/\u200B(__CB_\d+__):([\w-]*)\u200B/);
  if (codeblockMatch) {
    const [, id, langValue] = codeblockMatch;
    const lang = langValue || "code";
    return `<div class="codeblock-pill" data-cbid="${id}" data-cblang="${escHtml(lang)}" contenteditable="false"><span class="codeblock-icon"></span><span class="codeblock-lang">${escHtml(lang)}</span></div>`;
  }

  const hiddenMatch = trimmed.match(/^(?:\u200B)?(__HD_\d+__)$/);
  if (hiddenMatch) {
    return `<div class="hidden-marker" data-hidden-id="${hiddenMatch[1]}" contenteditable="false"></div>`;
  }

  if (/^###### /.test(line)) return `<h6>${inlineMarkdown(line.slice(7))}</h6>`;
  if (/^##### /.test(line)) return `<h5>${inlineMarkdown(line.slice(6))}</h5>`;
  if (/^#### /.test(line)) return `<h4>${inlineMarkdown(line.slice(5))}</h4>`;
  if (/^### /.test(line)) return `<h3>${inlineMarkdown(line.slice(4))}</h3>`;
  if (/^## /.test(line)) return `<h2>${inlineMarkdown(line.slice(3))}</h2>`;
  if (/^# /.test(line)) return `<h1>${inlineMarkdown(line.slice(2))}</h1>`;
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

export function markdownToHtml(text = "") {
  if (!text.trim()) return "";

  const lines = normalizeNewlines(text).split("\n");
  const htmlParts = [];
  let index = 0;

  while (index < lines.length) {
    const line = lines[index];

    if (/^>\s?/.test(line)) {
      const group = [];
      while (index < lines.length && /^>\s?/.test(lines[index])) {
        group.push(lines[index]);
        index += 1;
      }

      const callout = processCallout(group);
      if (callout) {
        htmlParts.push(callout);
      } else {
        const content = group
          .map((groupLine) => inlineMarkdown(groupLine.replace(/^>\s?/, "")))
          .join("<br>");
        htmlParts.push(`<blockquote>${content}</blockquote>`);
      }
      continue;
    }

    htmlParts.push(markdownLineToHtml(line));
    index += 1;
  }

  return htmlParts.join("");
}
