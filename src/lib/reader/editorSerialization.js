import { normalizeNewlines } from "./contentProcessing.js";

export function imageNodeToMarkdown(node) {
  const alt = node.getAttribute?.("alt") ?? "";
  const style = node.getAttribute?.("style") ?? "";
  const widthMatch = style.match(/width:\s*(\d+)px/i);
  const width = widthMatch ? widthMatch[1] : null;
  const isWikilink = !alt.includes("http") && !alt.startsWith("/");
  if (isWikilink) {
    return width ? `![[${alt}|${width}]]` : `![[${alt}]]`;
  }

  return `![${alt}](${alt})`;
}

export function elementInnerToMarkdown(el) {
  let result = "";

  el.childNodes.forEach((node) => {
    if (node.nodeType === Node.TEXT_NODE) {
      result += node.textContent ?? "";
      return;
    }

    if (node.nodeType !== Node.ELEMENT_NODE) {
      return;
    }

    const tag = node.tagName?.toLowerCase();
    const text = node.innerText ?? node.textContent ?? "";

    if (tag === "img") {
      result += imageNodeToMarkdown(node);
      return;
    }

    if (tag === "strong" || tag === "b") {
      result += `**${text}**`;
      return;
    }

    if (tag === "em" || tag === "i") {
      result += `*${text}*`;
      return;
    }

    if (tag === "code") {
      result += `\`${text}\``;
      return;
    }

    if (tag === "a") {
      result += `[${text}](${node.href})`;
      return;
    }

    if (tag === "span" && node.classList?.contains("wikilink")) {
      result += `[[${node.dataset.target ?? text}]]`;
      return;
    }

    result += text;
  });

  return result;
}

export function elementToMarkdownLine(el) {
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

  if (el.classList?.contains("callout")) {
    return el.dataset.raw ?? "";
  }

  if (el.classList?.contains("hidden-callout")) {
    return el.dataset.raw ?? "";
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
      const text = (el.innerText ?? "").trim();
      return `${checkbox.checked ? "- [x] " : "- [ ] "}${text}`;
    }

    if (el.classList.contains("list-item")) {
      return `- ${elementInnerToMarkdown(el)}`;
    }

    const paragraphMarkdown = elementInnerToMarkdown(el);
    if (!paragraphMarkdown.trim()) return "";
    return paragraphMarkdown;
  }

  return inner;
}

export function serializeInline(node) {
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

  if (tag === "img") {
    return imageNodeToMarkdown(element);
  }

  if (tag === "a") {
    const href = element.getAttribute("href") ?? "";
    return `[${serializeChildren(element)}](${href})`;
  }

  if (element.classList.contains("callout")) {
    return element.dataset.raw ?? "";
  }

  if (tag === "span" && element.classList.contains("hidden-inline-marker")) {
    return element.dataset.hiddenId ?? "";
  }

  if (tag === "span" && element.classList.contains("wikilink")) {
    return `[[${element.dataset.target ?? element.textContent ?? ""}]]`;
  }

  return serializeChildren(element);
}

export function serializeChildren(node, { skipCheckbox = false } = {}) {
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

export function htmlToMarkdown(el) {
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

    if (element.classList.contains("callout")) {
      lines.push(element.dataset.raw ?? "");
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

    if (tag === "img") {
      lines.push(imageNodeToMarkdown(element));
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
        lines.push(`- ${elementInnerToMarkdown(element).trim()}`);
        return;
      }

      const text = elementInnerToMarkdown(element).trim();
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

export function restoreCodeblocks(markdown = "", codeblockMap = new Map()) {
  let restored = markdown;
  restored = restored.replace(
    /\u200B(__CB_\d+__):[^\u200B]*\u200B/g,
    (_, id) => codeblockMap.get(id) ?? "",
  );

  return restored;
}

export function restoreHiddenBlocks(markdown = "", hiddenBlockMap = new Map()) {
  let restored = markdown;

  hiddenBlockMap.forEach((block, id) => {
    const escaped = id.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
    restored = restored.replace(new RegExp(`(?:\\u200B)?${escaped}`, "g"), block);
  });

  return restored;
}

export function composeContentFromMarkdown(
  markdown = "",
  {
    strippedFrontmatter = "",
    hiddenBlockMap = new Map(),
    codeblockMap = new Map(),
  } = {},
) {
  const normalized = normalizeNewlines(markdown);
  const withHiddenBlocks = restoreHiddenBlocks(normalized, hiddenBlockMap);
  const restored = restoreCodeblocks(withHiddenBlocks, codeblockMap);
  const frontmatter = strippedFrontmatter.trim();

  if (frontmatter && restored.trim()) {
    return `${frontmatter}\n\n${restored}`;
  }

  if (frontmatter) {
    return frontmatter;
  }

  return restored;
}
