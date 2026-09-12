<script>
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import SegmentedControl from "./SegmentedControl.svelte";
    import SliderField from "./SliderField.svelte";
    import { getSystemFonts } from "../utils.js";

    let {
        settings = $bindable(),
        validation = {},
        onChange = () => {},
    } = $props();

    const systemFonts = getSystemFonts();

    const BLUR_OPTIONS = [
        { label: "Off", value: 0 },
        { label: "Light", value: 50 },
        { label: "Medium", value: 100 },
        { label: "Strong", value: 150 },
    ];
    const blurSelected = $derived(
        BLUR_OPTIONS[
            Math.min(Math.max(Math.round((settings.window_blur ?? 50) / 50), 0), 3)
        ].value,
    );
</script>

<div class="settings-panel">
    <Section title="Glass Effect">
        <div class="field">
            <div class="field-label">Background Blur</div>
            <SegmentedControl
                id="window_blur"
                options={BLUR_OPTIONS}
                value={blurSelected}
                onSelect={(v) => {
                    settings.window_blur = v;
                    onChange();
                }}
            />
            <FieldIssue issue={validation.window_blur} />
            <small
                >Blurs what is behind the window. macOS only offers a few
                fixed blur looks, not a stepless amount.</small
            >
        </div>
        <div class="field">
            <SliderField
                id="window_saturation"
                label="Background Saturation"
                bind:value={settings.window_saturation}
                min={0}
                max={300}
                formatValue={(v) => `${v ?? 200}%`}
            />
            <FieldIssue issue={validation.window_saturation} />
            <small>Adjusts the color intensity behind the window.</small>
        </div>
        <div class="field">
            <SliderField
                id="window_brightness"
                label="Background Brightness"
                bind:value={settings.window_brightness}
                min={-100}
                max={100}
                formatValue={(v) => `${v ?? 0}`}
                showZeroTick
            />
            <FieldIssue issue={validation.window_brightness} />
            <small>Darkens or brightens what is behind the window.</small>
        </div>
        <div class="field">
            <label for="overlay_color">Tint Color</label>
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
            <SliderField
                id="overlay_strength"
                label="Tint Strength"
                bind:value={settings.overlay_strength}
                min={0}
                max={100}
                formatValue={(v) => `${v ?? 10}%`}
            />
            <FieldIssue issue={validation.overlay_strength} />
            <small>0% = no tint, 100% = solid tint color.</small>
        </div>
    </Section>

    <Section title="Window Shape">
        <div class="field">
            <SliderField
                id="border_radius"
                label="Corner Radius"
                bind:value={settings.border_radius}
                min={0}
                max={20}
                formatValue={(v) => `${v}px`}
            />
            <FieldIssue issue={validation.border_radius} />
        </div>
        <div class="field">
            <Checkbox bind:checked={settings.round_all_corners}>
                Round all four corners
            </Checkbox>
            <small>
                Off by default: the two corners facing the window's docked
                screen edge stay square, only the corners facing the
                screen's interior are rounded.
            </small>
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
            <SliderField
                id="font_size"
                label="Font Size"
                bind:value={settings.font_size}
                min={10}
                max={24}
                formatValue={(v) => `${v}px`}
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
