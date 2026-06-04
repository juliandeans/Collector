<script>
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";
    import FieldIssue from "./FieldIssue.svelte";

    export let settings;
    export let showStatus;
    export let validation = {};
    export let onChange = () => {};

    $: void showStatus;

    function handleShortcutKeyDown(event, field) {
        event.preventDefault();

        if (
            ["Control", "Shift", "Alt", "Meta", "Command"].includes(event.key)
        ) {
            return;
        }

        const modifiers = [];
        if (event.metaKey) modifiers.push("Cmd");
        if (event.ctrlKey) modifiers.push("Ctrl");
        if (event.shiftKey) modifiers.push("Shift");
        if (event.altKey) modifiers.push("Alt");

        let key = event.code;

        const keyMap = {
            Space: "Space",
            Escape: "Escape",
            Enter: "Enter",
            Backspace: "Backspace",
            Delete: "Delete",
            ArrowUp: "Up",
            ArrowDown: "Down",
            ArrowLeft: "Left",
            ArrowRight: "Right",
            Tab: "Tab",
            Home: "Home",
            End: "End",
            PageUp: "PageUp",
            PageDown: "PageDown",
        };

        if (keyMap[key]) {
            key = keyMap[key];
        } else if (key.startsWith("Key")) {
            key = key.substring(3).toUpperCase();
        } else if (key.startsWith("Digit")) {
            key = key.substring(5);
        } else if (key.startsWith("F") && key.length <= 3) {
            key = key;
        } else {
            return;
        }

        if (modifiers.length === 0) {
            return;
        }

        const shortcut = [...modifiers, key].join("+");
        settings[field] = shortcut;
        settings = { ...settings };
        onChange();
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
                on:keydown={(e) => handleShortcutKeyDown(e, "global_shortcut")}
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
                    on:keydown={(e) =>
                        handleShortcutKeyDown(e, "global_close_shortcut")}
                />
                <FieldIssue issue={validation.global_close_shortcut} />
                <small>Leave empty to disable closing via shortcut</small>
            </div>
        {/if}
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
                on:keydown={(e) => handleShortcutKeyDown(e, "reader_shortcut")}
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
                    on:keydown={(e) =>
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
                on:keydown={(e) =>
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
                on:keydown={(e) =>
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
                on:keydown={(e) =>
                    handleShortcutKeyDown(e, "reader_command_palette_shortcut")}
            />
            <FieldIssue issue={validation.reader_command_palette_shortcut} />
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
                on:keydown={(e) =>
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

    <Section title="Save Actions">
        <div class="field">
            <label for="save_to_daily_shortcut">Save to Daily Note</label>
            <input
                class="shortcut-input"
                type="text"
                id="save_to_daily_shortcut"
                bind:value={settings.save_to_daily_shortcut}
                placeholder="Cmd+Enter"
                on:keydown={(e) =>
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
                on:keydown={(e) =>
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
                on:keydown={(e) =>
                    handleShortcutKeyDown(e, "append_to_note_shortcut")}
            />
            <FieldIssue issue={validation.append_to_note_shortcut} />
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
