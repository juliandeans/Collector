<script>
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import { listen } from "@tauri-apps/api/event";
  import { onMount, onDestroy } from "svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

  let textareaRef;
  let content = "";
  let isDragging = false;
  let isLoading = false;
  let statusMessage = "";
  let statusType = "";
  let uploadedImages = [];
  let unlistenShow;
  let unlistenSettingsChanged;
  let unlistenDragDrop;
  let isTauri = false;
  let globalDragEnter;
  let globalDragOver;
  let globalDragLeave;
  let globalDrop;

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
    save_to_daily_shortcut: "Cmd+Enter",
    save_as_note_shortcut: "Cmd+Shift+Enter",
  };

  $: brightnessFilter = (() => {
    const b = appSettings.window_brightness;
    if (b === 0) return "";

    if (b > 0) {
      const brightnessValue = 1 + (b / 100) * 0.6;
      const contrastValue = 1 - (b / 100) * 0.25;
      return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
    } else {
      const brightnessValue = 1 + (b / 100) * 0.7;
      const contrastValue = 1 + (-b / 100) * 0.3;
      return ` brightness(${brightnessValue}) contrast(${contrastValue})`;
    }
  })();

  function isFileDrag(e) {
    const types = e.dataTransfer?.types;
    if (types && Array.from(types).includes("Files")) return true;
    const items = e.dataTransfer?.items;
    return !!items && items.length > 0;
  }

  function normalizeFilePath(path) {
    if (!path) return "";
    if (path.startsWith("file://")) {
      return decodeURIComponent(path.replace("file://", ""));
    }
    return path;
  }

  function normalizeImageResult(result) {
    if (typeof result === "string") {
      return { markdown: result, saved_path: null, filename: null };
    }
    return {
      markdown: result?.markdown ?? "",
      saved_path: result?.saved_path ?? null,
      filename: result?.filename ?? null,
    };
  }

  function handleDragDropEvent(event) {
    const payload = event?.payload;
    if (!payload) return;

    if (payload.type === "enter" || payload.type === "over") {
      isDragging = true;
      return;
    }

    if (payload.type === "leave") {
      isDragging = false;
      dragCounter = 0;
      return;
    }

    if (payload.type === "drop") {
      isDragging = false;
      dragCounter = 0;
      if (Array.isArray(payload.paths)) {
        handleTauriFileDrop(payload.paths);
      }
    }
  }

  onMount(async () => {
    try {
      await getCurrentWindow();
      isTauri = true;
    } catch (e) {
      isTauri = false;
    }

    globalDragEnter = (e) => {
      if (!isFileDrag(e)) return;
      e.preventDefault();
      dragCounter = Math.max(dragCounter, 1);
      isDragging = true;
    };

    globalDragOver = (e) => {
      e.preventDefault();
      if (isFileDrag(e)) {
        e.dataTransfer.dropEffect = "copy";
      }
    };

    globalDragLeave = (e) => {
      if (e.target === document.documentElement) {
        dragCounter = 0;
        isDragging = false;
      }
    };

    globalDrop = (e) => {
      const container = e.target?.closest?.(".capture-container");
      if (container) {
        e.preventDefault();
        e.stopPropagation();
        handleDrop(e);
      }
    };

    document.addEventListener("dragenter", globalDragEnter, true);
    document.addEventListener("dragover", globalDragOver, true);
    document.addEventListener("dragleave", globalDragLeave, true);
    document.addEventListener("drop", globalDrop, true);

    if (isTauri) {
      try {
        unlistenShow = await listen("show_capture", () => {
          uploadedImages.forEach((img) => {
            if (img.preview && img.preview.startsWith("blob:")) {
              URL.revokeObjectURL(img.preview);
            }
          });
          content = "";
          uploadedImages = [];
          statusMessage = "";
          isDragging = false;
          dragCounter = 0;
          isLoading = false;
          setTimeout(() => textareaRef?.focus(), 50);
        });

        await listen("insert_capture_text", (event) => {
          const text = typeof event.payload === "string" ? event.payload : "";
          if (!text.trim()) return;
          content = text;
          setTimeout(() => textareaRef?.focus(), 50);
        });

        await listen("save_as_note", () => {
          handleSaveAsNote();
        });

        unlistenSettingsChanged = await listen("settings_changed", (event) => {
          const newSettings = event.payload;

          appSettings = {
            ...appSettings,
            background_color:
              newSettings.background_color ?? appSettings.background_color,
            font_family: newSettings.font_family ?? appSettings.font_family,
            font_size: newSettings.font_size ?? appSettings.font_size,
            border_radius:
              newSettings.border_radius ?? appSettings.border_radius,
            window_transparency:
              newSettings.window_transparency ??
              appSettings.window_transparency,
            window_blur: newSettings.window_blur ?? appSettings.window_blur,
            window_saturation:
              newSettings.window_saturation ?? appSettings.window_saturation,
            window_brightness:
              newSettings.window_brightness ?? appSettings.window_brightness,
            text_color: newSettings.text_color ?? appSettings.text_color,
            save_to_daily_shortcut:
              newSettings.save_to_daily_shortcut ??
              appSettings.save_to_daily_shortcut,
            save_as_note_shortcut:
              newSettings.save_as_note_shortcut ??
              appSettings.save_as_note_shortcut,
          };
        });

        const currentWindow = await getCurrentWindow();
        unlistenDragDrop = await currentWindow.onDragDropEvent(
          handleDragDropEvent,
        );

        try {
          const settings = await invoke("load_settings");
          appSettings = {
            background_color: settings.background_color,
            font_family: settings.font_family,
            font_size: settings.font_size,
            border_radius: settings.border_radius,
            window_transparency: settings.window_transparency ?? 55,
            window_blur: settings.window_blur ?? 80,
            window_saturation: settings.window_saturation ?? 200,
            window_brightness: settings.window_brightness ?? 0,
            text_color: settings.text_color ?? "#ffffff",
            save_to_daily_shortcut:
              settings.save_to_daily_shortcut ?? "Cmd+Enter",
            save_as_note_shortcut:
              settings.save_as_note_shortcut ?? "Cmd+Shift+Enter",
          };
        } catch (e) {
          console.error("Failed to load initial settings:", e);
        }
      } catch (e) {
        console.error("Failed to listen to events:", e);
      }
    }
  });

  onDestroy(() => {
    unlistenShow?.();
    unlistenSettingsChanged?.();
    unlistenDragDrop?.();

    if (globalDragEnter) {
      document.removeEventListener("dragenter", globalDragEnter, true);
    }
    if (globalDragOver) {
      document.removeEventListener("dragover", globalDragOver, true);
    }
    if (globalDragLeave) {
      document.removeEventListener("dragleave", globalDragLeave, true);
    }
    if (globalDrop) {
      document.removeEventListener("drop", globalDrop, true);
    }

    uploadedImages.forEach((img) => {
      if (img.preview && img.preview.startsWith("blob:")) {
        URL.revokeObjectURL(img.preview);
      }
    });
  });

  function showStatus(message, type = "success") {
    if (type !== "error") return;
    statusMessage = message;
    statusType = type;
    setTimeout(() => (statusMessage = ""), 2000);
  }

  async function handleAppendToDaily() {
    if (!content.trim() || isLoading) {
      if (!content.trim()) {
        showStatus("Nothing to append", "error");
      }
      return;
    }

    isLoading = true;

    try {
      await invoke("append_to_daily_note", {
        text: content.trim(),
      });

      showStatus("✓ Saved", "success");

      uploadedImages.forEach((img) => {
        if (img.preview && img.preview.startsWith("blob:")) {
          URL.revokeObjectURL(img.preview);
        }
      });

      content = "";
      uploadedImages = [];
      isDragging = false;
      dragCounter = 0;
      isLoading = false;

      setTimeout(async () => {
        try {
          await invoke("hide_capture");
        } catch (e) {
          console.error("Hide error:", e);
        }
      }, 200);
    } catch (e) {
      console.error("Append to daily note failed:", e);
      showStatus("✗ " + e.toString(), "error");
      isLoading = false;
    }
  }

  async function handleClose() {
    uploadedImages.forEach((img) => {
      if (img.preview && img.preview.startsWith("blob:")) {
        URL.revokeObjectURL(img.preview);
      }
    });
    content = "";
    uploadedImages = [];
    isDragging = false;
    dragCounter = 0;
    isLoading = false;

    try {
      await invoke("hide_capture");
    } catch (e) {
      console.error("Failed to hide window:", e);
    }
  }

  async function handleSaveAsNote() {
    if (!content.trim() || isLoading) return;

    isLoading = true;

    try {
      const result = await invoke("save_as_note", {
        content: content.trim(),
      });
      showStatus("✓ " + result, "success");

      uploadedImages.forEach((img) => {
        if (img.preview && img.preview.startsWith("blob:")) {
          URL.revokeObjectURL(img.preview);
        }
      });
      content = "";
      uploadedImages = [];
      isDragging = false;
      dragCounter = 0;

      isLoading = false;

      setTimeout(async () => {
        try {
          await invoke("hide_capture");
        } catch (e) {
          console.error("Hide error:", e);
        }
      }, 200);
    } catch (e) {
      console.error("Save as note failed:", e);
      showStatus("✗ " + e.toString(), "error");
      isLoading = false;
    }
  }

  function matchesShortcut(event, shortcutString) {
    if (!shortcutString) return false;

    const parts = shortcutString.split("+").map((p) => p.trim());
    const modifiers = {
      hasCmd: parts.includes("Cmd") || parts.includes("Command"),
      hasCtrl: parts.includes("Ctrl") || parts.includes("Control"),
      hasShift: parts.includes("Shift"),
      hasAlt:
        parts.includes("Alt") ||
        parts.includes("Option") ||
        parts.includes("Opt"),
    };

    const key = parts.find(
      (p) =>
        ![
          "Cmd",
          "Command",
          "Ctrl",
          "Control",
          "Shift",
          "Alt",
          "Option",
          "Opt",
        ].includes(p),
    );

    if (!key) return false;

    const modifiersMatch =
      (event.metaKey === modifiers.hasCmd ||
        event.ctrlKey === modifiers.hasCmd) &&
      event.ctrlKey === modifiers.hasCtrl &&
      event.shiftKey === modifiers.hasShift &&
      event.altKey === modifiers.hasAlt;

    const keyMatches = event.key.toLowerCase() === key.toLowerCase();

    return modifiersMatch && keyMatches;
  }

  function handleKeydown(e) {
    if (e.metaKey && e.key === ",") {
      e.preventDefault();
      openSettings();
      return;
    }

    if (matchesShortcut(e, appSettings.save_to_daily_shortcut)) {
      e.preventDefault();
      handleAppendToDaily();
      return;
    }

    if (matchesShortcut(e, appSettings.save_as_note_shortcut)) {
      e.preventDefault();
      handleSaveAsNote();
      return;
    }

    if (e.key === "Escape") {
      e.preventDefault();
      handleClose();
      return;
    }
  }

  async function openSettings() {
    try {
      await invoke("open_settings");
    } catch (e) {
      console.error("Failed to open settings:", e);
    }
  }

  let dragCounter = 0;

  function handleDragEnter(e) {
    e.preventDefault();

    if (isFileDrag(e)) {
      dragCounter++;
      if (dragCounter === 1) {
        isDragging = true;
      }
    }
  }

  function handleDragLeave(e) {
    e.preventDefault();

    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX;
    const y = e.clientY;
    const isLeaving =
      x < rect.left || x > rect.right || y < rect.top || y > rect.bottom;

    if (isLeaving) {
      dragCounter--;
      if (dragCounter <= 0) {
        dragCounter = 0;
        isDragging = false;
      }
    }
  }

  function handleDragOver(e) {
    e.preventDefault();
    if (isFileDrag(e)) {
      e.dataTransfer.dropEffect = "copy";
      if (!isDragging) {
        isDragging = true;
      }
    } else {
      if (e.dataTransfer) {
        e.dataTransfer.dropEffect = "none";
      }
    }
  }

  async function handleDrop(e) {
    e.preventDefault();
    e.stopPropagation();
    isDragging = false;
    dragCounter = 0;

    if (!e.dataTransfer) {
      showStatus("Error: dataTransfer not available", "error");
      return;
    }

    const items = Array.from(e.dataTransfer.items || []);
    const files = Array.from(e.dataTransfer.files || []);

    if (files.length === 0) {
      showStatus("No files found", "error");
      return;
    }

    const promises = files.map(async (file, index) => {
      const ext = file.name.split(".").pop()?.toLowerCase();
      if (!["png", "jpg", "jpeg", "webp", "gif"].includes(ext)) {
        showStatus("Nicht unterstützt: " + file.name, "error");
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
              filePath = fileFromItem.path || fileFromItem.webkitRelativePath;
            }
          }
        }

        let markdownLink;

        if (filePath) {
          const result = await invoke("save_image", {
            filePath: filePath,
          });
          markdownLink = normalizeImageResult(result).markdown;
        } else {
          if (!isTauri) {
            showStatus("Please use in Tauri app", "error");
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
            throw new Error("Failed to encode file: " + base64Error.toString());
          }

          try {
            if (typeof invoke === "undefined") {
              throw new Error("invoke is undefined - not running in Tauri");
            }
            const result = await invoke("save_image_from_bytes", {
              bytesBase64: base64,
              filename: file.name,
            });
            markdownLink = normalizeImageResult(result).markdown;
          } catch (invokeError) {
            console.error("Invoke error:", invokeError);
            throw invokeError;
          }
        }

        const previewUrl = URL.createObjectURL(file);

        return {
          id: Date.now() + Math.random() + index,
          filename: file.name,
          markdown: markdownLink,
          preview: previewUrl,
          file: file,
        };
      } catch (e) {
        console.error("Error processing file:", e);
        showStatus("Error: " + e.toString(), "error");
        return null;
      }
    });

    const results = await Promise.all(promises);
    const validImages = results.filter((img) => img !== null);

    if (validImages.length > 0) {
      let insertPosition = 0;
      if (textareaRef) {
        const textareaRect = textareaRef.getBoundingClientRect();
        const dropX = e.clientX;
        const dropY = e.clientY;
        const isOverTextarea =
          dropX >= textareaRect.left &&
          dropX <= textareaRect.right &&
          dropY >= textareaRect.top &&
          dropY <= textareaRect.bottom;

        if (
          isOverTextarea ||
          e.target === textareaRef ||
          e.target.closest("textarea") === textareaRef
        ) {
          if (
            textareaRef.selectionStart !== null &&
            textareaRef.selectionStart !== undefined
          ) {
            insertPosition = textareaRef.selectionStart;
          } else {
            const lineHeight =
              parseInt(getComputedStyle(textareaRef).lineHeight) || 20;
            const scrollTop = textareaRef.scrollTop;
            const relativeY = dropY - textareaRect.top + scrollTop;
            const estimatedLine = Math.max(
              0,
              Math.floor(relativeY / lineHeight),
            );

            const lines = content.split("\n");
            let pos = 0;
            for (let i = 0; i < Math.min(estimatedLine, lines.length); i++) {
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
      validImages.forEach((img) => {
        const imageMarkdown = img.markdown + "\n";

        if (currentPosition >= content.length) {
          if (content.length > 0 && !content.endsWith("\n")) {
            content += "\n";
            currentPosition = content.length;
          }
          content += imageMarkdown;
          currentPosition += imageMarkdown.length;
        } else {
          const before = content.substring(0, currentPosition);
          const after = content.substring(currentPosition);

          const isAtLineStart = before.endsWith("\n") || before.length === 0;

          if (!isAtLineStart && !before.endsWith("\n\n")) {
            content = before + "\n" + imageMarkdown + after;
            currentPosition += 1 + imageMarkdown.length;
          } else {
            content = before + imageMarkdown + after;
            currentPosition += imageMarkdown.length;
          }
        }
      });

      uploadedImages = [...uploadedImages, ...validImages];

      showStatus(
        `✓ ${validImages.length} image${validImages.length > 1 ? "s" : ""} added`,
        "success",
      );

      if (textareaRef) {
        setTimeout(() => {
          const newPosition = currentPosition;
          textareaRef.setSelectionRange(newPosition, newPosition);
          textareaRef.focus();
        }, 10);
      }
    }
  }

  async function handleTauriFileDrop(paths) {
    isDragging = false;
    dragCounter = 0;

    if (!paths || paths.length === 0) {
      showStatus("No files found", "error");
      return;
    }

    const imagePaths = paths.filter((path) => {
      const ext = path.split(".").pop()?.toLowerCase();
      return ["png", "jpg", "jpeg", "webp", "gif"].includes(ext);
    });

    if (imagePaths.length === 0) {
      showStatus("No supported image files found", "error");
      return;
    }

    const promises = imagePaths.map(async (filePath, index) => {
      try {
        const result = await invoke("save_image", {
          filePath: filePath,
        });
        const normalizedResult = normalizeImageResult(result);

        const normalizedPath = normalizeFilePath(
          normalizedResult.saved_path || filePath,
        );
        const previewUrl = convertFileSrc(normalizedPath);

        return {
          id: Date.now() + Math.random() + index,
          filename:
            normalizedResult.filename ||
            normalizedPath.split("/").pop() ||
            `image${index}`,
          markdown: normalizedResult.markdown,
          preview: previewUrl,
          file: null,
        };
      } catch (e) {
        console.error("Error processing file:", e);
        showStatus("Error: " + e.toString(), "error");
        return null;
      }
    });

    const results = await Promise.all(promises);
    const validImages = results.filter((img) => img !== null);

    if (validImages.length > 0) {
      uploadedImages = [...uploadedImages, ...validImages];
      if (content.length > 0 && !content.endsWith("\n")) {
        content += "\n";
      }
      validImages.forEach((img) => {
        content += img.markdown + "\n";
      });
      showStatus(
        `✓ ${validImages.length} image${validImages.length > 1 ? "s" : ""} added`,
        "success",
      );
    }

    setTimeout(() => textareaRef?.focus(), 50);
  }

  function removeImage(id) {
    const image = uploadedImages.find((img) => img.id === id);
    if (image && image.preview && image.preview.startsWith("blob:")) {
      URL.revokeObjectURL(image.preview);
    }
    uploadedImages = uploadedImages.filter((img) => img.id !== id);
  }
</script>

<div
  class="capture-container"
  class:dragging={isDragging}
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
  on:dragenter={handleDragEnter}
  on:dragleave={handleDragLeave}
  on:dragover={handleDragOver}
  on:drop={handleDrop}
  role="application"
>
  <div class="accent-line" role="presentation"></div>

  <div
    class="content-wrapper"
    role="presentation"
    on:drop={(e) => {
      handleDrop(e);
    }}
    on:dragover={(e) => {
      e.preventDefault();
      handleDragOver(e);
    }}
    on:dragenter={(e) => {
      handleDragEnter(e);
    }}
  >
    {#if uploadedImages.length > 0}
      <div
        class="image-gallery"
        role="presentation"
        on:drop={(e) => {
          handleDrop(e);
        }}
        on:dragover={(e) => {
          e.preventDefault();
          handleDragOver(e);
        }}
        on:dragenter={(e) => {
          handleDragEnter(e);
        }}
      >
        {#each uploadedImages as image (image.id)}
          <div class="image-preview">
            <img src={image.preview} alt={image.filename} />
            <button
              class="remove-btn"
              on:click={() => removeImage(image.id)}
              title="Entfernen"
            >
              ×
            </button>
            <div class="image-name">{image.filename}</div>
          </div>
        {/each}
      </div>
    {/if}

    <div
      class="content-area"
      role="presentation"
      on:drop={(e) => {
        handleDrop(e);
      }}
      on:dragover={(e) => {
        e.preventDefault();
        handleDragOver(e);
      }}
      on:dragenter={(e) => {
        handleDragEnter(e);
      }}
    >
      <textarea
        bind:this={textareaRef}
        bind:value={content}
        on:keydown={handleKeydown}
        on:drop={(e) => {
          e.preventDefault();
          e.stopPropagation();
          handleDrop(e);
        }}
        on:dragover={(e) => {
          e.preventDefault();
          handleDragOver(e);
        }}
        on:dragenter={(e) => {
          handleDragEnter(e);
        }}
        placeholder="Enter note or drag image here..."
        disabled={isLoading}
        spellcheck="false"
      ></textarea>
    </div>
  </div>

  <div class="resize-handle"></div>

  {#if isDragging}
    <div class="drop-overlay">
      <div class="drop-icon">📎</div>
      <span>Drop Image</span>
    </div>
  {/if}

  {#if statusMessage}
    <div class="status-toast" class:error={statusType === "error"}>
      {statusMessage}
    </div>
  {/if}

  {#if isLoading}
    <div class="loading-indicator"></div>
  {/if}
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: transparent;
  }

  .capture-container {
    position: fixed;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) var(--app-transparency, 55%),
      transparent
    );
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
    display: flex;
    flex-direction: column;
    font-family: var(
      --app-font-family,
      -apple-system,
      BlinkMacSystemFont,
      "SF Pro Display",
      sans-serif
    );
    transform: translateZ(0);
    -webkit-transform: translateZ(0);
  }

  .capture-container.dragging {
    background: rgba(255, 255, 255, 0.7);
    border-color: rgba(255, 255, 255, 0.7);
    border-width: 2px;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.08),
      0 2px 8px rgba(0, 0, 0, 0.04);
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

  .content-wrapper {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  .image-gallery {
    display: flex;
    gap: 8px;
    padding: 12px 12px 8px;
    overflow-x: auto;
    overflow-y: hidden;
    flex-shrink: 0;
  }

  .image-gallery::-webkit-scrollbar {
    height: 4px;
  }

  .image-gallery::-webkit-scrollbar-track {
    background: transparent;
  }

  .image-gallery::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.15);
    border-radius: 2px;
  }

  .image-preview {
    position: relative;
    flex-shrink: 0;
    width: 80px;
    height: 80px;
    border-radius: 8px;
    overflow: hidden;
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.08);
    transition: all 0.2s;
  }

  .image-preview:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .image-preview img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .remove-btn {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 59, 48, 0.9);
    color: white;
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s;
    padding: 0;
  }

  .image-preview:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    background: rgba(255, 59, 48, 1);
    transform: scale(1.1);
  }

  .image-name {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 4px;
    background: rgba(0, 0, 0, 0.7);
    color: white;
    font-size: 9px;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .image-preview:hover .image-name {
    opacity: 1;
  }

  .content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 12px 16px 16px;
    overflow: hidden;
  }

  textarea {
    flex: 1;
    width: 100%;
    border: none;
    outline: none;
    resize: none;
    background: transparent;
    font-family: inherit;
    font-size: var(--app-font-size, 15px);
    line-height: 1.6;
    color: var(--app-text-color, #ffffff);
    caret-color: #8b5cf6;
  }

  textarea::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }

  textarea:disabled {
    opacity: 0.6;
  }

  .resize-handle {
    position: absolute;
    bottom: 4px;
    right: 4px;
    width: 16px;
    height: 16px;
    cursor: nwse-resize;
    opacity: 0.3;
    transition: opacity 0.2s;
  }

  .resize-handle::before {
    content: "";
    position: absolute;
    bottom: 0;
    right: 0;
    width: 12px;
    height: 12px;
    background: linear-gradient(
        135deg,
        transparent 40%,
        rgba(0, 0, 0, 0.2) 40%,
        rgba(0, 0, 0, 0.2) 45%,
        transparent 45%
      ),
      linear-gradient(
        135deg,
        transparent 50%,
        rgba(0, 0, 0, 0.2) 50%,
        rgba(0, 0, 0, 0.2) 55%,
        transparent 55%
      );
    border-radius: 0 0 14px 0;
  }

  .capture-container:hover .resize-handle {
    opacity: 0.6;
  }

  .drop-overlay {
    position: absolute;
    inset: 2px;
    background: none;
    border: 2px dashed rgba(255, 255, 255, 0.7);
    border: 2px rgba(255, 255, 255, 0.7);
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    pointer-events: none;
  }

  .drop-icon {
    font-size: 48px;
    opacity: 0.8;
  }

  .drop-overlay span {
    font-size: 15px;
    color: rgba(255, 255, 255, 0.9);
    font-weight: 600;
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

  .loading-indicator {
    position: absolute;
    top: 2px;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(
      90deg,
      transparent,
      rgba(139, 92, 246, 0.8),
      transparent
    );
    background-size: 200% 100%;
    animation: loading 1s ease-in-out infinite;
    z-index: 100;
  }

  @keyframes loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }

  textarea::-webkit-scrollbar {
    width: 6px;
  }
  textarea::-webkit-scrollbar-track {
    background: transparent;
  }
  textarea::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.12);
    border-radius: 3px;
  }
</style>
