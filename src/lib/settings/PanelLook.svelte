<script>
    import Section from "./Section.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import { getSystemFonts } from "../utils.js";

    export let settings;
    export let showStatus;
    export let validation = {};

    const systemFonts = getSystemFonts();

    $: void showStatus;
</script>

<div class="settings-panel">
    <Section title="Glass Effect">
        <div class="field">
            <label for="border_radius">Corner Radius: {settings.border_radius}px</label>
            <input
                type="range"
                id="border_radius"
                bind:value={settings.border_radius}
                min="0"
                max="20"
            />
            <FieldIssue issue={validation.border_radius} />
        </div>
        <div class="field">
            <label for="window_blur">Background Blur: {settings.window_blur ?? 80}px</label>
            <input
                type="range"
                id="window_blur"
                bind:value={settings.window_blur}
                min="0"
                max="200"
            />
            <FieldIssue issue={validation.window_blur} />
            <small>Blurs what is behind the window.</small>
        </div>
        <div class="field">
            <label for="window_saturation"
                >Background Saturation: {settings.window_saturation ?? 200}%</label
            >
            <input
                type="range"
                id="window_saturation"
                bind:value={settings.window_saturation}
                min="0"
                max="300"
            />
            <FieldIssue issue={validation.window_saturation} />
            <small>Adjusts the color intensity behind the window.</small>
        </div>
        <div class="field">
            <label for="window_brightness"
                >Background Brightness: {settings.window_brightness ?? 0}</label
            >
            <input
                type="range"
                id="window_brightness"
                bind:value={settings.window_brightness}
                min="-100"
                max="100"
            />
            <FieldIssue issue={validation.window_brightness} />
            <small>Darkens or brightens what is behind the window.</small>
        </div>
    </Section>

    <Section title="Color Overlay">
        <div class="field">
            <label for="overlay_color">Overlay Color</label>
            <div class="color-input">
                <input
                    type="color"
                    id="overlay_color"
                    bind:value={settings.overlay_color}
                />
                <input
                    type="text"
                    bind:value={settings.overlay_color}
                    pattern="^#[0-9A-Fa-f]{6}$"
                />
            </div>
            <FieldIssue issue={validation.overlay_color} />
        </div>
        <div class="field">
            <label for="overlay_strength"
                >Overlay Strength: {settings.overlay_strength ?? 10}%</label
            >
            <input
                type="range"
                id="overlay_strength"
                bind:value={settings.overlay_strength}
                min="0"
                max="100"
            />
            <FieldIssue issue={validation.overlay_strength} />
            <small>0% = no color overlay, 100% = solid overlay color.</small>
        </div>
    </Section>

    <Section title="Accent & Links">
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
            <FieldIssue issue={validation.accent_color} />
            <small>Used for active tabs, checkboxes, caret, and highlights</small>
        </div>
        <div class="field">
            <label for="internal_link_color">Internal Links [[wikilinks]]</label>
            <div class="color-input">
                <input
                    type="color"
                    id="internal_link_color"
                    bind:value={settings.internal_link_color}
                />
                <input
                    type="text"
                    bind:value={settings.internal_link_color}
                    pattern="^#[0-9A-Fa-f]{6}$"
                />
            </div>
            <FieldIssue issue={validation.internal_link_color} />
        </div>
        <div class="field">
            <label for="external_link_color">External Links [text](url)</label>
            <div class="color-input">
                <input
                    type="color"
                    id="external_link_color"
                    bind:value={settings.external_link_color}
                />
                <input
                    type="text"
                    bind:value={settings.external_link_color}
                    pattern="^#[0-9A-Fa-f]{6}$"
                />
            </div>
            <FieldIssue issue={validation.external_link_color} />
        </div>
    </Section>

    <Section title="Typography">
        <div class="field">
            <label for="font_family">Font Family</label>
            <select id="font_family" bind:value={settings.font_family}>
                {#each systemFonts as font}
                    <option value={font}>{font}</option>
                {/each}
            </select>
        </div>
        <div class="field">
            <label for="font_size">Font Size: {settings.font_size}px</label>
            <input
                type="range"
                id="font_size"
                bind:value={settings.font_size}
                min="10"
                max="24"
            />
            <FieldIssue issue={validation.font_size} />
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
            <FieldIssue issue={validation.text_color} />
            <small>Default: #ffffff</small>
        </div>
    </Section>
</div>

<style>
    input[type="range"] {
        width: 100%;
        margin: 4px 0;
        -webkit-appearance: none;
        appearance: none;
        background: transparent;
    }

    input[type="range"]::-webkit-slider-runnable-track {
        height: 4px;
        border-radius: 2px;
        background: var(--settings-input-border, rgba(0, 0, 0, 0.15));
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: var(--settings-text, #1a1a1a);
        margin-top: -5px;
        border: none;
        cursor: pointer;
    }

    input[type="range"]:focus {
        outline: none;
    }

    input[type="range"]:focus::-webkit-slider-thumb {
        box-shadow: 0 0 0 2px var(--settings-input-bg, white),
            0 0 0 4px rgba(139, 92, 246, 0.3);
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
        border: 1px solid var(--settings-input-border, #ddd);
        border-radius: 4px;
        cursor: pointer;
    }

    .color-input input[type="text"] {
        flex: 1;
        font-family: monospace;
    }
</style>
