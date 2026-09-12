import {
  ocrClipboardImage,
  ocrFilePath,
  getModifierFlags,
  isOcrModifierActive,
  normalizeOcrError,
} from "../ocr/ocrClient.js";

const SUPPORTED_EXTS = [
  "png",
  "jpg",
  "jpeg",
  "webp",
  "gif",
  "tiff",
  "tif",
  "bmp",
  "heic",
  "heif",
];

export function createReaderOcr({
  invoke,
  getAppSettings,
  getEditorComponent,
  showStatus,
  setIsDragging,
  setIsOcrLoading,
  getIsOcrLoading,
}) {
  async function handleOcrClipboard() {
    if (getIsOcrLoading()) return;
    setIsOcrLoading(true);
    showStatus("Extracting text…", "info", 1500);
    try {
      const result = await ocrClipboardImage({ invoke });
      if (!result?.text) {
        showStatus("No text recognized", "error", 2400);
        return;
      }
      const success = getEditorComponent()?.insertPlainText(result.text);
      if (success) {
        showStatus("Text extracted", "success", 1500);
      } else {
        showStatus("Failed to insert text", "error", 2400);
      }
    } catch (err) {
      showStatus(normalizeOcrError(err), "error", 2400);
    } finally {
      setIsOcrLoading(false);
    }
  }

  async function handleDragDropEvent(event) {
    const payload = event?.payload;
    if (!payload) return;

    if (payload.type === "enter" || payload.type === "over") {
      setIsDragging(true);
      return;
    }

    if (payload.type === "leave") {
      setIsDragging(false);
      return;
    }

    if (payload.type === "drop") {
      setIsDragging(false);
      const paths = payload.paths;
      if (!Array.isArray(paths) || paths.length === 0) return;

      const appSettings = getAppSettings();
      const modifierFlags = await getModifierFlags({ invoke });
      const isOcr = isOcrModifierActive(modifierFlags, appSettings?.ocr_drag_modifier);

      // Reader ignores image drop without OCR modifier
      if (!isOcr) {
        return;
      }

      if (getIsOcrLoading()) return;

      const imagePaths = paths.filter((path) => {
        const ext = path.split(".").pop()?.toLowerCase();
        return SUPPORTED_EXTS.includes(ext);
      });

      if (imagePaths.length === 0) {
        showStatus("No supported image files found", "error", 2400);
        return;
      }

      setIsOcrLoading(true);
      try {
        showStatus("Extracting text…", "info", 1500);
        const ocrTexts = [];
        let failedCount = 0;
        for (const filePath of imagePaths) {
          try {
            const res = await ocrFilePath(filePath, { invoke });
            if (res?.text) {
              ocrTexts.push(res.text);
            } else {
              failedCount++;
            }
          } catch {
            failedCount++;
          }
        }
        if (ocrTexts.length === 0) {
          showStatus("No text recognized", "error", 2400);
          return;
        }
        const combinedText = ocrTexts.join("\n\n");
        const success = getEditorComponent()?.insertPlainText(combinedText);
        if (success) {
          showStatus(
            failedCount > 0
              ? `Text extracted (${failedCount} image${failedCount > 1 ? "s" : ""} failed)`
              : "Text extracted",
            "success",
            1500,
          );
        } else {
          showStatus("Failed to insert text", "error", 2400);
        }
      } finally {
        setIsOcrLoading(false);
      }
    }
  }

  return {
    handleOcrClipboard,
    handleDragDropEvent,
  };
}
