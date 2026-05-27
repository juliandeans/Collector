<script>
    import { invoke } from "@tauri-apps/api/core";
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
        },
        {
            id: "images",
            label: "Images",
        },
        {
            id: "look",
            label: "Look",
        },
        {
            id: "note-window",
            label: "Note Window",
        },
        {
            id: "reader-window",
            label: "Reader Window",
        },
        {
            id: "activation",
            label: "Activation",
        },
        {
            id: "shortcuts",
            label: "Shortcuts",
        },
    ];

    async function loadSettings() {
        try {
            const loaded = await invoke("load_settings");
            const normalized = {
                ...defaultSettings,
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
                        </button>
                    {/each}
                </nav>
            </aside>

            <div class="settings-content">
                {#if activePanel === "obsidian"}
                    <PanelObsidian bind:settings {showStatus} />
                {:else if activePanel === "images"}
                    <PanelImages bind:settings {showStatus} />
                {:else if activePanel === "look"}
                    <PanelLook bind:settings {showStatus} />
                {:else if activePanel === "note-window"}
                    <PanelNoteWindow bind:settings {showStatus} />
                {:else if activePanel === "reader-window"}
                    <PanelReaderWindow bind:settings {showStatus} />
                {:else if activePanel === "activation"}
                    <PanelActivation bind:settings {showStatus} />
                {:else if activePanel === "shortcuts"}
                    <PanelShortcuts bind:settings {showStatus} />
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
        background: #f2f2f2;
    }

    header {
        padding: 12px 18px;
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
        background: rgba(255, 255, 255, 0.95);
        border-right: 1px solid rgba(0, 0, 0, 0.07);
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
        color: #374151;
        text-align: left;
        transition: background 0.15s ease;
    }

    .nav-item:hover {
        background: rgba(0, 0, 0, 0.03);
    }

    .nav-item.active {
        background: rgba(0, 0, 0, 0.05);
    }

    .nav-item.active:hover {
        background: rgba(0, 0, 0, 0.05);
    }

    .nav-item-label {
        font-size: 13px;
        font-weight: 500;
        color: #111827;
    }

    .settings-content {
        min-width: 0;
        min-height: 0;
        overflow-y: auto;
        padding-left: 24px;
        padding-right: 4px;
        background: none;
    }

    :global(.settings-panel) {
        display: flex;
        flex-direction: column;
        gap: 0;
        padding-bottom: 12px;
    }

    :global(.settings-panel > section) {
        background: none;
        backdrop-filter: none;
        -webkit-backdrop-filter: none;
        border: none;
        border-radius: 0;
        box-shadow: none;
        padding: 0 0 24px 0;
        margin-top: 0px;
        padding-bottom: 12px;
        border-top: 1px solid rgba(0, 0, 0, 0.07);
    }

    :global(.settings-panel > section:first-of-type) {
        border-top: none;
        margin-top: 0;
    }

    :global(.settings-panel > section h2) {
        font-size: 14px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: #9ca3af;
        margin: 0 0 4px 0;
        padding-top: 12px;
        padding-bottom: 0px;
    }

    :global(.panel-intro) {
        background: none;
        backdrop-filter: none;
        -webkit-backdrop-filter: none;
        border: none;
        border-radius: 0;
        box-shadow: none;
        padding: 0 0 8px 0;
        margin-top: 0;
        border-top: none;
    }

    :global(.panel-intro h2) {
        font-size: 18px;
        font-weight: 600;
        color: #111827;
        text-transform: none;
        letter-spacing: -0.3px;
        margin-bottom: 6px;
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
        color: #111827;
        margin-bottom: 5px;
        padding-top: 12px;
    }

    :global(.field small) {
        display: block;
        font-size: 11px;
        font-weight: 400;
        color: #9ca3af;
        margin-top: 4px;
        padding-left: 14px;
        padding-bottom: 4px;
    }

    :global(input[type="text"]),
    :global(input[type="number"]),
    :global(select),
    :global(textarea) {
        width: 100%;
        padding: 9px 12px;
        border: 1.5px solid rgba(0, 0, 0, 0.1);
        border-radius: 6px;
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
        background: rgba(0, 0, 0, 0.01);
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
        background: rgba(0, 0, 0, 0.07);
        color: #111827;
        border: none;
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
