<script>
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import ShortcutInput from "./ShortcutInput.svelte";

    let {
        settings = $bindable(),
        validation = {},
        onChange = () => {}
    } = $props();
</script>

<div class="settings-panel">
    <Section title="Capture Window">
        <div class="field">
            <label for="global_shortcut">Open Capture Window</label>
            <ShortcutInput
                id="global_shortcut"
                bind:value={settings.global_shortcut}
                placeholder="Cmd+Shift+N"
                {onChange}
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
                <ShortcutInput
                    id="global_close_shortcut"
                    bind:value={settings.global_close_shortcut}
                    placeholder="Optional"
                    {onChange}
                />
                <FieldIssue issue={validation.global_close_shortcut} />
                <small>Leave empty to disable closing via shortcut</small>
            </div>
        {/if}
        <div class="field">
            <label for="capture_mode_shortcut">Toggle Collect Mode</label>
            <ShortcutInput
                id="capture_mode_shortcut"
                bind:value={settings.capture_mode_shortcut}
                placeholder="Cmd+Shift+M"
                {onChange}
            />
            <FieldIssue issue={validation.capture_mode_shortcut} />
        </div>
        <div class="field">
            <label for="persist_capture_draft_shortcut">Toggle Keep Unsaved Text</label>
            <ShortcutInput
                id="persist_capture_draft_shortcut"
                bind:value={settings.persist_capture_draft_shortcut}
                placeholder="Cmd+Shift+D"
                {onChange}
            />
            <FieldIssue issue={validation.persist_capture_draft_shortcut} />
        </div>
        <div class="field">
            <label for="clear_capture_shortcut">Clear Capture Window</label>
            <ShortcutInput
                id="clear_capture_shortcut"
                bind:value={settings.clear_capture_shortcut}
                placeholder="Cmd+Backspace"
                {onChange}
            />
            <FieldIssue issue={validation.clear_capture_shortcut} />
            <small>Clears text, images and the persisted draft</small>
        </div>

        <div class="divider"></div>

        <div class="field">
            <label for="save_to_daily_shortcut">Save to Daily Note</label>
            <ShortcutInput
                id="save_to_daily_shortcut"
                bind:value={settings.save_to_daily_shortcut}
                placeholder="Cmd+Enter"
                {onChange}
            />
            <FieldIssue issue={validation.save_to_daily_shortcut} />
        </div>
        <div class="field">
            <label for="save_as_note_shortcut">Create New Note</label>
            <ShortcutInput
                id="save_as_note_shortcut"
                bind:value={settings.save_as_note_shortcut}
                placeholder="Cmd+Shift+Enter"
                {onChange}
            />
            <FieldIssue issue={validation.save_as_note_shortcut} />
        </div>
        <div class="field">
            <label for="append_to_note_shortcut">Append to Note</label>
            <ShortcutInput
                id="append_to_note_shortcut"
                bind:value={settings.append_to_note_shortcut}
                placeholder="Cmd+Option+Enter"
                {onChange}
            />
            <FieldIssue issue={validation.append_to_note_shortcut} />
        </div>
    </Section>

    <Section title="Reader Window">
        <div class="field">
            <label for="reader_shortcut">Open Reader Window</label>
            <ShortcutInput
                id="reader_shortcut"
                bind:value={settings.reader_shortcut}
                placeholder="Cmd+Shift+R"
                {onChange}
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
                <ShortcutInput
                    id="reader_close_shortcut"
                    bind:value={settings.reader_close_shortcut}
                    placeholder="Optional"
                    {onChange}
                />
                <FieldIssue issue={validation.reader_close_shortcut} />
                <small>Leave empty to disable closing via shortcut</small>
            </div>
        {/if}
        <div class="field">
            <label for="reader_open_in_obsidian_shortcut">Open Note in Obsidian</label>
            <ShortcutInput
                id="reader_open_in_obsidian_shortcut"
                bind:value={settings.reader_open_in_obsidian_shortcut}
                placeholder="Cmd+Shift+O"
                {onChange}
            />
            <FieldIssue issue={validation.reader_open_in_obsidian_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
        <div class="field">
            <label for="reader_navigate_back_shortcut">Navigate Back</label>
            <ShortcutInput
                id="reader_navigate_back_shortcut"
                bind:value={settings.reader_navigate_back_shortcut}
                placeholder="Optional"
                {onChange}
            />
            <FieldIssue issue={validation.reader_navigate_back_shortcut} />
            <small>Leave empty to disable</small>
        </div>
        <div class="field">
            <label for="reader_command_palette_shortcut">Open Command Palette</label>
            <ShortcutInput
                id="reader_command_palette_shortcut"
                bind:value={settings.reader_command_palette_shortcut}
                placeholder="Cmd+P"
                {onChange}
            />
            <FieldIssue issue={validation.reader_command_palette_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
        <div class="field">
            <label for="reader_next_tab_shortcut">Next Tab</label>
            <ShortcutInput
                id="reader_next_tab_shortcut"
                bind:value={settings.reader_next_tab_shortcut}
                placeholder="Option+Right"
                {onChange}
            />
            <FieldIssue issue={validation.reader_next_tab_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
        <div class="field">
            <label for="reader_previous_tab_shortcut">Previous Tab</label>
            <ShortcutInput
                id="reader_previous_tab_shortcut"
                bind:value={settings.reader_previous_tab_shortcut}
                placeholder="Option+Left"
                {onChange}
            />
            <FieldIssue issue={validation.reader_previous_tab_shortcut} />
            <small>Shortcut pressed inside the Reader window</small>
        </div>
    </Section>

    <Section title="Copy Text to Collector">
        <div class="field">
            <label for="capture_text_shortcut">Shortcut</label>
            <ShortcutInput
                id="capture_text_shortcut"
                bind:value={settings.capture_text_shortcut}
                placeholder="Cmd+Shift+C"
                {onChange}
            />
            <FieldIssue issue={validation.capture_text_shortcut} />
        </div>
        <div class="info-note">
            <div class="info-note-title">Accessibility Permission</div>
            <p>
                Required for "Copy Text to Collector". Enable Collector in
                <strong>System Settings → Privacy & Security → Accessibility</strong>, then restart the app.
            </p>
        </div>
    </Section>

    <Section title="OCR / Image to Text">
        <div class="field">
            <label for="ocr_shortcut">Shortcut</label>
            <ShortcutInput
                id="ocr_shortcut"
                bind:value={settings.ocr_shortcut}
                placeholder="Cmd+Shift+T"
                {onChange}
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
    .divider {
        border-top: 1px solid var(--settings-border, rgba(0, 0, 0, 0.07));
        margin: 14px 0 4px;
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
