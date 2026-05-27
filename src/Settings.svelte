<script>
    import { invoke } from "@tauri-apps/api/core";
    import { getCurrentWindow } from "@tauri-apps/api/window";
    import { onMount } from "svelte";
    import PanelActivation from "./lib/settings/PanelActivation.svelte";
    import PanelImages from "./lib/settings/PanelImages.svelte";
    import PanelLook from "./lib/settings/PanelLook.svelte";
    import PanelNoteWindow from "./lib/settings/PanelNoteWindow.svelte";
    import PanelObsidian from "./lib/settings/PanelObsidian.svelte";
    import PanelReaderWindow from "./lib/settings/PanelReaderWindow.svelte";
    import PanelShortcuts from "./lib/settings/PanelShortcuts.svelte";
    import { normalizeDelayValue } from "./lib/settings/delay-utils.js";
    import { normalizePinnedNotes } from "./lib/settings/pinned-notes.js";
    import { defaultSettings } from "./lib/stores.js";

    let settings = { ...defaultSettings };
    let originalSettings = { ...defaultSettings };
    let isSaving = false;
    let statusMessage = "";
    let statusType = "";
    let activePanel = "obsidian";

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
                    <PanelObsidian bind:settings={settings} {showStatus} />
                {:else if activePanel === "images"}
                    <PanelImages bind:settings={settings} {showStatus} />
                {:else if activePanel === "look"}
                    <PanelLook bind:settings={settings} {showStatus} />
                {:else if activePanel === "note-window"}
                    <PanelNoteWindow bind:settings={settings} {showStatus} />
                {:else if activePanel === "reader-window"}
                    <PanelReaderWindow bind:settings={settings} {showStatus} />
                {:else if activePanel === "activation"}
                    <PanelActivation bind:settings={settings} {showStatus} />
                {:else if activePanel === "shortcuts"}
                    <PanelShortcuts bind:settings={settings} {showStatus} />
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

    :global(.settings-panel) {
        display: flex;
        flex-direction: column;
        gap: 16px;
        padding-bottom: 12px;
    }

    :global(section) {
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

    :global(section h2) {
        font-size: 15px;
        font-weight: 600;
        margin: 0 0 10px 0;
        color: #111827;
        letter-spacing: -0.24px;
    }

    :global(.panel-intro) {
        background: linear-gradient(
            155deg,
            color-mix(in srgb, var(--accent-color) 5%, white) 0%,
            rgba(255, 255, 255, 0.98) 100%
        );
    }

    :global(.panel-intro h2) {
        font-size: 16px;
        margin-bottom: 8px;
    }

    :global(.panel-intro .section-description) {
        max-width: 620px;
    }

    :global(.field) {
        margin-bottom: 12px;
    }

    :global(.field:last-child) {
        margin-bottom: 0;
    }

    :global(.field label) {
        display: block;
        font-weight: 500;
        margin-bottom: 4px;
    }

    .field-label {
        display: block;
        font-weight: 500;
        margin-bottom: 4px;
    }

    :global(.field small) {
        display: block;
        color: #888;
        font-size: 11px;
        margin-top: 4px;
    }

    :global(input[type="text"]),
    :global(input[type="number"]),
    :global(select),
    :global(textarea) {
        width: 100%;
        padding: 9px 12px;
        border: 1.5px solid rgba(0, 0, 0, 0.1);
        border-radius: 8px;
        font-size: 13px;
        background: white;
        transition: all 0.2s ease;
        font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
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
        box-shadow: inset 0px 0px 6px 1px rgba(15, 23, 42, 0.06);
        background: linear-gradient(
            170deg,
            rgba(139, 92, 246, 0.05) 20%,
            rgba(255, 255, 255, 0.04) 80%
        );
    }

    :global(.section-description) {
        margin: 0 0 14px;
        color: #6b7280;
        font-size: 13px;
        line-height: 1.45;
    }

    :global(.checkbox) {
        display: flex;
        align-items: center;
        gap: 6px;
        cursor: pointer;
        font-weight: normal;
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

    :global(button) {
        padding: 8px 16px;
        border-radius: 6px;
        font-size: 13px;
        font-weight: 500;
        cursor: pointer;
        border: none;
        transition: all 0.2s ease;
    }

    :global(button.primary) {
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

    :global(button.primary:hover:not(:disabled)) {
        background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
        box-shadow: 0 4px 12px
            color-mix(in srgb, var(--accent-color, #8b5cf6) 30%, transparent);
        transform: translateY(-1px);
    }

    :global(button.primary:disabled) {
        opacity: 0.75;
        background: linear-gradient(135deg, #a78bfa 0%, #8b5cf6 100%);
        color: rgba(255, 255, 255, 0.96);
        cursor: not-allowed;
    }

    :global(button.secondary) {
        background: #e5e5e5;
        color: #333;
    }

    :global(button.secondary:hover) {
        background: #d5d5d5;
    }

    :global(.path-picker) {
        display: flex;
        gap: 8px;
    }

    :global(.path-picker input) {
        flex: 1;
        font-family: "SF Mono", Menlo, Monaco, monospace;
        font-size: 12px;
        background: #f9f9f9;
    }

    :global(.path-picker button) {
        padding: 8px 16px;
        white-space: nowrap;
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

    }
</style>
