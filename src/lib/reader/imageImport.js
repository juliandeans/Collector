import { invoke } from "@tauri-apps/api/core";

function fileExtension(name = "") {
  return name.split(".").pop()?.toLowerCase() ?? "";
}

function basename(path = "") {
  return path.split(/[\\/]/).pop() ?? path;
}

function normalizeImportedImageResult(result) {
  if (typeof result === "string") {
    return { markdown: result };
  }

  return {
    markdown: result?.markdown ?? "",
  };
}

function fileToBase64(file) {
  return new Promise((resolve, reject) => {
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
}

export function buildImageMarkdown(filename, width) {
  return width ? `![[${filename}|${width}]]` : `![[${filename}]]`;
}

export function createImportPlaceholder(filename = "image") {
  const label = filename.replace(/\s+/g, " ").trim() || "image";
  return `[Importing image: ${label} · ${Date.now()}-${Math.random().toString(36).slice(2, 8)}]`;
}

export function isImageFile(filename = "") {
  return ["png", "jpg", "jpeg", "webp", "gif"].includes(
    fileExtension(filename),
  );
}

export async function processDroppedPaths(paths = [], settings = {}) {
  void settings;

  return Promise.all(
    paths.map(async (path) => {
      if (!isImageFile(path)) {
        throw new Error(`Unsupported image: ${path || "file"}`);
      }

      const result = await invoke("save_image", {
        filePath: path,
      });

      return {
        markdown: normalizeImportedImageResult(result).markdown,
        filename: basename(path),
        previewUrl: "",
      };
    }),
  );
}

export async function processDroppedFiles(files = [], settings = {}) {
  void settings;

  return Promise.all(
    files.map(async (file) => {
      const filename = file?.name || "clipboard-image.png";
      if (!isImageFile(filename)) {
        throw new Error(`Unsupported image: ${filename}`);
      }

      const base64 = await fileToBase64(file);
      const result = await invoke("save_image_from_bytes", {
        bytesBase64: base64,
        filename,
      });

      return {
        markdown: normalizeImportedImageResult(result).markdown,
        filename,
        previewUrl: "",
      };
    }),
  );
}
