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
    import { lightTheme, darkTheme } from "./lib/settings/themes.js";
    import "./lib/settings/settings.css";

    let settings = $state({ ...defaultSettings });
    let vaultNotes = $state([]);
    let isLoaded = false;
    let statusMessage = $state("");
    let statusType = $state("");
    let activePanel = $state("vault");
    let statusTimeout;

    const settingsPanels = [
        { id: "vault", label: "Vault" },
        { id: "capture", label: "Capture" },
        { id: "reader", label: "Reader" },
        { id: "look", label: "Appearance" },
        { id: "shortcuts", label: "Shortcuts" },
        { id: "activation", label: "Activation" },
        { id: "images", label: "Images" },
    ];

    // ── theme (system dark/light) ─────────────────────────

    let prefersDark = $state(false);

    function updateTheme(mq) {
        prefersDark = mq.matches;
    }

    onMount(() => {
        const mq = window.matchMedia("(prefers-color-scheme: dark)");
        updateTheme(mq);
        mq.addEventListener("change", updateTheme);
    });

    let t = $derived(prefersDark ? darkTheme : lightTheme);
    let validation = $derived(validateSettings(settings));
    let fieldIssues = $derived(validation.fields);

    // ── auto-save ──────────────────────────────────────────

    let saveTimer;
    let isSaving = $state(false);
    let saveAgainRequested = false;

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
        if (isSaving) {
            saveAgainRequested = true;
            return false;
        }

        isSaving = true;
        try {
            return await performSaveNow();
        } finally {
            isSaving = false;
            if (saveAgainRequested) {
                saveAgainRequested = false;
                void performSave();
            }
        }
    }

    async function performSaveNow() {
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
                            onclick={() => (activePanel = panel.id)}
                        >
                            <span class="nav-item-label">{panel.label}</span>
                        </button>
                    {/each}
                </nav>
            </aside>

            <div class="settings-content" oninput={scheduleAutoSave}>
                {#if activePanel === "vault"}
                    <PanelVault bind:settings {showStatus} validation={fieldIssues} onChange={scheduleAutoSave} />
                {:else if activePanel === "capture"}
                    <PanelCapture bind:settings {showStatus} validation={fieldIssues} />
                {:else if activePanel === "reader"}
                    <PanelReader bind:settings validation={fieldIssues} {vaultNotes} onChange={scheduleAutoSave} />
                {:else if activePanel === "look"}
                    <PanelLook bind:settings validation={fieldIssues} onChange={scheduleAutoSave} />
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
        <button class="secondary" onclick={handleReset}>Reset All</button>
        <div class="spacer"></div>
        <button class="secondary" onclick={handleClose}>Close</button>
    </footer>
</div>
