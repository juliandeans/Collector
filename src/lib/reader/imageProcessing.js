import { invoke } from "@tauri-apps/api/core";

const MAX_CACHE_ENTRIES = 150;

const imagePathCache = new Map();

export function getCachedImageSrc(path) {
  return imagePathCache.get(path) ?? "";
}

export function setCachedImageSrc(path, src) {
  if (!path || !src) return;
  imagePathCache.set(path, src);
  if (imagePathCache.size > MAX_CACHE_ENTRIES) {
    const oldestKey = imagePathCache.keys().next().value;
    if (oldestKey !== undefined) {
      imagePathCache.delete(oldestKey);
    }
  }
}

export function clearImageCache() {
  imagePathCache.clear();
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

  try {
    const results = await invoke("load_images_batch", {
      paths: missingPaths,
    });

    for (const [path, dataUrl] of results) {
      setCachedImageSrc(path, dataUrl);
    }

    return results.length > 0;
  } catch (error) {
    console.warn("Batch image load failed:", error);
    return false;
  }
}
