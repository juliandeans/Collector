<script>
    import { invoke } from "@tauri-apps/api/core";
    import { onMount, onDestroy } from "svelte";
    import { confirm as dialogConfirm } from "@tauri-apps/plugin-dialog";
    import PanelActivation from "./lib/settings/PanelActivation.svelte";
    import PanelCapture from "./lib/settings/PanelCapture.svelte";
    import PanelImages from "./lib/settings/PanelImages.svelte";
    import PanelLook from "./lib/settings/PanelLook.svelte";
    import PanelReader from "./lib/settings/PanelReader.svelte";
    import PanelShortcuts from "./lib/settings/PanelShortcuts.svelte";
    import PanelVault from "./lib/settings/PanelVault.svelte";
    import { normalizeDelayValue } from "./lib/settings/delay-utils.js";
    import { normalizePinnedNotes } from "./lib/settings/pinned-notes.js";
    import { validateSettings } from "./lib/settings/validation.js";
    import { defaultSettings } from "./lib/stores.js";

    let settings = { ...defaultSettings };
    let vaultNotes = [];
    let isLoaded = false;
    let statusMessage = "";
    let statusType = "";
    let activePanel = "vault";
    let statusTimeout;

    const settingsPanels = [
        { id: "vault", label: "Vault" },
        { id: "capture", label: "Capture" },
        { id: "reader", label: "Reader" },
        { id: "look", label: "Look & Feel" },
        { id: "shortcuts", label: "Shortcuts" },
        { id: "activation", label: "Activation" },
        { id: "images", label: "Images" },
    ];

    // ── theme (system dark/light) ─────────────────────────

    let prefersDark = false;

    function updateTheme(mq) {
        prefersDark = mq.matches;
    }

    onMount(() => {
        const mq = window.matchMedia("(prefers-color-scheme: dark)");
        updateTheme(mq);
        mq.addEventListener("change", updateTheme);
    });

    const lightTheme = {
        bg: "#f2f2f2",
        surface: "rgba(255,255,255,0.95)",
        text: "#1a1a1a",
        textSecondary: "#6b7280",
        inputBg: "#ffffff",
        border: "rgba(0,0,0,0.08)",
        inputBorder: "rgba(0,0,0,0.10)",
        navText: "#374151",
        navLabel: "#111827",
        sectionTitle: "#9ca3af",
        fieldLabel: "#111827",
        btnBg: "rgba(0,0,0,0.07)",
        btnText: "#111827",
        indicator: "#6b7280",
    };

    const darkTheme = {
        bg: "#1e1e2e",
        surface: "rgba(255,255,255,0.06)",
        text: "#e4e4e7",
        textSecondary: "rgba(255,255,255,0.5)",
        inputBg: "rgba(255,255,255,0.08)",
        border: "rgba(255,255,255,0.08)",
        inputBorder: "rgba(255,255,255,0.12)",
        navText: "rgba(255,255,255,0.65)",
        navLabel: "rgba(255,255,255,0.9)",
        sectionTitle: "rgba(255,255,255,0.4)",
        fieldLabel: "rgba(255,255,255,0.8)",
        btnBg: "rgba(255,255,255,0.08)",
        btnText: "rgba(255,255,255,0.85)",
        indicator: "rgba(255,255,255,0.45)",
    };

    $: t = prefersDark ? darkTheme : lightTheme;
    $: validation = validateSettings(settings);
    $: fieldIssues = validation.fields;

    // ── auto-save ──────────────────────────────────────────

    let saveTimer;

    function scheduleAutoSave() {
        if (!isLoaded) return;
        clearTimeout(saveTimer);
        saveTimer = setTimeout(() => {
            saveTimer = null;
            void performSave();
        }, 400);
    }

    function flushPendingSave() {
        if (!saveTimer) return;
        clearTimeout(saveTimer);
        saveTimer = null;
        void performSave();
    }

    async function performSave() {
        const currentValidation = validateSettings(settings);
        if (currentValidation.hasErrors) {
            const plural = currentValidation.errorCount === 1 ? "" : "s";
            showStatus(
                `Not saved: fix ${currentValidation.errorCount} highlighted setting${plural}`,
                "error",
            );
            return false;
        }

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
            showStatus("Saved", "success");
            return true;
        } catch (e) {
            console.error("Auto-save failed:", e);
            showStatus("Save failed: " + e.toString(), "error");
            return false;
        }
    }

    // ── lifecycle ──────────────────────────────────────────

    async function loadSettings() {
        try {
            const loaded = await invoke("load_settings");
            const normalized = {
                ...defaultSettings,
                ...loaded,
                pinned_notes: normalizePinnedNotes(loaded.pinned_notes),
            };
            settings = normalized;
        } catch (e) {
            console.error("Failed to load settings:", e);
            showStatus("Failed to load settings", "error");
        } finally {
            isLoaded = true;
        }
    }

    onMount(async () => {
        await loadSettings();
        try {
            vaultNotes = await invoke("list_vault_notes");
        } catch (e) {
            console.error("Failed to load vault notes:", e);
        }
    });

    onDestroy(() => {
        clearTimeout(statusTimeout);
        flushPendingSave();
    });

    // ── ui helpers ─────────────────────────────────────────

    function showStatus(message, type = "success") {
        clearTimeout(statusTimeout);
        statusMessage = message;
        statusType = type;
        statusTimeout = setTimeout(() => {
            statusMessage = "";
            statusType = "";
        }, 2000);
    }

    async function handleClose() {
        statusMessage = "";
        statusType = "";
        flushPendingSave();
        try {
            await invoke("close_settings");
        } catch (e) {
            console.error("Failed to close settings window:", e);
        }
    }

    async function handleReset() {
        const confirmed = await dialogConfirm(
            "Reset all settings to defaults? Your vault connection will be kept.",
            { title: "Reset settings", kind: "warning" }
        );
        if (!confirmed) return;

        const vaultPath = settings.vault_path || defaultSettings.vault_path;
        const vaultName = settings.vault_name || defaultSettings.vault_name;
        settings = {
            ...defaultSettings,
            vault_path: vaultPath,
            vault_name: vaultName,
        };
        await performSave();
    }
</script>

<div class="settings-container"
    style="
        --settings-bg: {t.bg};
        --settings-surface: {t.surface};
        --settings-text: {t.text};
        --settings-text-secondary: {t.textSecondary};
        --settings-input-bg: {t.inputBg};
        --settings-border: {t.border};
        --settings-input-border: {t.inputBorder};
        --settings-nav-text: {t.navText};
        --settings-nav-label: {t.navLabel};
        --settings-section-title: {t.sectionTitle};
        --settings-label: {t.fieldLabel};
        --settings-btn-bg: {t.btnBg};
        --settings-btn-text: {t.btnText};
        --settings-indicator: {t.indicator};
        --settings-accent: {settings.accent_color || '#8b5cf6'};
    "
>
    <header>
        <h1>Settings</h1>
        {#if statusMessage}
            <span
                class="save-indicator"
                class:visible={statusMessage}
                class:error={statusType === "error"}
            >
                {statusMessage}
            </span>
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
                        </button>
                    {/each}
                </nav>
            </aside>

            <div class="settings-content" on:input={scheduleAutoSave}>
                {#if activePanel === "vault"}
                    <PanelVault bind:settings {showStatus} validation={fieldIssues} onChange={scheduleAutoSave} />
                {:else if activePanel === "capture"}
                    <PanelCapture bind:settings {showStatus} validation={fieldIssues} />
                {:else if activePanel === "reader"}
                    <PanelReader bind:settings validation={fieldIssues} {vaultNotes} onChange={scheduleAutoSave} />
                {:else if activePanel === "look"}
                    <PanelLook bind:settings {showStatus} validation={fieldIssues} />
                {:else if activePanel === "shortcuts"}
                    <PanelShortcuts bind:settings {showStatus} validation={fieldIssues} onChange={scheduleAutoSave} />
                {:else if activePanel === "activation"}
                    <PanelActivation bind:settings validation={fieldIssues} onChange={scheduleAutoSave} />
                {:else if activePanel === "images"}
                    <PanelImages bind:settings {showStatus} validation={fieldIssues} onChange={scheduleAutoSave} />
                {/if}
            </div>
        </div>
    </main>

    <footer>
        <button class="secondary" on:click={handleReset}>Reset All</button>
        <div class="spacer"></div>
        <button class="secondary" on:click={handleClose}>Close</button>
    </footer>
</div>

<style>
    .settings-container {
        display: flex;
        flex-direction: column;
        height: 100vh;
        font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
        font-size: 13px;
        color: var(--settings-text);
        background: var(--settings-bg);
    }

    header {
        padding: 12px 18px;
        background: var(--settings-surface);
        backdrop-filter: blur(40px);
        -webkit-backdrop-filter: blur(40px);
        border-bottom: 1px solid var(--settings-border);
        display: flex;
        align-items: center;
        gap: 12px;
    }

    header h1 {
        font-size: 20px;
        font-weight: 600;
        margin: 0;
        letter-spacing: -0.3px;
    }

    .save-indicator {
        font-size: 11px;
        font-weight: 600;
        padding: 4px 10px;
        border-radius: 5px;
        color: #065f46;
        background: #d1fae5;
        opacity: 0;
        transition: opacity 0.2s ease;
        letter-spacing: 0.1px;
    }

    .save-indicator.visible {
        opacity: 1;
    }

    .save-indicator.error {
        color: #991b1b;
        background: #fecaca;
    }

    main {
        flex: 1;
        overflow: hidden;
    }

    .settings-layout {
        display: grid;
        grid-template-columns: max-content minmax(0, 1fr);
        gap: 0;
        height: 100%;
        min-height: 0;
    }

    .settings-sidebar {
        min-height: 0;
        background: var(--settings-surface);
        border-right: 1px solid var(--settings-border);
        padding-right: 0;
    }

    .settings-nav {
        display: flex;
        flex-direction: column;
        gap: 0px;
        position: sticky;
        top: 0;
        padding: 0px 0px 8px 0;
    }

    .nav-item {
        display: flex;
        align-items: center;
        width: 100%;
        padding: 12px 20px;
        border-radius: 0;
        border: none;
        background: none;
        backdrop-filter: none;
        -webkit-backdrop-filter: none;
        color: var(--settings-nav-text);
        text-align: left;
        transition: background 0.15s ease;
    }

    .nav-item:hover {
        background: rgba(128, 128, 128, 0.08);
    }

    .nav-item.active {
        background: rgba(128, 128, 128, 0.12);
    }

    .nav-item.active:hover {
        background: rgba(128, 128, 128, 0.12);
    }

    .nav-item-label {
        font-size: 13px;
        font-weight: 500;
        color: var(--settings-nav-label);
    }

    .settings-content {
        min-width: 0;
        min-height: 0;
        overflow-y: auto;
        padding-left: 24px;
        padding-right: 24px;
        background: none;
    }

    :global(.settings-panel) {
        display: flex;
        flex-direction: column;
        gap: 0;
        padding-bottom: 12px;
    }

    :global(.field) {
        margin-bottom: 4px;
    }

    :global(.field:last-child) {
        margin-bottom: 0;
    }

    :global(.field label),
    :global(.field .field-label) {
        display: block;
        font-size: 13px;
        font-weight: 500;
        color: var(--settings-label);
        margin-bottom: 5px;
        padding-top: 12px;
    }

    :global(.field small) {
        display: block;
        font-size: 11px;
        font-weight: 400;
        color: var(--settings-text-secondary);
        margin-top: 4px;
        padding-left: 14px;
        padding-bottom: 4px;
    }

    :global(.field small.field-issue) {
        font-weight: 600;
        padding-bottom: 0;
    }

    :global(.field small.field-issue.error) {
        color: #b44d4d;
    }

    :global(.field small.field-issue.warning) {
        color: #9a6a2f;
    }

    :global(input[type="text"]),
    :global(input[type="number"]),
    :global(textarea) {
        width: 100%;
        padding: 9px 12px;
        border: 1.5px solid var(--settings-input-border);
        border-radius: 6px;
        font-size: 13px;
        background: var(--settings-input-bg);
        color: var(--settings-text);
        transition: all 0.2s ease;
        font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
    }

    :global(input[type="number"]) {
        -moz-appearance: textfield;
    }

    :global(input[type="number"]::-webkit-inner-spin-button),
    :global(input[type="number"]::-webkit-outer-spin-button) {
        -webkit-appearance: none;
        appearance: none;
        display: none;
    }

    :global(select) {
        display: block;
        width: 100%;
        padding: 9px 28px 9px 12px;
        border: 1.5px solid var(--settings-input-border);
        border-radius: 6px;
        font-size: 13px;
        background: var(--settings-input-bg);
        color: var(--settings-text);
        transition: all 0.2s ease;
        font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
        min-height: 36px;
        box-sizing: border-box;
        -moz-appearance: none;
        -webkit-appearance: none;
        appearance: none;
        background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='10' height='6'%3E%3Cpath d='M0 0l5 6 5-6z' fill='%23999'/%3E%3C/svg%3E");
        background-repeat: no-repeat;
        background-position: right 10px center;
        background-size: 10px 6px;
        cursor: pointer;
    }

    :global(textarea) {
        resize: vertical;
        min-height: 100px;
        font-family: "SF Mono", Menlo, Monaco, monospace;
        font-size: 12px;
    }

    :global(input[type="text"]:focus),
    :global(input[type="number"]:focus),
    :global(select:focus),
    :global(textarea:focus) {
        outline: none;
        background: rgba(139, 92, 246, 0.04);
    }

    :global(.field:has(.field-issue.error) input[type="text"]),
    :global(.field:has(.field-issue.error) input[type="number"]),
    :global(.field:has(.field-issue.error) textarea),
    :global(.field:has(.field-issue.error) select) {
        border-color: #b44d4d;
        background: color-mix(in srgb, #b44d4d 5%, var(--settings-input-bg));
    }

    :global(.field:has(.field-issue.warning) input[type="text"]),
    :global(.field:has(.field-issue.warning) input[type="number"]),
    :global(.field:has(.field-issue.warning) textarea),
    :global(.field:has(.field-issue.warning) select) {
        border-color: #a8793f;
        background: color-mix(in srgb, #a8793f 5%, var(--settings-input-bg));
    }

    :global(.section-description) {
        margin: 0 0 14px;
        color: #6b7280;
        font-size: 13px;
        line-height: 1.45;
    }

    footer {
        padding: 16px 24px;
        background: var(--settings-surface);
        backdrop-filter: blur(40px);
        -webkit-backdrop-filter: blur(40px);
        border-top: 1px solid var(--settings-border);
        display: flex;
        gap: 10px;
        align-items: center;
    }

    .spacer {
        flex: 1;
    }

    :global(button) {
        padding: 8px 16px;
        border-radius: 6px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        border: none;
        transition: all 0.2s ease;
    }

    :global(button.secondary) {
        background: var(--settings-btn-bg);
        color: var(--settings-btn-text);
        border: none;
    }

    :global(button.secondary:hover) {
        background: rgba(128, 128, 128, 0.18);
    }

    :global(.path-picker) {
        display: flex;
        gap: 8px;
    }

    :global(.path-picker input) {
        flex: 1;
        font-family: "SF Mono", Menlo, Monaco, monospace;
        font-size: 12px;
        background: var(--settings-input-bg);
    }

    :global(.path-picker button) {
        padding: 8px 16px;
        white-space: nowrap;
    }

    @media (max-width: 860px) {
        main {
            overflow-y: auto;
            padding: 0;
        }

        .settings-layout {
            grid-template-columns: 1fr;
            height: auto;
            gap: 0;
        }

        .settings-sidebar {
            overflow-x: auto;
            margin: 0;
            background: rgba(255, 255, 255, 0.95);
            border-right: none;
            padding: 0px;
            gap: 0;
        }

        .settings-nav {
            position: static;
            flex-direction: row;
            align-items: stretch;
            padding: 0px 0;
            gap: 0px;
        }

        .nav-item {
            width: auto;
            flex-shrink: 0;
            padding: 10px 14px;
            border-bottom: 2px solid transparent;
            border-left: none;
            border-radius: 0;
        }

        .settings-content {
            overflow: visible;
            padding: 20px 20px;
            background: #f2f2f2;
        }

        .nav-item.active {
            background: rgba(0, 0, 0, 0.05);
        }

        .nav-item:hover {
            background: rgba(0, 0, 0, 0.03);
        }

        .nav-item.active:hover {
            background: rgba(0, 0, 0, 0.05);
        }

        :global(.settings-panel) {
            padding-bottom: 0;
            background: transparent;
        }
    }
</style>
