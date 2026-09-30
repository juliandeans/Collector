export {
  getCachedImageSrc,
  setCachedImageSrc,
  clearImageCache,
  resolveImagePath,
  collectImagePaths,
  warmImagesInText,
} from "./imageProcessing.js";

export {
  normalizeNewlines,
  parseRawBlocks,
  stripInlineFields,
  stripHtmlTags,
  preprocessContent,
  replaceCodeblocks,
} from "./contentPreprocess.js";

import { getCachedImageSrc } from "./imageProcessing.js";
import { normalizeNewlines } from "./contentPreprocess.js";
import { indentColumnWidth, parseListLine } from "../capture/list-continuation.js";

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
    const imageTag = `<img src="${escAttr(src)}" alt="${escAttr(cleanPath)}" data-path="${escAttr(cleanPath)}" data-style="${escAttr(style)}" class="md-image" loading="lazy">`;
    imageTokens.push(imageTag);
    return `\u0000IMG${imageTokens.length - 1}\u0000`;
  });

  html = html.replace(/!\[([^\]]*)\]\(([^)]+)\)/g, (_, alt, path) => {
    const cleanPath = path.trim();
    const src = getCachedImageSrc(cleanPath) ?? "";
    const imageTag = `<img src="${escAttr(src)}" alt="${escAttr(alt)}" data-path="${escAttr(cleanPath)}" data-style="max-width:100%;min-height:20px;" class="md-image" loading="lazy">`;
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

function processCallout(lines = [], appSettings = {}) {
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
  const raw = lines.join("\n");

  if (appSettings.reader_hide_callouts) {
    return `<blockquote class="hidden-callout" data-raw="${escAttr(raw)}">${content}</blockquote>`;
  }

  const icon = calloutIcon(type);
  const colorClass = calloutColorClass(type);

  return `<div class="callout callout-${colorClass}" data-raw="${escAttr(raw)}"><div class="callout-title"><span class="callout-icon">${icon}</span><span class="callout-label">${escHtml(title)}</span></div>${content ? `<div class="callout-content">${content}</div>` : ""}</div>`;
}

function listItemToHtml({ indent, marker, spacing, checkbox, checkboxSpacing, text }) {
  const attrs = [
    `data-indent="${escAttr(indent)}"`,
    `data-marker="${escAttr(marker)}"`,
    `data-spacing="${escAttr(spacing)}"`,
    `data-style="--list-indent: ${indentColumnWidth(indent)}; --list-spacing: ${indentColumnWidth(spacing)}"`,
  ];

  if (checkbox === null) {
    return `<p class="list-item" ${attrs.join(" ")}>${inlineMarkdown(text)}</p>`;
  }

  attrs.push(
    `data-checkbox="${escAttr(checkbox)}"`,
    `data-checkbox-spacing="${escAttr(checkboxSpacing)}"`,
  );
  const checked = checkbox === " " ? "" : " checked";
  return `<p class="list-item task" ${attrs.join(" ")}><input type="checkbox" class="md-checkbox" contenteditable="false"${checked}> ${inlineMarkdown(text)}</p>`;
}

// The built app's CSP blocks style attributes; CSSOM writes remain allowed.
export function applyDataStyles(root) {
  if (!root) return;

  const elements = [];
  if (root.hasAttribute?.("data-style")) {
    elements.push(root);
  }
  elements.push(...(root.querySelectorAll?.("[data-style]") ?? []));

  for (const element of elements) {
    element.style.cssText = element.getAttribute("data-style");
    element.removeAttribute("data-style");
  }
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

  const listItem = parseListLine(line);
  if (listItem) return listItemToHtml(listItem);

  if (/^> /.test(line)) {
    return `<blockquote>${inlineMarkdown(line.slice(2))}</blockquote>`;
  }

  return `<p>${inlineMarkdown(line)}</p>`;
}

export function markdownToHtml(text = "", appSettings = {}) {
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

      const callout = processCallout(group, appSettings);
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
