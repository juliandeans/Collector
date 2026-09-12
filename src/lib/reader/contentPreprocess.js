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
