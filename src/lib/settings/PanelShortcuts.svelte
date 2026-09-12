<script>
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import { handleShortcutKeyDown as captureShortcutKeyDown } from "./shortcut-utils.js";

    let {
        settings = $bindable(),
        validation = {},
        onChange = () => {}
    } = $props();

    function handleShortcutKeyDown(event, field) {
        captureShortcutKeyDown(event, field, settings, onChange);
    }
</script>

<div class="settings-panel">
    <Section title="Capture Window">
        <div class="field">
            <label for="global_shortcut">Open Capture Window</label>
            <input
                class="shortcut-input"
                type="text"
                id="global_shortcut"
                bind:value={settings.global_shortcut}
                placeholder="Cmd+Shift+N"
                onkeydown={(e) => handleShortcutKeyDown(e, "global_shortcut")}
            />
            <FieldIssue issue={validation.global_shortcut} />
        </div>
        <div class="field">
            <Checkbox bind:checked={settings.global_shortcut_closes_window}>
                Use the same shortcut to close the note window
            </Checkbox>
        </div>
        {#if !settings.global_shortcut_closes_window}
            <div class="field">
                <label for="global_close_shortcut">Close Capture Window</label>
                <input
                    class="shortcut-input"
                    type="text"
                    id="global_close_shortcut"
                    bind:value={settings.global_close_shortcut}
                    placeholder="Optional"
                    onkeydown={(e) =>
                        handleShortcutKeyDown(e, "global_close_shortcut")}
                />
                <FieldIssue issue={validation.global_close_shortcut} />
                <small>Leave empty to disable closing via shortcut</small>
            </div>
        {/if}
    </Section>

    <Section title="Capture Mode">
        <div class="field">
            <label for="capture_mode_shortcut">Toggle Collect Mode</label>
            <input
                class="shortcut-input"
                type="text"
                id="capture_mode_shortcut"
                bind:value={settings.capture_mode_shortcut}
                placeholder="Cmd+Shift+M"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "capture_mode_shortcut")}
            />
            <FieldIssue issue={validation.capture_mode_shortcut} />
        </div>
    </Section>

    <Section title="Save Actions">
        <div class="field">
            <label for="save_to_daily_shortcut">Save to Daily Note</label>
            <input
                class="shortcut-input"
                type="text"
                id="save_to_daily_shortcut"
                bind:value={settings.save_to_daily_shortcut}
                placeholder="Cmd+Enter"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "save_to_daily_shortcut")}
            />
            <FieldIssue issue={validation.save_to_daily_shortcut} />
        </div>
        <div class="field">
            <label for="save_as_note_shortcut">Create New Note</label>
            <input
                class="shortcut-input"
                type="text"
                id="save_as_note_shortcut"
                bind:value={settings.save_as_note_shortcut}
                placeholder="Cmd+Shift+Enter"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "save_as_note_shortcut")}
            />
            <FieldIssue issue={validation.save_as_note_shortcut} />
        </div>
        <div class="field">
            <label for="append_to_note_shortcut">Append to Note</label>
            <input
                class="shortcut-input"
                type="text"
                id="append_to_note_shortcut"
                bind:value={settings.append_to_note_shortcut}
                placeholder="Cmd+Option+Enter"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "append_to_note_shortcut")}
            />
            <FieldIssue issue={validation.append_to_note_shortcut} />
        </div>
    </Section>

    <Section title="Reader Window">
        <div class="field">
            <label for="reader_shortcut">Open Reader Window</label>
            <input
                class="shortcut-input"
                type="text"
                id="reader_shortcut"
                bind:value={settings.reader_shortcut}
                placeholder="Cmd+Shift+R"
                onkeydown={(e) => handleShortcutKeyDown(e, "reader_shortcut")}
            />
            <FieldIssue issue={validation.reader_shortcut} />
        </div>
        <div class="field">
            <Checkbox bind:checked={settings.reader_shortcut_closes_window}>
                Use the same shortcut to close the reader window
            </Checkbox>
        </div>
        {#if !settings.reader_shortcut_closes_window}
            <div class="field">
                <label for="reader_close_shortcut">Close Reader Window</label>
                <input
                    class="shortcut-input"
                    type="text"
                    id="reader_close_shortcut"
                    bind:value={settings.reader_close_shortcut}
                    placeholder="Optional"
                    onkeydown={(e) =>
                        handleShortcutKeyDown(e, "reader_close_shortcut")}
                />
                <FieldIssue issue={validation.reader_close_shortcut} />
                <small>Leave empty to disable closing via shortcut</small>
            </div>
        {/if}
        <div class="field">
            <label for="reader_open_in_obsidian_shortcut">Open Note in Obsidian</label>
            <input
                class="shortcut-input"
                type="text"
                id="reader_open_in_obsidian_shortcut"
                bind:value={settings.reader_open_in_obsidian_shortcut}
                placeholder="Cmd+Shift+O"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "reader_open_in_obsidian_shortcut")}
            />
            <FieldIssue issue={validation.reader_open_in_obsidian_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
        <div class="field">
            <label for="reader_navigate_back_shortcut">Navigate Back</label>
            <input
                class="shortcut-input"
                type="text"
                id="reader_navigate_back_shortcut"
                bind:value={settings.reader_navigate_back_shortcut}
                placeholder="Optional"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "reader_navigate_back_shortcut")}
            />
            <FieldIssue issue={validation.reader_navigate_back_shortcut} />
            <small>Leave empty to disable</small>
        </div>
        <div class="field">
            <label for="reader_command_palette_shortcut">Open Command Palette</label>
            <input
                class="shortcut-input"
                type="text"
                id="reader_command_palette_shortcut"
                bind:value={settings.reader_command_palette_shortcut}
                placeholder="Cmd+P"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "reader_command_palette_shortcut")}
            />
            <FieldIssue issue={validation.reader_command_palette_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
        <div class="field">
            <label for="reader_next_tab_shortcut">Next Tab</label>
            <input
                class="shortcut-input"
                type="text"
                id="reader_next_tab_shortcut"
                bind:value={settings.reader_next_tab_shortcut}
                placeholder="Option+Right"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "reader_next_tab_shortcut")}
            />
            <FieldIssue issue={validation.reader_next_tab_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
        <div class="field">
            <label for="reader_previous_tab_shortcut">Previous Tab</label>
            <input
                class="shortcut-input"
                type="text"
                id="reader_previous_tab_shortcut"
                bind:value={settings.reader_previous_tab_shortcut}
                placeholder="Option+Left"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "reader_previous_tab_shortcut")}
            />
            <FieldIssue issue={validation.reader_previous_tab_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
    </Section>

    <Section title="Copy Text to Collector">
        <div class="field">
            <label for="capture_text_shortcut">Shortcut</label>
            <input
                class="shortcut-input"
                type="text"
                id="capture_text_shortcut"
                bind:value={settings.capture_text_shortcut}
                placeholder="Cmd+Shift+C"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "capture_text_shortcut")}
            />
            <FieldIssue issue={validation.capture_text_shortcut} />
        </div>
        <div class="info-note">
            <div class="info-note-title">Accessibility Permission</div>
            <p>
                Required for "Copy Text to Collector". Enable Collector in
                <strong
                    >System Settings → Privacy & Security → Accessibility</strong
                >, then restart the app.
            </p>
        </div>
    </Section>

    <Section title="OCR / Image to Text">
        <div class="field">
            <label for="ocr_shortcut">Shortcut</label>
            <input
                class="shortcut-input"
                type="text"
                id="ocr_shortcut"
                bind:value={settings.ocr_shortcut}
                placeholder="Cmd+Shift+T"
                onkeydown={(e) =>
                    handleShortcutKeyDown(e, "ocr_shortcut")}
            />
            <FieldIssue issue={validation.ocr_shortcut} />
            <small>Extracts text from clipboard image at cursor</small>
        </div>
        <div class="field">
            <label for="ocr_drag_modifier">Drag Modifier</label>
            <select
                id="ocr_drag_modifier"
                bind:value={settings.ocr_drag_modifier}
                onchange={onChange}
            >
                <option value="Option">Option</option>
                <option value="Shift">Shift</option>
                <option value="Control">Control</option>
                <option value="Command">Command</option>
            </select>
            <FieldIssue issue={validation.ocr_drag_modifier} />
            <small>Hold this modifier key while dropping an image to extract text</small>
        </div>
    </Section>
</div>

<style>
    .shortcut-input {
        width: 280px;
        max-width: 100%;
    }

    .info-note {
        margin-top: 14px;
        padding: 13px 14px;
        border-radius: 12px;
        border: 1px solid var(--settings-border, rgba(15, 23, 42, 0.08));
        background: rgba(128, 128, 128, 0.05);
    }

    .info-note-title {
        font-size: 12px;
        font-weight: 600;
        color: var(--settings-label, #111827);
        margin-bottom: 4px;
    }

    .info-note p {
        margin: 0;
        color: var(--settings-text-secondary, #6b7280);
        font-size: 12px;
        line-height: 1.45;
    }
</style>
