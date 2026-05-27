<script>
    import { getSystemFonts } from "../utils.js";

    export let settings;
    export let showStatus;

    const systemFonts = getSystemFonts();

    $: void showStatus;
</script>

<div class="settings-panel">
    <section class="panel-intro">
        <h2>Look & Feel</h2>
        <p class="section-description">
            Shared appearance settings for the capture and reader windows.
        </p>
    </section>

    <section>
        <h2>Window Surface</h2>
        <p class="section-description">
            Shape the glass surface and background treatment for both floating
            windows.
        </p>

        <div class="field">
            <label for="border_radius">Corner Radius: {settings.border_radius}px</label>
            <input
                type="range"
                id="border_radius"
                bind:value={settings.border_radius}
                min="0"
                max="12"
            />
        </div>
        <div class="field">
            <label for="background_color">Background Color</label>
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
                >Transparency: {settings.window_transparency ?? 55}%</label
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
            <label for="window_blur">Blur: {settings.window_blur ?? 80}px</label>
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
                >Saturation: {settings.window_saturation ?? 200}%</label
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
                >Brightness: {(settings.window_brightness ?? 0 > 0)
                    ? ""
                    : ""}{settings.window_brightness ?? 0}</label
            >
            <input
                type="range"
                id="window_brightness"
                bind:value={settings.window_brightness}
                min="-100"
                max="100"
            />
            <small>Brightens dark areas or darkens light areas</small>
        </div>
    </section>

    <section>
        <h2>Accent & Links</h2>
        <p class="section-description">
            Choose the highlight color and how links are rendered in both
            windows.
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
        </div>
    </section>

    <section>
        <h2>Typography</h2>
        <p class="section-description">
            Set the font family, size and default text color for both windows.
        </p>

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

<style>
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
</style>
