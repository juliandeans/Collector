<script>
    import { invoke } from "@tauri-apps/api/core";
    import { normalizeDelayValue } from "./delay-utils.js";

    export let settings;
    export let showStatus;

    let showAppPicker = false;
    let runningApps = [];
    let filteredRunningApps = [];
    let appPickerQuery = "";

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

    function normalizeDelayField(field, fallback = 1000) {
        settings[field] = normalizeDelayValue(settings[field], fallback);
        settings = { ...settings };
    }

    $: filteredRunningApps = filterApps(runningApps, appPickerQuery);
</script>

<div class="settings-panel">
    <section>
        <h2>Edge Detection</h2>
        <p class="section-description">
            Enable or disable screen-edge activation for the note and reader
            windows.
        </p>

        <div class="field">
            <label class="checkbox">
                <input
                    type="checkbox"
                    bind:checked={settings.edge_detection_enabled}
                />
                Edge Detection enabled
            </label>
            <small>Panels open when moving mouse to screen edges</small>
        </div>
    </section>

    <section>
        <h2>Open Delays</h2>
        <p class="section-description">
            Set separate edge-trigger delays for each window. If disabled, the
            standard trigger timing is used.
        </p>

        <div class="delay-grid">
            <div class="delay-card">
                <div class="delay-card-title">Note Window</div>
                <div class="delay-toggle-row">
                    <label class="checkbox compact-checkbox">
                        <input
                            type="checkbox"
                            bind:checked={settings.note_edge_open_delay_enabled}
                        />
                        Open Delay
                    </label>

                    {#if settings.note_edge_open_delay_enabled}
                        <label class="delay-input">
                            <input
                                type="number"
                                bind:value={settings.note_edge_open_delay_ms}
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
                    Wait this long before the note window opens when touching
                    the edge.
                </small>
            </div>

            <div class="delay-card">
                <div class="delay-card-title">Reader Window</div>
                <div class="delay-toggle-row">
                    <label class="checkbox compact-checkbox">
                        <input
                            type="checkbox"
                            bind:checked={settings.reader_edge_open_delay_enabled}
                        />
                        Open Delay
                    </label>

                    {#if settings.reader_edge_open_delay_enabled}
                        <label class="delay-input">
                            <input
                                type="number"
                                bind:value={settings.reader_edge_open_delay_ms}
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
                    Wait this long before the reader opens when touching the
                    edge.
                </small>
            </div>
        </div>
    </section>

    <section>
        <h2>Modifier Keys</h2>
        <p class="section-description">
            Hold these keys while touching the edge to open a panel. Leave all
            unchecked to open without any modifier.
        </p>

        <div class="field">
            <div class="modifier-grid">
                {#each ["cmd", "option", "shift", "ctrl"] as mod}
                    <label class="checkbox modifier-checkbox">
                        <input
                            type="checkbox"
                            checked={settings.edge_modifier_keys?.includes(mod)}
                            on:change={(event) => {
                                const keys = settings.edge_modifier_keys ?? [];
                                if (event.currentTarget.checked) {
                                    settings.edge_modifier_keys = [...keys, mod];
                                } else {
                                    settings.edge_modifier_keys = keys.filter(
                                        (key) => key !== mod,
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
            Edge detection is paused when these apps are in focus.
        </p>

        <div class="field">
            {#if (settings.edge_excluded_apps ?? []).length > 0}
                <ul class="exclusion-list">
                    {#each settings.edge_excluded_apps as app}
                        <li class="exclusion-item">
                            <span class="exclusion-name">{app}</span>
                            <button
                                class="exclusion-remove"
                                type="button"
                                on:click={() => removeExcludedApp(app)}
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
                                on:click={() => addExcludedApp(app)}
                                disabled={settings.edge_excluded_apps?.includes(
                                    app,
                                )}
                            >
                                {app}
                                {#if settings.edge_excluded_apps?.includes(app)}
                                    <span class="app-added">✓</span>
                                {/if}
                            </button>
                        {/each}
                    </div>
                </div>
            {/if}
        </div>
    </section>
</div>

<style>
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

    @media (max-width: 860px) {
        .delay-grid {
            grid-template-columns: 1fr;
        }

        .delay-toggle-row {
            align-items: flex-start;
            flex-direction: column;
        }
    }
</style>
