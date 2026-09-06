import { isFileDrag, normalizeFilePath, normalizeImageResult } from "../capture/capture-utils.js";

export function handleDragDropEvent(
    event,
    { setIsDragging, setDragCounter, handleTauriFileDrop },
) {
    const payload = event?.payload;
    if (!payload) return;

    if (payload.type === "enter" || payload.type === "over") {
        setIsDragging(true);
        return;
    }

    if (payload.type === "leave") {
        setIsDragging(false);
        setDragCounter(0);
        return;
    }

    if (payload.type === "drop") {
        setIsDragging(false);
        setDragCounter(0);
        if (Array.isArray(payload.paths)) {
            handleTauriFileDrop(payload.paths);
        }
    }
}

export function handleDragEnter(
    e,
    { getDragCounter, setDragCounter, setIsDragging },
) {
    e.preventDefault();

    if (isFileDrag(e)) {
        const next = getDragCounter() + 1;
        setDragCounter(next);
        if (next === 1) {
            setIsDragging(true);
        }
    }
}

export function handleDragLeave(
    e,
    { getDragCounter, setDragCounter, setIsDragging },
) {
    e.preventDefault();

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX;
    const y = e.clientY;
    const isLeaving =
        x < rect.left || x > rect.right || y < rect.top || y > rect.bottom;

    if (isLeaving) {
        const next = getDragCounter() - 1;
        if (next <= 0) {
            setDragCounter(0);
            setIsDragging(false);
        } else {
            setDragCounter(next);
        }
    }
}

export function handleDragOver(e, { getIsDragging, setIsDragging }) {
    e.preventDefault();
    if (isFileDrag(e)) {
        e.dataTransfer.dropEffect = "copy";
        if (!getIsDragging()) {
            setIsDragging(true);
        }
    } else {
        if (e.dataTransfer) {
            e.dataTransfer.dropEffect = "none";
        }
    }
}

export async function handleDrop(
    e,
    {
        invoke,
        getIsTauri,
        getTextarea,
        getContent,
        setContent,
        updateImages,
        setIsDragging,
        setDragCounter,
        showStatus,
        tick,
    },
) {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(false);
    setDragCounter(0);

    if (!e.dataTransfer) {
        showStatus(
            "Failed to import images: dataTransfer unavailable",
            "error",
        );
        return;
    }

    const items = Array.from(e.dataTransfer.items || []);
    const files = Array.from(e.dataTransfer.files || []);

    if (files.length === 0) {
        showStatus("Failed to import images: no files found", "error");
        return;
    }

    const promises = files.map(async (file, index) => {
        const ext = file.name.split(".").pop()?.toLowerCase();
        if (!["png", "jpg", "jpeg", "webp", "gif"].includes(ext)) {
            showStatus("Unsupported image: " + file.name, "error");
            return null;
        }

        try {
            let filePath = file.path || file.webkitRelativePath || null;

            if (!filePath && items[index]) {
                const item = items[index];
                if (item.kind === "file") {
                    const fileFromItem = item.getAsFile();
                    if (
                        fileFromItem &&
                        (fileFromItem.path || fileFromItem.webkitRelativePath)
                    ) {
                        filePath =
                            fileFromItem.path || fileFromItem.webkitRelativePath;
                    }
                }
            }

            let normalizedResult;

            if (filePath) {
                const result = await invoke("save_image", {
                    filePath: filePath,
                });
                normalizedResult = normalizeImageResult(result);
            } else {
                if (!getIsTauri()) {
                    showStatus(
                        "Failed to import images outside the Tauri app",
                        "error",
                    );
                    return null;
                }

                const arrayBuffer = await file.arrayBuffer();
                const uint8Array = new Uint8Array(arrayBuffer);

                let base64;
                try {
                    base64 = await new Promise((resolve, reject) => {
                        const reader = new FileReader();
                        reader.onload = () => {
                            const result = reader.result;
                            const base64String =
                                typeof result === "string"
                                    ? result.split(",")[1] || result
                                    : "";
                            resolve(base64String);
                        };
                        reader.onerror = reject;
                        reader.readAsDataURL(file);
                    });
                } catch (base64Error) {
                    console.error("Failed to convert to base64:", base64Error);
                    throw new Error(
                        "Failed to encode file: " + base64Error.toString(),
                    );
                }

                try {
                    if (typeof invoke === "undefined") {
                        throw new Error(
                            "invoke is undefined - not running in Tauri",
                        );
                    }
                    const result = await invoke("save_image_from_bytes", {
                        bytesBase64: base64,
                        filename: file.name,
                    });
                    normalizedResult = normalizeImageResult(result);
                } catch (invokeError) {
                    console.error("Invoke error:", invokeError);
                    throw invokeError;
                }
            }

            const previewUrl =
                normalizedResult?.preview_data_url ||
                URL.createObjectURL(file);

            return {
                id: Date.now() + Math.random() + index,
                filename: normalizedResult?.filename || file.name,
                markdown: normalizedResult?.markdown || "",
                preview: previewUrl,
                file: file,
            };
        } catch (e) {
            console.error("Error processing file:", e);
            showStatus("Failed to import image: " + e.toString(), "error");
            return null;
        }
    });

    const results = await Promise.all(promises);
    const validImages = results.filter((img) => img !== null);

    if (validImages.length > 0) {
        const textarea = getTextarea();
        const content = getContent();
        let insertPosition = 0;

        if (textarea) {
            const textareaRect = textarea.getBoundingClientRect();
            const dropX = e.clientX;
            const dropY = e.clientY;
            const isOverTextarea =
                dropX >= textareaRect.left &&
                dropX <= textareaRect.right &&
                dropY >= textareaRect.top &&
                dropY <= textareaRect.bottom;

            if (
                isOverTextarea ||
                e.target === textarea ||
                e.target.closest("textarea") === textarea
            ) {
                if (
                    textarea.selectionStart !== null &&
                    textarea.selectionStart !== undefined
                ) {
                    insertPosition = textarea.selectionStart;
                } else {
                    const lineHeight =
                        parseInt(getComputedStyle(textarea).lineHeight) || 20;
                    const scrollTop = textarea.scrollTop;
                    const relativeY = dropY - textareaRect.top + scrollTop;
                    const estimatedLine = Math.max(
                        0,
                        Math.floor(relativeY / lineHeight),
                    );

                    const lines = content.split("\n");
                    let pos = 0;
                    for (
                        let i = 0;
                        i < Math.min(estimatedLine, lines.length);
                        i++
                    ) {
                        pos += lines[i].length + 1;
                    }
                    insertPosition = Math.min(pos, content.length);
                }
            } else {
                insertPosition = content.length;
            }
        } else {
            insertPosition = content.length;
        }

        let currentPosition = insertPosition;
        let nextContent = content;
        validImages.forEach((img) => {
            const imageMarkdown = img.markdown + "\n";

            if (currentPosition >= nextContent.length) {
                if (nextContent.length > 0 && !nextContent.endsWith("\n")) {
                    nextContent += "\n";
                    currentPosition = nextContent.length;
                }
                nextContent += imageMarkdown;
                currentPosition += imageMarkdown.length;
            } else {
                const before = nextContent.substring(0, currentPosition);
                const after = nextContent.substring(currentPosition);

                const isAtLineStart =
                    before.endsWith("\n") || before.length === 0;

                if (!isAtLineStart && !before.endsWith("\n\n")) {
                    nextContent = before + "\n" + imageMarkdown + after;
                    currentPosition += 1 + imageMarkdown.length;
                } else {
                    nextContent = before + imageMarkdown + after;
                    currentPosition += imageMarkdown.length;
                }
            }
        });

        setContent(nextContent);
        updateImages((prev) => [...prev, ...validImages]);

        if (textarea) {
            await tick();
            const newPosition = currentPosition;
            textarea.setSelectionRange(newPosition, newPosition);
            textarea.focus();
        }
    }
}

export async function handleTauriFileDrop(
    paths,
    {
        invoke,
        getContent,
        setContent,
        updateImages,
        getTextarea,
        setIsDragging,
        setDragCounter,
        showStatus,
    },
) {
    setIsDragging(false);
    setDragCounter(0);

    if (!paths || paths.length === 0) {
        showStatus("Failed to import images: no files found", "error");
        return;
    }

    const imagePaths = paths.filter((path) => {
        const ext = path.split(".").pop()?.toLowerCase();
        return ["png", "jpg", "jpeg", "webp", "gif"].includes(ext);
    });

    if (imagePaths.length === 0) {
        showStatus(
            "Failed to import images: no supported image files found",
            "error",
        );
        return;
    }

    const promises = imagePaths.map(async (filePath, index) => {
        try {
            const result = await invoke("save_image", { filePath: filePath });
            const normalizedResult = normalizeImageResult(result);

            const previewUrl = normalizedResult.preview_data_url || null;

            return {
                id: Date.now() + Math.random() + index,
                filename:
                    normalizedResult.filename ||
                    normalizeFilePath(
                        normalizedResult.saved_path || filePath,
                    )
                        .split("/")
                        .pop() ||
                    `image${index}`,
                markdown: normalizedResult.markdown,
                preview: previewUrl,
                file: null,
            };
        } catch (e) {
            console.error("Error processing file:", e);
            showStatus("Failed to import image: " + e.toString(), "error");
            return null;
        }
    });

    const results = await Promise.all(promises);
    const validImages = results.filter((img) => img !== null);

    if (validImages.length > 0) {
        updateImages((prev) => [...prev, ...validImages]);
        let nextContent = getContent();
        if (nextContent.length > 0 && !nextContent.endsWith("\n")) {
            nextContent += "\n";
        }
        validImages.forEach((img) => {
            nextContent += img.markdown + "\n";
        });
        setContent(nextContent);
    }

    // Required: after the native Tauri drag-drop bridge finishes, macOS needs a
    // short delay before the textarea reliably regains focus.
    setTimeout(() => getTextarea()?.focus(), 50);
}

export function removeImage(id, { getImages, updateImages }) {
    const images = getImages();
    const image = images.find((img) => img.id === id);
    if (image && image.preview && image.preview.startsWith("blob:")) {
        URL.revokeObjectURL(image.preview);
    }
    updateImages((prev) => prev.filter((img) => img.id !== id));
}
