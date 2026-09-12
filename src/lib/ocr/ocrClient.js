export function normalizeOcrError(err) {
    if (!err) return "OCR failed";
    const msg = typeof err === "string" ? err : err.message || String(err);
    if (msg.includes("No image in clipboard")) {
        return "No image in clipboard";
    }
    if (msg.includes("No text recognized")) {
        return "No text recognized";
    }
    return msg;
}

export async function getModifierFlags({ invoke }) {
    try {
        return await invoke("get_modifier_flags");
    } catch {
        return { alt: false, shift: false, ctrl: false, meta: false };
    }
}

export function isOcrModifierActive(flags, configuredModifier = "Option") {
    if (!flags) return false;
    const mod = String(configuredModifier).trim().toLowerCase();
    if (mod === "option" || mod === "alt") {
        return Boolean(flags.altKey ?? flags.alt);
    }
    if (mod === "shift") {
        return Boolean(flags.shiftKey ?? flags.shift);
    }
    if (mod === "control" || mod === "ctrl") {
        return Boolean(flags.ctrlKey ?? flags.ctrl);
    }
    if (mod === "command" || mod === "cmd" || mod === "meta") {
        return Boolean(flags.metaKey ?? flags.meta);
    }
    return false;
}

export async function ocrClipboardImage({ invoke }) {
    try {
        const result = await invoke("ocr_clipboard_image");
        if (!result || !result.text || !result.text.trim()) {
            throw new Error("No text recognized");
        }
        return result;
    } catch (err) {
        throw new Error(normalizeOcrError(err));
    }
}

export async function ocrFilePath(path, { invoke }) {
    try {
        const result = await invoke("ocr_image_file", { path });
        if (!result || !result.text || !result.text.trim()) {
            throw new Error("No text recognized");
        }
        return result;
    } catch (err) {
        throw new Error(normalizeOcrError(err));
    }
}

export async function ocrImageBytes(bytesBase64, { invoke }) {
    try {
        const result = await invoke("ocr_image_bytes", { bytesBase64 });
        if (!result || !result.text || !result.text.trim()) {
            throw new Error("No text recognized");
        }
        return result;
    } catch (err) {
        throw new Error(normalizeOcrError(err));
    }
}

export async function ocrFile(file, { invoke }) {
    if (file?.path) {
        return await ocrFilePath(file.path, { invoke });
    }
    const base64 = await new Promise((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () => {
            const dataUrl = typeof reader.result === "string" ? reader.result : "";
            const b64 = dataUrl.split(",")[1] || "";
            resolve(b64);
        };
        reader.onerror = () => reject(new Error("Failed to read image file"));
        reader.readAsDataURL(file);
    });
    return await ocrImageBytes(base64, { invoke });
}
