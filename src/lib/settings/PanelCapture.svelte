<script>
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";

    export let settings;
    export let showStatus;

    let showTemplateEditor = false;

    $: void showStatus;
</script>

<div class="settings-panel">
    <Section title="Window Size">
        <div class="field-row">
            <div class="field">
                <label for="window_width">Width (px)</label>
                <input
                    type="number"
                    id="window_width"
                    bind:value={settings.window_width}
                    min="200"
                    max="800"
                />
            </div>
            <div class="field">
                <label for="window_height">Height (px)</label>
                <input
                    type="number"
                    id="window_height"
                    bind:value={settings.window_height}
                    min="200"
                    max="1200"
                />
            </div>
        </div>
    </Section>

    <Section title="Entry">
        <div class="field">
            <label for="entry_header">Entry Header</label>
            <input
                type="text"
                id="entry_header"
                bind:value={settings.entry_header}
                placeholder="#### HH:mm"
            />
            <small>
                Supported: HH (24h), hh / h (12h), mm, ss, a / A (am/pm) · e.g.
                #### HH:mm or #### h:mm a
            </small>
        </div>
    </Section>

    <Section title="Daily Note">
        <div class="field">
            <label for="daily_note_folder">Daily Note Path</label>
            <input
                type="text"
                id="daily_note_folder"
                bind:value={settings.daily_note_folder}
                placeholder="e.g. Daily Notes/"
            />
            <small>Relative path in vault for daily notes. Must match your Obsidian Daily Notes folder path.</small>
        </div>
        <div class="field">
            <label for="daily_note_format">Daily Note Format</label>
            <input
                type="text"
                id="daily_note_format"
                bind:value={settings.daily_note_format}
                placeholder="YYYY-MM-DD"
            />
            <small>
                Filename format (e.g. YYYY-MM-DD). Supports: YYYY, MM, DD
            </small>
        </div>

        <div class="field">
            <Checkbox bind:checked={settings.daily_note_create_if_missing}>
                Create daily note if it does not exist
            </Checkbox>
            {#if settings.daily_note_create_if_missing}
                <small>
                    Requires the Obsidian Advanced URI plugin. Collector opens
                    Obsidian to create the daily note, then appends the capture
                    to the configured section.
                </small>
            {/if}
        </div>

        {#if settings.daily_note_create_if_missing}
            <div class="field">
                <label for="daily_note_create_timeout_ms">Wait timeout (ms)</label>
                <input
                    type="number"
                    id="daily_note_create_timeout_ms"
                    min="1000"
                    max="60000"
                    step="100"
                    bind:value={settings.daily_note_create_timeout_ms}
                />
                <small
                    >How long Collector waits for Obsidian to create the daily
                    note before giving up (1000–60000 ms).</small
                >
            </div>
        {/if}

        <div class="field">
            <label for="daily_note_target_heading">Target Heading</label>
            <input
                type="text"
                id="daily_note_target_heading"
                bind:value={settings.daily_note_target_heading}
                placeholder="### Note"
            />
            <small>Leave empty to append to the end of the daily note.</small>
        </div>

        {#if settings.daily_note_target_heading?.trim()}
            <div class="field">
                <label for="daily_note_insert_position"
                    >Insert Position in Target Header Area</label
                >
                <select
                    id="daily_note_insert_position"
                    bind:value={settings.daily_note_insert_position}
                >
                    <option value="bottom">Bottom of section</option>
                    <option value="top">Top of section</option>
                </select>
            </div>
        {/if}

        {#if settings.daily_note_target_heading?.trim()}
            <div class="field">
                <Checkbox bind:checked={settings.daily_note_create_heading_if_missing}>
                    Create heading if it does not exist
                </Checkbox>
            </div>
        {/if}
    </Section>

    <Section title="Buttons">
        <div class="field">
            <Checkbox bind:checked={settings.show_capture_action_bar}>
                Show action buttons
            </Checkbox>
        </div>
    </Section>

    <Section title="Pickers">
        <div class="field">
            <Checkbox bind:checked={settings.show_note_paths}>
                Show file paths in note pickers
            </Checkbox>
            <small>
                Displays the vault-relative path below each note name in the
                Command Palette, Append Picker, and Wikilink autocomplete
            </small>
        </div>

        <div class="field">
            <label for="autocomplete_results">Max autocomplete results</label>
            <input
                type="number"
                id="autocomplete_results"
                min="5"
                max="50"
                step="1"
                bind:value={settings.autocomplete_results}
            />
            <small>Number of notes shown in pickers (5–50)</small>
        </div>
    </Section>

    <Section title="New Note Defaults">
        <div class="field">
            <label for="notes_folder">Notes Folder</label>
            <input
                type="text"
                id="notes_folder"
                bind:value={settings.notes_folder}
                placeholder="Notes/"
            />
            <small>Relative path in vault for new notes. Supports date tokens: YYYY, MM, DD, HH, mm, ss</small>
        </div>
        <div class="field">
            <label for="note_filename_template">Note Filename Template</label>
            <input
                type="text"
                id="note_filename_template"
                bind:value={settings.note_filename_template}
                placeholder="note-YYYY-MM-DD-HHmmss"
            />
            <small
                >Supported: YYYY, MM, DD, HH (24h), hh / h (12h), mm, ss, A / a
                (am/pm)</small
            >
        </div>
    </Section>

    <Section title="Template">
        <div class="field">
            <label for="note_template">Template Text</label>
            <button
                class="secondary"
                on:click={() => (showTemplateEditor = !showTemplateEditor)}
            >
                {showTemplateEditor ? "Hide Template" : "Edit Template"}
            </button>
            {#if showTemplateEditor}
                <textarea
                    id="note_template"
                    bind:value={settings.note_template}
                    placeholder="---&#10;created: <% tp.date.now(&quot;YYYY-MM-DD hh:mm&quot;) %>&#10;modified: &#10;daily: &quot;[[<% tp.date.now(&quot;YYYY-MM-DD&quot;) %>]]&quot;&#10;tags: inbox&#10;type: inbox&#10;---"
                    rows="8"
                    class="template-editor"
                />
                <small
                    >This text will be inserted at the beginning of each new
                    note (e.g. for frontmatter/properties)</small
                >
            {/if}
        </div>
    </Section>
</div>

<style>
    .field-row {
        display: flex;
        gap: 12px;
    }

    .field-row .field {
        flex: 1;
    }

    .template-editor {
        margin-top: 8px;
    }

    @media (max-width: 860px) {
        .field-row {
            flex-direction: column;
            gap: 0;
        }
    }
</style>
