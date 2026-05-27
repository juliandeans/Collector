<script>
    export let settings;
    export let showStatus;

    let showTemplateEditor = false;

    $: void showStatus;
</script>

<div class="settings-panel">
    <section>
        <h2>Window Size</h2>
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
    </section>

    <section>
        <h2>New Note Defaults</h2>
        <div class="field">
            <label for="notes_folder">Notes Folder</label>
            <input
                type="text"
                id="notes_folder"
                bind:value={settings.notes_folder}
                placeholder="Notes/"
            />
            <small>Relative path in vault for new notes</small>
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
    </section>

    <section>
        <h2>Template</h2>
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
    </section>
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
