<script>
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import {
        getReaderIconComponent,
        readerIconOptions,
    } from "./lib/reader-icons.js";
    import { getSystemFonts } from "./lib/utils.js";
    import { defaultSettings } from "./lib/stores.js";

    let settings = { ...defaultSettings };
    let originalSettings = { ...defaultSettings };
    let isSaving = false;
    let statusMessage = "";
    let statusType = "";
    let showTemplateEditor = false;
    let showAppPicker = false;
    let runningApps = [];
    let filteredRunningApps = [];
    let appPickerQuery = "";
    let activePanel = "obsidian";
    let isIndexing = false;
    let indexStatus = "";

    const systemFonts = getSystemFonts();
    const settingsPanels = [
        {
            id: "obsidian",
            label: "Obsidian Integration",
            description: "Vault, daily notes and capture defaults",
        },
        {
            id: "images",
            label: "Images",
            description: "Screenshot folder and image defaults",
        },
        {
            id: "look",
            label: "Look",
            description: "Window appearance and typography",
        },
        {
            id: "note-window",
            label: "Note Window",
            description: "Defaults for newly created notes",
        },
        {
            id: "reader-window",
            label: "Reader Window",
            description: "Pinned notes and reader filters",
        },
        {
            id: "activation",
            label: "Activation",
            description: "Edge trigger timing, modifiers, and exclusions",
        },
        {
            id: "shortcuts",
            label: "Shortcuts",
            description: "Keyboard shortcuts for both windows",
        },
    ];

    function normalizePinnedNotes(pinnedNotes = []) {
        return pinnedNotes
            .map((entry) => {
                if (typeof entry === "string") {
                    return {
                        path: entry,
                        label: getFilename(entry),
                        icon: "",
                    };
                }

                return {
                    path: entry?.path ?? "",
                    label: entry?.label ?? "",
                    icon: entry?.icon ?? "",
                };
            })
            .filter((entry) => entry.path.trim() !== "");
    }

    async function loadSettings() {
        try {
            const loaded = await invoke("load_settings");
            const normalized = {
                ...loaded,
                pinned_notes: normalizePinnedNotes(loaded.pinned_notes),
            };
            settings = normalized;
            originalSettings = { ...normalized };
        } catch (e) {
            console.error("Failed to load settings:", e);
            showStatus("Failed to load settings", "error");
        }
    }

    onMount(async () => {
        await loadSettings();
    });

    function showStatus(message, type = "success") {
        statusMessage = message;
        statusType = type;
        setTimeout(() => {
            statusMessage = "";
            statusType = "";
        }, 3000);
    }

    async function pickVaultPath() {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: settings.vault_path || undefined,
        });
        if (selected) {
            settings.vault_path = selected;
            indexStatus = "";
        }
    }

    async function handleReindex() {
        isIndexing = true;
        indexStatus = "";

        try {
            const count = await invoke("reindex_vault");
            indexStatus = `✓ Index built (${count} files)`;
        } catch (e) {
            indexStatus = `Failed to index vault: ${e}`;
        } finally {
            isIndexing = false;
        }
    }

    async function pickScreenshotPath() {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath:
                resolveVaultSettingPath(settings.screenshot_path) ||
                settings.vault_path ||
                undefined,
        });
        if (selected) {
            const relative = toRelativeVaultDirectoryPath(selected);
            if (!relative) {
                showStatus(
                    "Image folder must be inside the current vault",
                    "error",
                );
                return;
            }

            settings.screenshot_path = relative;
            settings = { ...settings };
        }
    }

    async function addPinnedNotes() {
        const selected = await open({
            multiple: true,
            filters: [{ name: "Markdown", extensions: ["md"] }],
            defaultPath: settings.vault_path || undefined,
        });

        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];
            const existing = normalizePinnedNotes(settings.pinned_notes);
            const existingByPath = new Set(existing.map((note) => note.path));
            let skippedOutsideVault = false;
            const additions = paths
                .map((path) => toRelativeVaultPath(path))
                .filter((path) => {
                    if (!path) {
                        skippedOutsideVault = true;
                        return false;
                    }

                    return !existingByPath.has(path);
                })
                .map((path) => ({
                    path,
                    label: getFilename(path),
                    icon: "",
                }));

            settings.pinned_notes = [...existing, ...additions];
            settings = { ...settings };

            if (skippedOutsideVault) {
                showStatus(
                    "Pinned notes must be inside the current vault",
                    "error",
                );
            }
        }
    }

    function removePinnedNote(pathToRemove) {
        settings.pinned_notes = normalizePinnedNotes(
            settings.pinned_notes,
        ).filter((note) => note.path !== pathToRemove);
        settings = { ...settings };
    }

    function updatePinnedNote(pathToUpdate, field, value) {
        settings.pinned_notes = normalizePinnedNotes(settings.pinned_notes).map(
            (note) =>
                note.path === pathToUpdate
                    ? {
                          ...note,
                          [field]: value,
                      }
                    : note,
        );
        settings = { ...settings };
    }

    function getFilename(path) {
        return (
            path.replace(/\\/g, "/").split("/").pop()?.replace(/\.md$/i, "") ||
            path
        );
    }

    function normalizeComparablePath(path = "") {
        return path.replace(/\\/g, "/").replace(/\/+$/, "");
    }

    function toRelativeVaultPath(path = "") {
        const normalizedPath = normalizeComparablePath(path.trim());
        const normalizedVaultPath = normalizeComparablePath(
            settings.vault_path ?? "",
        );

        if (!normalizedPath || !normalizedVaultPath) {
            return "";
        }

        if (normalizedPath === normalizedVaultPath) {
            return "";
        }

        if (normalizedPath.startsWith(`${normalizedVaultPath}/`)) {
            return normalizedPath.slice(normalizedVaultPath.length + 1);
        }

        return "";
    }

    function toRelativeVaultDirectoryPath(path = "") {
        const normalizedPath = normalizeComparablePath(path.trim());
        const normalizedVaultPath = normalizeComparablePath(
            settings.vault_path ?? "",
        );

        if (!normalizedPath || !normalizedVaultPath) {
            return "";
        }

        if (normalizedPath === normalizedVaultPath) {
            return ".";
        }

        if (normalizedPath.startsWith(`${normalizedVaultPath}/`)) {
            return normalizedPath.slice(normalizedVaultPath.length + 1);
        }

        return "";
    }

    function resolveVaultSettingPath(path = "") {
        const rawPath = path.trim();
        const normalizedVaultPath = normalizeComparablePath(
            settings.vault_path ?? "",
        );

        if (!rawPath) {
            return normalizedVaultPath || "";
        }

        if (
            rawPath.startsWith("/") ||
            /^[A-Za-z]:[\\/]/.test(rawPath)
        ) {
            return rawPath;
        }

        if (!normalizedVaultPath) {
            return rawPath;
        }

        if (rawPath === ".") {
            return normalizedVaultPath;
        }

        return `${normalizedVaultPath}/${normalizeComparablePath(rawPath)}`;
    }

    function modifierLabel(mod) {
        return (
            {
                cmd: "⌘ Cmd",
                option: "⌥ Option",
                shift: "⇧ Shift",
                ctrl: "⌃ Ctrl",
            }[mod] ?? mod
        );
    }

    function filterApps(apps, query) {
        if (!query) return apps;
        const lower = query.toLowerCase();
        return apps.filter((app) => app.toLowerCase().includes(lower));
    }

    function refreshFilteredApps() {
        filteredRunningApps = filterApps(runningApps, appPickerQuery);
    }

    async function toggleAppPicker() {
        showAppPicker = !showAppPicker;
        if (!showAppPicker) return;

        if (runningApps.length === 0) {
            try {
                runningApps = await invoke("get_running_apps");
            } catch (error) {
                console.error("Could not get running apps:", error);
                showStatus("Failed to load running apps", "error");
                return;
            }
        }

        refreshFilteredApps();
    }

    function addExcludedApp(app) {
        const current = settings.edge_excluded_apps ?? [];
        if (current.includes(app)) return;
        settings.edge_excluded_apps = [...current, app];
        settings = { ...settings };
    }

    function removeExcludedApp(app) {
        settings.edge_excluded_apps = (
            settings.edge_excluded_apps ?? []
        ).filter((entry) => entry !== app);
        settings = { ...settings };
    }

    function normalizeDelayValue(value, fallback = 1000) {
        const parsed = Number(value);

        if (!Number.isFinite(parsed)) {
            return fallback;
        }

        return Math.min(10000, Math.max(50, Math.round(parsed)));
    }

    function normalizeDelayField(field, fallback = 1000) {
        settings[field] = normalizeDelayValue(settings[field], fallback);
        settings = { ...settings };
    }

    async function handleSave() {
        isSaving = true;

        try {
            const payload = {
                ...settings,
                pinned_notes: normalizePinnedNotes(settings.pinned_notes),
                note_edge_open_delay_ms: normalizeDelayValue(
                    settings.note_edge_open_delay_ms,
                ),
                reader_edge_open_delay_ms: normalizeDelayValue(
                    settings.reader_edge_open_delay_ms,
                ),
            };

            await invoke("save_settings", { newSettings: payload });

            settings = { ...payload };
            originalSettings = { ...payload };

            showStatus("✓ Settings saved", "success");
        } catch (e) {
            console.error("Failed to save settings:", e);
            console.error("Error details:", JSON.stringify(e));
            showStatus("Failed to save settings: " + e.toString(), "error");
        } finally {
            isSaving = false;
            settings = { ...settings };
        }
    }

    async function handleCancel() {
        settings = { ...originalSettings };

        statusMessage = "";
        statusType = "";

        try {
            await invoke("close_settings");
        } catch (e) {
            console.error("Failed to close settings window:", e);
        }
    }

    function handleReset() {
        if (confirm("Reset all settings to default?")) {
            settings = { ...defaultSettings };
        }
    }

    function handleShortcutKeyDown(event, field) {
        event.preventDefault();

        if (
            ["Control", "Shift", "Alt", "Meta", "Command"].includes(event.key)
        ) {
            return;
        }

        const modifiers = [];
        if (event.metaKey) modifiers.push("Cmd");
        if (event.ctrlKey) modifiers.push("Ctrl");
        if (event.shiftKey) modifiers.push("Shift");
        if (event.altKey) modifiers.push("Alt");

        let key = event.code;

        const keyMap = {
            Space: "Space",
            Escape: "Escape",
            Enter: "Enter",
            Backspace: "Backspace",
            Delete: "Delete",
            ArrowUp: "Up",
            ArrowDown: "Down",
            ArrowLeft: "Left",
            ArrowRight: "Right",
            Tab: "Tab",
            Home: "Home",
            End: "End",
            PageUp: "PageUp",
            PageDown: "PageDown",
        };

        if (keyMap[key]) {
            key = keyMap[key];
        } else if (key.startsWith("Key")) {
            key = key.substring(3).toUpperCase();
        } else if (key.startsWith("Digit")) {
            key = key.substring(5);
        } else if (key.startsWith("F") && key.length <= 3) {
            key = key;
        } else {
            return;
        }

        if (modifiers.length === 0) {
            return;
        }

        const shortcut = [...modifiers, key].join("+");
        settings[field] = shortcut;
    }

    $: filteredRunningApps = filterApps(runningApps, appPickerQuery);
    $: hasChanges =
        JSON.stringify(settings) !== JSON.stringify(originalSettings);
</script>

<div class="settings-container">
    <header>
        <h1>Settings</h1>
        {#if statusMessage}
            <div class="status {statusType}">{statusMessage}</div>
        {/if}
    </header>

    <main>
        <div class="settings-layout">
            <aside class="settings-sidebar">
                <nav class="settings-nav" aria-label="Settings sections">
                    {#each settingsPanels as panel}
                        <button
                            class="nav-item"
                            class:active={activePanel === panel.id}
                            type="button"
                            on:click={() => (activePanel = panel.id)}
                        >
                            <span class="nav-item-label">{panel.label}</span>
                            <span class="nav-item-description"
                                >{panel.description}</span
                            >
                        </button>
                    {/each}
                </nav>
            </aside>

            <div class="settings-content">
                {#if activePanel === "obsidian"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>Obsidian Integration</h2>
                            <p class="section-description">
                                Configure your vault, daily note location and
                                the default header Collector inserts into
                                captures.
                            </p>
                        </section>

                        <section>
                            <h2>Vault</h2>
                            <p class="section-description">
                                Choose the Obsidian vault Collector works
                                against and refresh the index when files change
                                outside the app.
                            </p>

                            <div class="field">
                                <label for="vault_name">Vault Name</label>
                                <input
                                    type="text"
                                    id="vault_name"
                                    bind:value={settings.vault_name}
                                    placeholder="Vault"
                                />
                                <small>Name of your Obsidian vault</small>
                            </div>
                            <div class="field">
                                <label for="vault_path">Vault Path</label>
                                <div class="path-picker">
                                    <input
                                        type="text"
                                        id="vault_path"
                                        bind:value={settings.vault_path}
                                        placeholder="/Users/username/Vault"
                                        readonly
                                    />
                                    <button
                                        class="secondary"
                                        on:click={pickVaultPath}
                                        >Choose...</button
                                    >
                                </div>
                                <div class="reindex-row">
                                    <button
                                        class="secondary"
                                        type="button"
                                        on:click={handleReindex}
                                        disabled={isIndexing}
                                    >
                                        {isIndexing
                                            ? "Indexing..."
                                            : "↻ Re-index Vault"}
                                    </button>
                                    {#if indexStatus}
                                        <span class="index-status"
                                            >{indexStatus}</span
                                        >
                                    {/if}
                                </div>
                                <small>Full path to your Obsidian vault</small>
                                <small>
                                    Re-index if images or notes added outside
                                    Collector are not appearing correctly.
                                </small>
                            </div>
                        </section>

                        <section>
                            <h2>Daily Notes</h2>
                            <p class="section-description">
                                Define where daily notes live and how their
                                filenames are generated.
                            </p>

                            <div class="field">
                                <label for="daily_note_folder"
                                    >Daily Note Path</label
                                >
                                <input
                                    type="text"
                                    id="daily_note_folder"
                                    bind:value={settings.daily_note_folder}
                                    placeholder="Journal/Notes/"
                                />
                                <small
                                    >Relative path in vault for daily notes</small
                                >
                            </div>
                            <div class="field">
                                <label for="daily_note_format"
                                    >Daily Note Format</label
                                >
                                <input
                                    type="text"
                                    id="daily_note_format"
                                    bind:value={settings.daily_note_format}
                                    placeholder="YYYY-MM-DD"
                                />
                                <small
                                    >Filename format (e.g. YYYY-MM-DD).
                                    Supports: YYYY, MM, DD</small
                                >
                            </div>
                        </section>

                        <section>
                            <h2>Entry Header</h2>
                            <p class="section-description">
                                This Markdown heading is inserted before each
                                saved capture.
                            </p>

                            <div class="field">
                                <label for="entry_header">Entry Header</label>
                                <input
                                    type="text"
                                    id="entry_header"
                                    bind:value={settings.entry_header}
                                    placeholder="#### HH:mm"
                                />
                                <small>
                                    Supported: HH (24h), hh / h (12h), mm, ss, a / A (am/pm) · e.g. #### HH:mm or #### h:mm a
                                </small>
                            </div>
                        </section>
                    </div>
                {:else if activePanel === "images"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>Images</h2>
                            <p class="section-description">
                                Control where screenshots are stored and how new
                                image embeds are created.
                            </p>
                        </section>

                        <section>
                            <h2>Storage</h2>
                            <p class="section-description">
                                Set the destination folder and filename pattern
                                for captured images.
                            </p>

                            <div class="field">
                                <label for="screenshot_path">Image Folder</label
                                >
                                <div class="path-picker">
                                    <input
                                        type="text"
                                        id="screenshot_path"
                                        bind:value={settings.screenshot_path}
                                        placeholder="Grafiken/Screenshots"
                                    />
                                    <button
                                        class="secondary"
                                        on:click={pickScreenshotPath}
                                        >Choose...</button
                                    >
                                </div>
                                <small
                                    >Relative path in the vault for saved
                                    images (folder will be created
                                    automatically)</small
                                >
                            </div>
                            <div class="field">
                                <label for="image_filename"
                                    >Filename Template</label
                                >
                                <input
                                    type="text"
                                    id="image_filename"
                                    bind:value={settings.image_filename}
                                    placeholder="screenshot-YYYY-MM-DD-HHmmss"
                                />
                                <small>Supports: YYYY, MM, DD, HH, mm, ss</small
                                >
                            </div>
                        </section>

                        <section>
                            <h2>Embed Defaults</h2>
                            <p class="section-description">
                                Tune compression and the default display width
                                for new image links.
                            </p>

                            <div class="field">
                                <label for="compression_max_kb"
                                    >Max. Image Size (KB)</label
                                >
                                <input
                                    type="number"
                                    id="compression_max_kb"
                                    bind:value={settings.compression_max_kb}
                                    min="50"
                                    max="1000"
                                    step="50"
                                />
                                <small
                                    >Images will be compressed to this size</small
                                >
                            </div>
                            <div class="field">
                                <label for="default_image_width"
                                    >Default Image Width</label
                                >
                                <input
                                    type="text"
                                    id="default_image_width"
                                    bind:value={settings.default_image_width}
                                    placeholder="600"
                                    inputmode="numeric"
                                />
                                <small
                                    >Optional width in pixels for new image
                                    links (leave empty for no width)</small
                                >
                            </div>
                        </section>
                    </div>
                {:else if activePanel === "look"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>Look & Feel</h2>
                            <p class="section-description">
                                Shared appearance settings for the capture and
                                reader windows.
                            </p>
                        </section>

                        <section>
                            <h2>Window Surface</h2>
                            <p class="section-description">
                                Shape the glass surface and background treatment
                                for both floating windows.
                            </p>

                            <div class="field">
                                <label for="border_radius"
                                    >Corner Radius:
                                    {settings.border_radius}px</label
                                >
                                <input
                                    type="range"
                                    id="border_radius"
                                    bind:value={settings.border_radius}
                                    min="0"
                                    max="12"
                                />
                            </div>
                            <div class="field">
                                <label for="background_color"
                                    >Background Color</label
                                >
                                <div class="color-input">
                                    <input
                                        type="color"
                                        id="background_color"
                                        bind:value={settings.background_color}
                                    />
                                    <input
                                        type="text"
                                        bind:value={settings.background_color}
                                        pattern="^#[0-9A-Fa-f]{6}$"
                                    />
                                </div>
                            </div>
                            <div class="field">
                                <label for="window_transparency"
                                    >Transparency:
                                    {settings.window_transparency ?? 55}%</label
                                >
                                <input
                                    type="range"
                                    id="window_transparency"
                                    bind:value={settings.window_transparency}
                                    min="0"
                                    max="100"
                                />
                                <small>Transparency of window background</small>
                            </div>
                            <div class="field">
                                <label for="window_blur"
                                    >Blur:
                                    {settings.window_blur ?? 80}px</label
                                >
                                <input
                                    type="range"
                                    id="window_blur"
                                    bind:value={settings.window_blur}
                                    min="0"
                                    max="200"
                                />
                                <small>Background blur effect</small>
                            </div>
                            <div class="field">
                                <label for="window_saturation"
                                    >Saturation:
                                    {settings.window_saturation ?? 200}%</label
                                >
                                <input
                                    type="range"
                                    id="window_saturation"
                                    bind:value={settings.window_saturation}
                                    min="0"
                                    max="300"
                                />
                                <small>Background color intensity</small>
                            </div>
                            <div class="field">
                                <label for="window_brightness"
                                    >Brightness: {(settings.window_brightness ??
                                    0 > 0)
                                        ? ""
                                        : ""}{settings.window_brightness ??
                                        0}</label
                                >
                                <input
                                    type="range"
                                    id="window_brightness"
                                    bind:value={settings.window_brightness}
                                    min="-100"
                                    max="100"
                                />
                                <small
                                    >Brightens dark areas or darkens light areas</small
                                >
                            </div>
                        </section>

                        <section>
                            <h2>Accent & Links</h2>
                            <p class="section-description">
                                Choose the highlight color and how links are
                                rendered in both windows.
                            </p>

                            <div class="field">
                                <label for="accent_color">Accent Color</label>
                                <div class="color-input">
                                    <input
                                        type="color"
                                        id="accent_color"
                                        bind:value={settings.accent_color}
                                    />
                                    <input
                                        type="text"
                                        bind:value={settings.accent_color}
                                        pattern="^#[0-9A-Fa-f]{6}$"
                                    />
                                </div>
                                <small
                                    >Used for active tabs, checkboxes, caret,
                                    and highlights</small
                                >
                            </div>
                            <div class="field">
                                <label for="internal_link_color"
                                    >Internal Links [[wikilinks]]</label
                                >
                                <div class="color-input">
                                    <input
                                        type="color"
                                        id="internal_link_color"
                                        bind:value={
                                            settings.internal_link_color
                                        }
                                    />
                                    <input
                                        type="text"
                                        bind:value={
                                            settings.internal_link_color
                                        }
                                        pattern="^#[0-9A-Fa-f]{6}$"
                                    />
                                </div>
                            </div>
                            <div class="field">
                                <label for="external_link_color"
                                    >External Links [text](url)</label
                                >
                                <div class="color-input">
                                    <input
                                        type="color"
                                        id="external_link_color"
                                        bind:value={
                                            settings.external_link_color
                                        }
                                    />
                                    <input
                                        type="text"
                                        bind:value={
                                            settings.external_link_color
                                        }
                                        pattern="^#[0-9A-Fa-f]{6}$"
                                    />
                                </div>
                            </div>
                        </section>

                        <section>
                            <h2>Typography</h2>
                            <p class="section-description">
                                Set the font family, size and default text color
                                for both windows.
                            </p>

                            <div class="field">
                                <label for="font_family">Font Family</label>
                                <select
                                    id="font_family"
                                    bind:value={settings.font_family}
                                >
                                    {#each systemFonts as font}
                                        <option value={font}>{font}</option>
                                    {/each}
                                </select>
                            </div>
                            <div class="field">
                                <label for="font_size"
                                    >Font Size:
                                    {settings.font_size}px</label
                                >
                                <input
                                    type="range"
                                    id="font_size"
                                    bind:value={settings.font_size}
                                    min="10"
                                    max="20"
                                />
                            </div>
                            <div class="field">
                                <label for="text_color">Text Color</label>
                                <div class="color-input">
                                    <input
                                        type="color"
                                        id="text_color"
                                        bind:value={settings.text_color}
                                    />
                                    <input
                                        type="text"
                                        bind:value={settings.text_color}
                                        pattern="^#[0-9A-Fa-f]{6}$"
                                    />
                                </div>
                                <small>Default: White (#ffffff)</small>
                            </div>
                        </section>
                    </div>
                {:else if activePanel === "note-window"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>New Notes</h2>
                            <p class="section-description">
                                Defaults used when Collector creates a new note
                                file.
                            </p>
                        </section>

                        <section>
                            <h2>Window Size</h2>
                            <p class="section-description">
                                Control the default size of the floating note
                                window.
                            </p>

                            <div class="field-row">
                                <div class="field">
                                    <label for="window_width">Width (px)</label>
                                    <input
                                        type="number"
                                        id="window_width"
                                        bind:value={settings.window_width}
                                        min="200"
                                        max="800"
                                    />
                                </div>
                                <div class="field">
                                    <label for="window_height"
                                        >Height (px)</label
                                    >
                                    <input
                                        type="number"
                                        id="window_height"
                                        bind:value={settings.window_height}
                                        min="200"
                                        max="1200"
                                    />
                                </div>
                            </div>
                        </section>

                        <section>
                            <h2>New Note Defaults</h2>
                            <p class="section-description">
                                Define where new notes are created and how their
                                filenames are generated.
                            </p>

                            <div class="field">
                                <label for="notes_folder">Notes Folder</label>
                                <input
                                    type="text"
                                    id="notes_folder"
                                    bind:value={settings.notes_folder}
                                    placeholder="Notes/"
                                />
                                <small
                                    >Relative path in vault for new notes</small
                                >
                            </div>
                            <div class="field">
                                <label for="note_filename_template"
                                    >Note Filename Template</label
                                >
                                <input
                                    type="text"
                                    id="note_filename_template"
                                    bind:value={settings.note_filename_template}
                                    placeholder="note-YYYY-MM-DD-HHmmss"
                                />
                                <small
                                    >Supported: YYYY, MM, DD, HH (24h), hh / h (12h), mm, ss, A / a (am/pm)</small
                                >
                            </div>
                        </section>

                        <section>
                            <h2>Template</h2>
                            <p class="section-description">
                                Optional boilerplate inserted at the beginning
                                of each newly created note.
                            </p>

                            <div class="field">
                                <label for="note_template">Template Text</label>
                                <button
                                    class="secondary"
                                    on:click={() =>
                                        (showTemplateEditor =
                                            !showTemplateEditor)}
                                >
                                    {showTemplateEditor
                                        ? "Hide Template"
                                        : "Edit Template"}
                                </button>
                                {#if showTemplateEditor}
                                    <textarea
                                        id="note_template"
                                        bind:value={settings.note_template}
                                        placeholder="---&#10;created: <% tp.date.now(&quot;YYYY-MM-DD hh:mm&quot;) %>&#10;modified: &#10;daily: &quot;[[<% tp.date.now(&quot;YYYY-MM-DD&quot;) %>]]&quot;&#10;tags: inbox&#10;type: inbox&#10;---"
                                        rows="8"
                                        class="template-editor"
                                    />
                                    <small
                                        >This text will be inserted at the
                                        beginning of each new note (e.g. for
                                        frontmatter/properties)</small
                                    >
                                {/if}
                            </div>
                        </section>
                    </div>
                {:else if activePanel === "reader-window"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>Reader Panel</h2>
                            <p class="section-description">
                                Configure the left reader window and the tabs
                                shown there by default.
                            </p>
                        </section>

                        <section>
                            <h2>Window Size</h2>
                            <p class="section-description">
                                Control the default size of the floating reader
                                window.
                            </p>

                            <div class="field-row">
                                <div class="field">
                                    <label for="reader_width">Width (px)</label>
                                    <input
                                        type="number"
                                        id="reader_width"
                                        bind:value={settings.reader_width}
                                        min="200"
                                        max="800"
                                    />
                                </div>
                                <div class="field">
                                    <label for="reader_height"
                                        >Height (px)</label
                                    >
                                    <input
                                        type="number"
                                        id="reader_height"
                                        bind:value={settings.reader_height}
                                        min="200"
                                        max="1200"
                                    />
                                </div>
                            </div>
                        </section>

                        <section>
                            <h2>Pinned Notes</h2>
                            <p class="section-description">
                                These notes appear as tabs in the left reader
                                panel. The Daily Note is always included
                                automatically and paths are stored relative to
                                the current vault.
                            </p>

                            {#if normalizePinnedNotes(settings.pinned_notes).length > 0}
                                <div class="note-list">
                                    {#each normalizePinnedNotes(settings.pinned_notes) as note}
                                        <div class="note-list-item">
                                            <div class="note-list-copy">
                                                <div class="note-list-editors">
                                                    <input
                                                        class="pinned-note-label"
                                                        type="text"
                                                        value={note.label}
                                                        placeholder={getFilename(
                                                            note.path,
                                                        )}
                                                        on:input={(event) =>
                                                            updatePinnedNote(
                                                                note.path,
                                                                "label",
                                                                event
                                                                    .currentTarget
                                                                    .value,
                                                            )}
                                                    />
                                                </div>
                                                <div class="pinned-note-icons">
                                                    {#each readerIconOptions as iconOption}
                                                        <button
                                                            class="icon-choice"
                                                            class:selected={note.icon ===
                                                                iconOption.id}
                                                            type="button"
                                                            title={iconOption.label}
                                                            aria-label={iconOption.label}
                                                            on:click={() =>
                                                                updatePinnedNote(
                                                                    note.path,
                                                                    "icon",
                                                                    iconOption.id,
                                                                )}
                                                        >
                                                            {#if iconOption.component}
                                                                <svelte:component
                                                                    this={getReaderIconComponent(
                                                                        iconOption.id,
                                                                    )}
                                                                    size={14}
                                                                    strokeWidth={1.9}
                                                                />
                                                            {:else}
                                                                <span
                                                                    class="icon-choice-none"
                                                                    >-</span
                                                                >
                                                            {/if}
                                                        </button>
                                                    {/each}
                                                </div>
                                                <small>{note.path}</small>
                                            </div>
                                            <button
                                                class="remove-note"
                                                type="button"
                                                on:click={() =>
                                                    removePinnedNote(note.path)}
                                            >
                                                ✕
                                            </button>
                                        </div>
                                    {/each}
                                </div>
                            {:else}
                                <div class="empty-note-list">
                                    No pinned notes selected.
                                </div>
                            {/if}

                            <small>
                                Optional icon and label only affect the reader
                                tab display, not the actual Markdown file.
                            </small>
                            <button
                                class="secondary"
                                type="button"
                                on:click={addPinnedNotes}>+ Add Note</button
                            >
                        </section>

                        <section>
                            <h2>Content Filters</h2>
                            <p class="section-description">
                                Choose which elements are hidden in the reader.
                                They are never deleted, only hidden from view.
                            </p>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.reader_hide_frontmatter
                                        }
                                    />
                                    Hide YAML Frontmatter
                                </label>
                                <small
                                    >Hides the --- metadata block at the top of
                                    notes</small
                                >
                            </div>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.reader_hide_dataview
                                        }
                                    />
                                    Hide Code Blocks (Dataview, JS, etc.)
                                </label>
                                <small
                                    >Hides all ```language ... ``` code blocks</small
                                >
                            </div>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.reader_hide_obsidian_comments
                                        }
                                    />
                                    Hide Obsidian Comments
                                </label>
                                <small>Hides %% ... %% comment blocks</small>
                            </div>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.reader_hide_inline_fields
                                        }
                                    />
                                    Hide Inline Fields (Dataview)
                                </label>
                                <small
                                    >Hides key:: value fields from note text</small
                                >
                            </div>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={settings.reader_hide_html}
                                    />
                                    Hide HTML Blocks
                                </label>
                                <small
                                    >Hides raw &lt;div&gt;, &lt;span&gt; and
                                    other HTML tags</small
                                >
                            </div>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.reader_hide_callouts
                                        }
                                    />
                                    Hide Callouts
                                </label>
                            </div>
                        </section>
                    </div>
                {:else if activePanel === "activation"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>Activation</h2>
                            <p class="section-description">
                                Control when and how edge detection triggers the
                                panels.
                            </p>
                        </section>

                        <section>
                            <h2>Edge Detection</h2>
                            <p class="section-description">
                                Enable or disable screen-edge activation for the
                                note and reader windows.
                            </p>

                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.edge_detection_enabled
                                        }
                                    />
                                    Edge Detection enabled
                                </label>
                                <small
                                    >Panels open when moving mouse to screen
                                    edges</small
                                >
                            </div>
                        </section>

                        <section>
                            <h2>Open Delays</h2>
                            <p class="section-description">
                                Set separate edge-trigger delays for each
                                window. If disabled, the standard trigger timing
                                is used.
                            </p>

                            <div class="delay-grid">
                                <div class="delay-card">
                                    <div class="delay-card-title">
                                        Note Window
                                    </div>
                                    <div class="delay-toggle-row">
                                        <label
                                            class="checkbox compact-checkbox"
                                        >
                                            <input
                                                type="checkbox"
                                                bind:checked={
                                                    settings.note_edge_open_delay_enabled
                                                }
                                            />
                                            Open Delay
                                        </label>

                                        {#if settings.note_edge_open_delay_enabled}
                                            <label class="delay-input">
                                                <input
                                                    type="number"
                                                    bind:value={
                                                        settings.note_edge_open_delay_ms
                                                    }
                                                    min="50"
                                                    max="10000"
                                                    step="50"
                                                    on:blur={() =>
                                                        normalizeDelayField(
                                                            "note_edge_open_delay_ms",
                                                        )}
                                                />
                                                <span>ms</span>
                                            </label>
                                        {/if}
                                    </div>
                                    <small>
                                        Wait this long before the note window
                                        opens when touching the edge.
                                    </small>
                                </div>

                                <div class="delay-card">
                                    <div class="delay-card-title">
                                        Reader Window
                                    </div>
                                    <div class="delay-toggle-row">
                                        <label
                                            class="checkbox compact-checkbox"
                                        >
                                            <input
                                                type="checkbox"
                                                bind:checked={
                                                    settings.reader_edge_open_delay_enabled
                                                }
                                            />
                                            Open Delay
                                        </label>

                                        {#if settings.reader_edge_open_delay_enabled}
                                            <label class="delay-input">
                                                <input
                                                    type="number"
                                                    bind:value={
                                                        settings.reader_edge_open_delay_ms
                                                    }
                                                    min="50"
                                                    max="10000"
                                                    step="50"
                                                    on:blur={() =>
                                                        normalizeDelayField(
                                                            "reader_edge_open_delay_ms",
                                                        )}
                                                />
                                                <span>ms</span>
                                            </label>
                                        {/if}
                                    </div>
                                    <small>
                                        Wait this long before the reader opens
                                        when touching the edge.
                                    </small>
                                </div>
                            </div>
                        </section>

                        <section>
                            <h2>Modifier Keys</h2>
                            <p class="section-description">
                                Hold these keys while touching the edge to open
                                a panel. Leave all unchecked to open without any
                                modifier.
                            </p>

                            <div class="field">
                                <div class="modifier-grid">
                                    {#each ["cmd", "option", "shift", "ctrl"] as mod}
                                        <label
                                            class="checkbox modifier-checkbox"
                                        >
                                            <input
                                                type="checkbox"
                                                checked={settings.edge_modifier_keys?.includes(
                                                    mod,
                                                )}
                                                on:change={(event) => {
                                                    const keys =
                                                        settings.edge_modifier_keys ??
                                                        [];
                                                    if (
                                                        event.currentTarget
                                                            .checked
                                                    ) {
                                                        settings.edge_modifier_keys =
                                                            [...keys, mod];
                                                    } else {
                                                        settings.edge_modifier_keys =
                                                            keys.filter(
                                                                (key) =>
                                                                    key !== mod,
                                                            );
                                                    }
                                                    settings = { ...settings };
                                                }}
                                            />
                                            {modifierLabel(mod)}
                                        </label>
                                    {/each}
                                </div>
                            </div>
                        </section>

                        <section>
                            <h2>Excluded Apps</h2>
                            <p class="section-description">
                                Edge detection is paused when these apps are in
                                focus.
                            </p>

                            <div class="field">
                                {#if (settings.edge_excluded_apps ?? []).length > 0}
                                    <ul class="exclusion-list">
                                        {#each settings.edge_excluded_apps as app}
                                            <li class="exclusion-item">
                                                <span class="exclusion-name"
                                                    >{app}</span
                                                >
                                                <button
                                                    class="exclusion-remove"
                                                    type="button"
                                                    on:click={() =>
                                                        removeExcludedApp(app)}
                                                >
                                                    ✕
                                                </button>
                                            </li>
                                        {/each}
                                    </ul>
                                {/if}

                                <button
                                    class="secondary add-app-btn"
                                    type="button"
                                    on:click={toggleAppPicker}
                                >
                                    + Add App
                                </button>

                                {#if showAppPicker}
                                    <div class="app-picker">
                                        <input
                                            bind:value={appPickerQuery}
                                            class="app-picker-search"
                                            placeholder="Filter apps…"
                                            on:input={refreshFilteredApps}
                                        />
                                        <div class="app-picker-list">
                                            {#each filteredRunningApps as app}
                                                <button
                                                    class="app-picker-item"
                                                    type="button"
                                                    on:click={() =>
                                                        addExcludedApp(app)}
                                                    disabled={settings.edge_excluded_apps?.includes(
                                                        app,
                                                    )}
                                                >
                                                    {app}
                                                    {#if settings.edge_excluded_apps?.includes(app)}
                                                        <span class="app-added"
                                                            >✓</span
                                                        >
                                                    {/if}
                                                </button>
                                            {/each}
                                        </div>
                                    </div>
                                {/if}
                            </div>
                        </section>
                    </div>
                {:else if activePanel === "shortcuts"}
                    <div class="settings-panel">
                        <section class="panel-intro">
                            <h2>Shortcuts</h2>
                            <p class="section-description">
                                Keyboard shortcuts for both windows. Click in
                                the field and press the desired key combination.
                            </p>
                        </section>

                        <section>
                            <h2>Note Window</h2>
                            <p class="section-description">
                                Global shortcut for opening the capture window,
                                with optional closing via the same or a separate
                                shortcut.
                            </p>

                            <div class="field">
                                <label for="global_shortcut"
                                    >Open Note Window</label
                                >
                                <input
                                    class="shortcut-input"
                                    type="text"
                                    id="global_shortcut"
                                    bind:value={settings.global_shortcut}
                                    placeholder="Cmd+Shift+N"
                                    on:keydown={(e) =>
                                        handleShortcutKeyDown(
                                            e,
                                            "global_shortcut",
                                        )}
                                />
                            </div>
                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.global_shortcut_closes_window
                                        }
                                    />
                                    <span
                                        >Use the same shortcut to close the note
                                        window</span
                                    >
                                </label>
                            </div>
                            {#if !settings.global_shortcut_closes_window}
                                <div class="field">
                                    <label for="global_close_shortcut"
                                        >Close Note Window</label
                                    >
                                    <input
                                        class="shortcut-input"
                                        type="text"
                                        id="global_close_shortcut"
                                        bind:value={
                                            settings.global_close_shortcut
                                        }
                                        placeholder="Optional"
                                        on:keydown={(e) =>
                                            handleShortcutKeyDown(
                                                e,
                                                "global_close_shortcut",
                                            )}
                                    />
                                    <small
                                        >Leave empty to disable closing via
                                        shortcut</small
                                    >
                                </div>
                            {/if}
                        </section>

                        <section>
                            <h2>Reader Window</h2>
                            <p class="section-description">
                                Global shortcut for opening the reader, with an
                                optional close shortcut.
                            </p>

                            <div class="field">
                                <label for="reader_shortcut"
                                    >Open Reader Window</label
                                >
                                <input
                                    class="shortcut-input"
                                    type="text"
                                    id="reader_shortcut"
                                    bind:value={settings.reader_shortcut}
                                    placeholder="Cmd+Shift+R"
                                    on:keydown={(e) =>
                                        handleShortcutKeyDown(
                                            e,
                                            "reader_shortcut",
                                        )}
                                />
                            </div>
                            <div class="field">
                                <label class="checkbox">
                                    <input
                                        type="checkbox"
                                        bind:checked={
                                            settings.reader_shortcut_closes_window
                                        }
                                    />
                                    <span
                                        >Use the same shortcut to close the
                                        reader window</span
                                    >
                                </label>
                            </div>
                            {#if !settings.reader_shortcut_closes_window}
                                <div class="field">
                                    <label for="reader_close_shortcut"
                                        >Close Reader Window</label
                                    >
                                    <input
                                        class="shortcut-input"
                                        type="text"
                                        id="reader_close_shortcut"
                                        bind:value={
                                            settings.reader_close_shortcut
                                        }
                                        placeholder="Optional"
                                        on:keydown={(e) =>
                                            handleShortcutKeyDown(
                                                e,
                                                "reader_close_shortcut",
                                            )}
                                    />
                                    <small
                                        >Leave empty to disable closing via
                                        shortcut</small
                                    >
                                </div>
                            {/if}
                        </section>

                        <section>
                            <h2>Copy Text to Collector</h2>
                            <p class="section-description">
                                Copies the current selection from the active app
                                into the capture window.
                            </p>

                            <div class="field">
                                <label for="capture_text_shortcut"
                                    >Shortcut</label
                                >
                                <input
                                    class="shortcut-input"
                                    type="text"
                                    id="capture_text_shortcut"
                                    bind:value={settings.capture_text_shortcut}
                                    placeholder="Cmd+Shift+C"
                                    on:keydown={(e) =>
                                        handleShortcutKeyDown(
                                            e,
                                            "capture_text_shortcut",
                                        )}
                                />
                            </div>
                            <div class="info-note">
                                <div class="info-note-title">
                                    Accessibility Permission
                                </div>
                                <p>
                                    Required for "Copy Text to Collector".
                                    Enable Collector in
                                    <strong
                                        >System Settings → Privacy & Security →
                                        Accessibility</strong
                                    >, then restart the app.
                                </p>
                            </div>
                        </section>

                        <section>
                            <h2>Save Actions</h2>
                            <p class="section-description">
                                Shortcuts for saving the current capture into
                                your vault.
                            </p>

                            <div class="field">
                                <label for="save_to_daily_shortcut"
                                    >Save to Daily Note</label
                                >
                                <input
                                    class="shortcut-input"
                                    type="text"
                                    id="save_to_daily_shortcut"
                                    bind:value={settings.save_to_daily_shortcut}
                                    placeholder="Cmd+Enter"
                                    on:keydown={(e) =>
                                        handleShortcutKeyDown(
                                            e,
                                            "save_to_daily_shortcut",
                                        )}
                                />
                            </div>
                            <div class="field">
                                <label for="save_as_note_shortcut"
                                    >Create New Note</label
                                >
                                <input
                                    class="shortcut-input"
                                    type="text"
                                    id="save_as_note_shortcut"
                                    bind:value={settings.save_as_note_shortcut}
                                    placeholder="Cmd+Shift+Enter"
                                    on:keydown={(e) =>
                                        handleShortcutKeyDown(
                                            e,
                                            "save_as_note_shortcut",
                                        )}
                                />
                            </div>
                            <div class="field">
                                <label for="append_to_note_shortcut"
                                    >Append to Note</label
                                >
                                <input
                                    class="shortcut-input"
                                    type="text"
                                    id="append_to_note_shortcut"
                                    bind:value={
                                        settings.append_to_note_shortcut
                                    }
                                    placeholder="Cmd+Option+Enter"
                                    on:keydown={(e) =>
                                        handleShortcutKeyDown(
                                            e,
                                            "append_to_note_shortcut",
                                        )}
                                />
                            </div>
                        </section>
                    </div>
                {/if}
            </div>
        </div>
    </main>

    <footer>
        <button class="secondary" on:click={handleReset}>Reset</button>
        <div class="spacer"></div>
        <button class="secondary" on:click={handleCancel}>Cancel</button>
        <button
            class="primary"
            on:click={handleSave}
            disabled={isSaving || !hasChanges}
        >
            {isSaving ? "Saving..." : "Save"}
        </button>
    </footer>
</div>

<style>
    .settings-container {
        display: flex;
        flex-direction: column;
        height: 100vh;
        font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
        font-size: 13px;
        color: #1a1a1a;
        background: #f5f5f7;
    }

    header {
        padding: 20px 24px;
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(40px);
        -webkit-backdrop-filter: blur(40px);
        border-bottom: 1px solid rgba(0, 0, 0, 0.08);
        display: flex;
        align-items: center;
        gap: 16px;
    }

    header h1 {
        font-size: 20px;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.3px;
    }

    .status {
        padding: 8px 14px;
        border-radius: 8px;
        font-size: 12px;
        font-weight: 500;
        letter-spacing: 0.2px;
        animation: slideIn 0.3s ease;
    }

    @keyframes slideIn {
        from {
            opacity: 0;
            transform: translateX(-8px);
        }
    }

    .status.success {
        background: var(--success-bg);
        color: var(--success-color);
        border: 1px solid var(--success-border);
    }

    .status.error {
        background: var(--error-bg);
        color: var(--error-color);
        border: 1px solid var(--error-border);
    }

    main {
        flex: 1;
        padding: 20px 24px;
        overflow: hidden;
    }

    .settings-layout {
        display: grid;
        grid-template-columns: 240px minmax(0, 1fr);
        gap: 20px;
        height: 100%;
        min-height: 0;
    }

    .settings-sidebar {
        min-height: 0;
    }

    .settings-nav {
        display: flex;
        flex-direction: column;
        gap: 10px;
        position: sticky;
        top: 0;
    }

    .nav-item {
        display: flex;
        flex-direction: column;
        align-items: flex-start;
        gap: 4px;
        width: 100%;
        padding: 14px 16px;
        border-radius: 14px;
        border: 1px solid rgba(0, 0, 0, 0.08);
        background: rgba(255, 255, 255, 0.88);
        backdrop-filter: blur(30px);
        -webkit-backdrop-filter: blur(30px);
        color: #374151;
        text-align: left;
        transition:
            transform 0.18s ease,
            box-shadow 0.18s ease,
            border-color 0.18s ease,
            background 0.18s ease;
    }

    .nav-item:hover {
        background: linear-gradient(
            170deg,
            color-mix(in srgb, var(--accent-color) 3%, transparent) 0%,
            rgba(255, 255, 255, 0.94) 100%
        );
        box-shadow: inset 0px 0px 6px 2px rgba(15, 23, 42, 0.06);
    }

    .nav-item.active {
        background: linear-gradient(
            170deg,
            color-mix(in srgb, var(--accent-color) 6%, transparent) 0%,
            rgba(255, 255, 255, 0.96) 100%
        );

        box-shadow: inset 0px 0px 6px 2px rgba(15, 23, 42, 0.06);
    }

    .nav-item-label {
        font-size: 13px;
        font-weight: 600;
        color: #111827;
    }

    .nav-item-description {
        font-size: 11px;
        line-height: 1.45;
        color: #6b7280;
    }

    .settings-content {
        min-width: 0;
        min-height: 0;
        overflow-y: auto;
        padding-right: 4px;
    }

    .settings-panel {
        display: flex;
        flex-direction: column;
        gap: 16px;
        padding-bottom: 12px;
    }

    section {
        background: linear-gradient(
            180deg,
            rgba(255, 255, 255, 0.97) 0%,
            rgba(248, 250, 252, 0.94) 100%
        );
        backdrop-filter: blur(40px);
        -webkit-backdrop-filter: blur(40px);
        border-radius: 16px;
        padding: 22px;
        margin-bottom: 0;
        border: 1px solid rgba(15, 23, 42, 0.08);
    }

    section h2 {
        font-size: 15px;
        font-weight: 600;
        margin: 0 0 10px 0;
        color: #111827;
        letter-spacing: -0.24px;
    }

    .panel-intro {
        background: linear-gradient(
            155deg,
            color-mix(in srgb, var(--accent-color) 5%, white) 0%,
            rgba(255, 255, 255, 0.98) 100%
        );
    }

    .panel-intro h2 {
        font-size: 16px;
        margin-bottom: 8px;
    }

    .panel-intro .section-description {
        max-width: 620px;
    }

    .modifier-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
        margin-top: 6px;
    }

    .delay-grid {
        display: grid;
        grid-template-columns: repeat(2, minmax(0, 1fr));
        gap: 12px;
        margin-top: 8px;
    }

    .delay-card {
        padding: 12px;
        border: 1.5px solid rgba(0, 0, 0, 0.08);
        border-radius: 10px;
        background: rgba(248, 250, 252, 0.72);
    }

    .delay-card-title {
        font-size: 12px;
        font-weight: 600;
        color: #111827;
        margin-bottom: 8px;
    }

    .delay-toggle-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
    }

    .compact-checkbox {
        display: flex;
        align-items: center;
        margin-bottom: 0;
    }

    .delay-input {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        margin-bottom: 0;
    }

    .delay-input input {
        width: 110px;
        margin: 0;
    }

    .delay-input span {
        color: #6b7280;
        font-size: 12px;
        font-weight: 500;
    }

    .modifier-checkbox {
        display: flex;
        align-items: center;
        gap: 8px;
        padding: 8px 10px;
        border: 1.5px solid rgba(0, 0, 0, 0.1);
        border-radius: 8px;
        cursor: pointer;
        transition:
            border-color 0.15s,
            background 0.15s;
        font-size: 13px;
    }

    .modifier-checkbox:has(input:checked) {
        border-color: #8b5cf6;
        background: rgba(139, 92, 246, 0.06);
    }

    .exclusion-list {
        list-style: none;
        margin: 8px 0;
        padding: 0;
        display: flex;
        flex-direction: column;
        gap: 4px;
    }

    .exclusion-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 6px 10px;
        background: rgba(0, 0, 0, 0.04);
        border-radius: 6px;
        font-size: 13px;
    }

    .exclusion-name {
        min-width: 0;
    }

    .exclusion-remove {
        border: none;
        background: none;
        cursor: pointer;
        color: #999;
        font-size: 11px;
        padding: 2px 4px;
        border-radius: 3px;
    }

    .exclusion-remove:hover {
        color: #ef4444;
        background: rgba(239, 68, 68, 0.1);
    }

    .add-app-btn {
        margin-top: 8px;
        font-size: 12px;
        padding: 6px 12px;
    }

    .app-picker {
        margin-top: 8px;
        border: 1.5px solid rgba(0, 0, 0, 0.1);
        border-radius: 8px;
        overflow: hidden;
        background: #fff;
    }

    .app-picker-search {
        width: 100%;
        border: none;
        border-bottom: 1px solid rgba(0, 0, 0, 0.08);
        padding: 8px 12px;
        font-size: 13px;
        outline: none;
    }

    .app-picker-list {
        max-height: 200px;
        overflow-y: auto;
    }

    .app-picker-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        width: 100%;
        padding: 8px 12px;
        border: none;
        background: transparent;
        text-align: left;
        font-size: 13px;
        cursor: pointer;
        transition: background 0.1s;
    }

    .app-picker-item:hover {
        background: rgba(139, 92, 246, 0.06);
    }

    .app-picker-item:disabled {
        opacity: 0.5;
        cursor: default;
    }

    .app-added {
        color: #22c55e;
        font-size: 12px;
    }

    .field {
        margin-bottom: 12px;
    }

    .field:last-child {
        margin-bottom: 0;
    }

    .field label {
        display: block;
        font-weight: 500;
        margin-bottom: 4px;
    }

    .field-label {
        display: block;
        font-weight: 500;
        margin-bottom: 4px;
    }

    .field small {
        display: block;
        color: #888;
        font-size: 11px;
        margin-top: 4px;
    }

    .field-row {
        display: flex;
        gap: 12px;
    }

    .field-row .field {
        flex: 1;
    }

    .info-note {
        margin-top: 14px;
        padding: 13px 14px;
        border-radius: 12px;
        border: 1px solid rgba(15, 23, 42, 0.08);
        background: rgba(15, 23, 42, 0.035);
    }

    .info-note-title {
        font-size: 12px;
        font-weight: 600;
        color: #111827;
        margin-bottom: 4px;
    }

    .info-note p {
        margin: 0;
        color: #6b7280;
        font-size: 12px;
        line-height: 1.45;
    }

    input[type="text"],
    input[type="number"],
    select,
    textarea {
        width: 100%;
        padding: 9px 12px;
        border: 1.5px solid rgba(0, 0, 0, 0.1);
        border-radius: 8px;
        font-size: 13px;
        background: white;
        transition: all 0.2s ease;
        font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
    }

    textarea {
        resize: vertical;
        min-height: 100px;
        font-family: "SF Mono", Menlo, Monaco, monospace;
        font-size: 12px;
    }

    .shortcut-input {
        width: 280px;
        max-width: 100%;
    }

    input[type="text"]:focus,
    input[type="number"]:focus,
    select:focus,
    textarea:focus {
        outline: none;
        box-shadow: inset 0px 0px 6px 1px rgba(15, 23, 42, 0.06);
        background: linear-gradient(
            170deg,
            rgba(139, 92, 246, 0.05) 20%,
            rgba(255, 255, 255, 0.04) 80%
        );
    }

    input[type="range"] {
        width: 100%;
        margin: 4px 0;
    }

    .color-input {
        display: flex;
        gap: 8px;
        align-items: center;
    }

    .color-input input[type="color"] {
        width: 40px;
        height: 32px;
        padding: 2px;
        border: 1px solid #ddd;
        border-radius: 4px;
        cursor: pointer;
    }

    .color-input input[type="text"] {
        flex: 1;
        font-family: monospace;
    }

    .section-description {
        margin: 0 0 14px;
        color: #6b7280;
        font-size: 13px;
        line-height: 1.45;
    }

    .checkbox {
        display: flex;
        align-items: center;
        gap: 6px;
        cursor: pointer;
        font-weight: normal;
    }

    .template-editor {
        margin-top: 8px;
    }

    .modifier-help {
        display: block;
        margin-bottom: 8px;
    }

    footer {
        padding: 16px 24px;
        background: rgba(255, 255, 255, 0.95);
        backdrop-filter: blur(40px);
        -webkit-backdrop-filter: blur(40px);
        border-top: 1px solid rgba(0, 0, 0, 0.08);
        display: flex;
        gap: 10px;
        align-items: center;
    }

    .spacer {
        flex: 1;
    }

    button {
        padding: 8px 16px;
        border-radius: 6px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        border: none;
        transition: all 0.2s ease;
    }

    button.primary {
        background: linear-gradient(
            135deg,
            var(--accent-color, #8b5cf6) 0%,
            #7c3aed 100%
        );
        color: white;
        border: 1px solid
            color-mix(in srgb, var(--accent-color, #8b5cf6) 35%, transparent);
        box-shadow: 0 2px 8px
            color-mix(in srgb, var(--accent-color, #8b5cf6) 25%, transparent);
    }

    button.primary:hover:not(:disabled) {
        background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
        box-shadow: 0 4px 12px
            color-mix(in srgb, var(--accent-color, #8b5cf6) 30%, transparent);
        transform: translateY(-1px);
    }

    button.primary:disabled {
        opacity: 0.75;
        background: linear-gradient(135deg, #a78bfa 0%, #8b5cf6 100%);
        color: rgba(255, 255, 255, 0.96);
        cursor: not-allowed;
    }

    button.secondary {
        background: #e5e5e5;
        color: #333;
    }

    button.secondary:hover {
        background: #d5d5d5;
    }

    .path-picker {
        display: flex;
        gap: 8px;
    }

    .path-picker input {
        flex: 1;
        font-family: "SF Mono", Menlo, Monaco, monospace;
        font-size: 12px;
        background: #f9f9f9;
    }

    .path-picker button {
        padding: 8px 16px;
        white-space: nowrap;
    }

    .reindex-row {
        display: flex;
        gap: 8px;
        align-items: center;
        margin-top: 8px;
    }

    .index-status {
        color: #666;
        font-size: 11px;
    }

    .note-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin: 10px 0 12px;
    }

    .note-list-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 10px 12px;
        border-radius: 10px;
        background: rgba(0, 0, 0, 0.03);
        border: 1px solid rgba(0, 0, 0, 0.06);
    }

    .note-list-copy {
        min-width: 0;
    }

    .note-list-editors {
        display: block;
    }

    .pinned-note-label {
        width: 100%;
        font-weight: 600;
    }

    .pinned-note-icons {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-top: 8px;
    }

    .icon-choice {
        width: 30px;
        height: 30px;
        padding: 0;
        border-radius: 8px;
        border: 1px solid rgba(0, 0, 0, 0.08);
        background: rgba(255, 255, 255, 0.72);
        color: #6b7280;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        box-shadow: none;
    }

    .icon-choice:hover {
        color: #374151;
        border-color: rgba(139, 92, 246, 0.22);
        background: rgba(139, 92, 246, 0.08);
        transform: none;
    }

    .icon-choice.selected {
        color: #7c3aed;
        border-color: rgba(124, 58, 237, 0.28);
        background: rgba(139, 92, 246, 0.14);
    }

    .icon-choice-none {
        font-size: 14px;
        line-height: 1;
    }

    .note-list-copy small {
        display: block;
        margin-top: 3px;
        color: #6b7280;
        overflow-wrap: anywhere;
    }

    .empty-note-list {
        margin: 10px 0 12px;
        padding: 12px;
        border-radius: 10px;
        background: rgba(0, 0, 0, 0.03);
        border: 1px dashed rgba(0, 0, 0, 0.08);
        color: #6b7280;
        font-size: 12px;
    }

    .remove-note {
        flex-shrink: 0;
        width: 28px;
        height: 28px;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: #9ca3af;
        cursor: pointer;
        transition:
            background 0.2s ease,
            color 0.2s ease;
    }

    .remove-note:hover {
        background: rgba(239, 68, 68, 0.1);
        color: #dc2626;
    }

    @media (max-width: 860px) {
        main {
            overflow-y: auto;
        }

        .settings-layout {
            grid-template-columns: 1fr;
            height: auto;
        }

        .settings-sidebar {
            overflow-x: auto;
        }

        .settings-nav {
            position: static;
            flex-direction: row;
            align-items: stretch;
            padding-bottom: 4px;
        }

        .nav-item {
            min-width: 220px;
        }

        .settings-content {
            overflow: visible;
            padding-right: 0;
            background: none;
        }

        .settings-panel {
            padding-bottom: 0;
            background: transparent;
        }

        .delay-grid {
            grid-template-columns: 1fr;
        }

        .field-row {
            flex-direction: column;
            gap: 0;
        }

        .delay-toggle-row {
            align-items: flex-start;
            flex-direction: column;
        }
    }
</style>
