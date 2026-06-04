<script>
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";
    import AppPicker from "./AppPicker.svelte";
    import { normalizeDelayValue } from "./delay-utils.js";
    import FieldIssue from "./FieldIssue.svelte";

    export let settings;
    export let validation = {};
    export let onChange = () => {};

    let showAppPicker = false;

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

    function toggleAppPicker() {
        showAppPicker = !showAppPicker;
    }

    function addExcludedApp(app) {
        const current = settings.edge_excluded_apps ?? [];
        if (current.includes(app)) return;
        settings.edge_excluded_apps = [...current, app];
        settings = { ...settings };
        onChange();
    }

    function removeExcludedApp(app) {
        settings.edge_excluded_apps = (
            settings.edge_excluded_apps ?? []
        ).filter((entry) => entry !== app);
        settings = { ...settings };
        onChange();
    }

    function toggleModifier(field, mod) {
        const keys = settings[field] ?? [];
        if (keys.includes(mod)) {
            settings[field] = keys.filter((key) => key !== mod);
        } else {
            settings[field] = [...keys, mod];
        }
        settings = { ...settings };
        onChange();
    }

    function normalizeDelayField(field, fallback = 1000) {
        settings[field] = normalizeDelayValue(settings[field], fallback);
        settings = { ...settings };
        onChange();
    }

    $: showConflictHint =
        settings.edge_detection_enabled &&
        settings.reader_edge_enabled &&
        settings.edge_side === settings.reader_edge_side;
</script>

<div class="settings-panel">
    <Section title="Capture Window">
        <div class="field">
            <Checkbox bind:checked={settings.edge_detection_enabled}>
                Open from screen edge
            </Checkbox>
            <small>Open capture window when cursor touches screen edge</small>
        </div>

        <div class="field">
            <label for="edge_side">Screen edge</label>
            <select id="edge_side" bind:value={settings.edge_side}>
                <option value="right">Right side</option>
                <option value="left">Left side</option>
            </select>
            <FieldIssue issue={validation.edge_side} />
        </div>

        <div class="field">
            <div class="field-label">Modifier keys</div>
            <div class="modifier-grid">
                {#each ["cmd", "option", "shift", "ctrl"] as mod}
                    <div class="modifier-checkbox">
                        <Checkbox
                            checked={settings.edge_modifier_keys?.includes(mod)}
                            on:change={() =>
                                toggleModifier("edge_modifier_keys", mod)}
                        >
                            {modifierLabel(mod)}
                        </Checkbox>
                    </div>
                {/each}
            </div>
            <small>Hold modifier keys while touching edge</small>
        </div>

        <div class="field">
            <div class="delay-toggle-row">
                <Checkbox bind:checked={settings.note_edge_open_delay_enabled}>
                    Open delay
                </Checkbox>

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
            <FieldIssue issue={validation.note_edge_open_delay_ms} />
            <small>Delay before window opens when touching edge</small>
        </div>
    </Section>

    <Section title="Reader Window">
        <div class="field">
            <Checkbox bind:checked={settings.reader_edge_enabled}>
                Open from screen edge
            </Checkbox>
            <small>Open reader window when cursor touches screen edge</small>
        </div>

        <div class="field">
            <label for="reader_edge_side">Screen edge</label>
            <select id="reader_edge_side" bind:value={settings.reader_edge_side}>
                <option value="left">Left side</option>
                <option value="right">Right side</option>
            </select>
            <FieldIssue issue={validation.reader_edge_side} />
        </div>

        <div class="field">
            <div class="field-label">Modifier keys</div>
            <div class="modifier-grid">
                {#each ["cmd", "option", "shift", "ctrl"] as mod}
                    <div class="modifier-checkbox">
                        <Checkbox
                            checked={settings.reader_edge_modifier_keys?.includes(
                                mod,
                            )}
                            on:change={() =>
                                toggleModifier(
                                    "reader_edge_modifier_keys",
                                    mod,
                                )}
                        >
                            {modifierLabel(mod)}
                        </Checkbox>
                    </div>
                {/each}
            </div>
            <small>Hold modifier keys while touching edge</small>
        </div>

        <div class="field">
            <div class="delay-toggle-row">
                <Checkbox bind:checked={settings.reader_edge_open_delay_enabled}>
                    Open delay
                </Checkbox>

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
            <FieldIssue issue={validation.reader_edge_open_delay_ms} />
            <small>Delay before window opens when touching edge</small>
        </div>
    </Section>

    {#if showConflictHint}
        <small class="edge-conflict-hint">
            When Capture and Reader use the same edge, Reader takes priority.
        </small>
    {/if}

    <Section title="Excluded Apps">
        <small class="global-note">
            Applies to both Capture and Reader edge activation
        </small>

        <style>
            :global(.section-body > small.edge-conflict-hint) {
                display: block;
                font-size: 11px;
                font-weight: 500;
                color: var(--settings-accent, #8b5cf6);
                margin-top: 0;
                margin-bottom: 4px;
                padding: 6px 14px;
                background: rgba(139, 92, 246, 0.08);
                border-radius: 6px;
            }

            :global(.section-body > small.global-note) {
                display: block;
                font-size: 11px;
                font-weight: 400;
                color: var(--settings-text-secondary, #6b7280);
                margin-top: 4px;
                margin-bottom: 8px;
                padding-left: 14px;
            }
        </style>

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

            <AppPicker
                show={showAppPicker}
                excludedApps={settings.edge_excluded_apps ?? []}
                on:add={(event) => addExcludedApp(event.detail)}
                on:close={() => {
                    showAppPicker = false;
                }}
            />
        </div>
    </Section>
</div>

<style>
    .modifier-grid {
        display: grid;
        grid-template-columns: 1fr 1fr;
        gap: 8px;
        margin-top: 6px;
    }

    .delay-toggle-row {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
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
        padding: 8px 10px;
        border: 1.5px solid var(--settings-input-border, rgba(0, 0, 0, 0.1));
        border-radius: 8px;
        transition: all 0.15s;
    }

    .modifier-checkbox :global(.checkbox-wrapper) {
        width: 100%;
    }

    .modifier-checkbox:has(.checkbox-wrapper input:checked) {
        background: color-mix(
            in srgb,
            var(--settings-accent, #8b5cf6) 6%,
            transparent
        );
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
        color: var(--settings-btn-text, inherit);
    }

    @media (max-width: 860px) {
        .delay-toggle-row {
            align-items: flex-start;
            flex-direction: column;
        }
    }
</style>
