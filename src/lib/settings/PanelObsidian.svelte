<script>
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";

    export let settings;
    export let showStatus;

    let isIndexing = false;
    let indexStatus = "";

    $: void showStatus;

    async function pickVaultPath() {
        const selected = await open({
            directory: true,
            multiple: false,
            defaultPath: settings.vault_path || undefined,
        });
        if (selected) {
            settings.vault_path = selected;
            indexStatus = "";
        }
    }

    async function handleReindex() {
        isIndexing = true;
        indexStatus = "";

        try {
            const count = await invoke("reindex_vault");
            indexStatus = `✓ Index built (${count} files)`;
        } catch (e) {
            indexStatus = `Failed to index vault: ${e}`;
        } finally {
            isIndexing = false;
        }
    }
</script>

<div class="settings-panel">
    <section>
        <h2>Vault</h2>
        <div class="field">
            <label for="vault_name">Vault Name</label>
            <input
                type="text"
                id="vault_name"
                bind:value={settings.vault_name}
                placeholder="Vault"
            />
            <small>Name of your Obsidian vault</small>
        </div>
        <div class="field">
            <label for="vault_path">Vault Path</label>
            <div class="path-picker">
                <input
                    type="text"
                    id="vault_path"
                    bind:value={settings.vault_path}
                    placeholder="/Users/username/Vault"
                    readonly
                />
                <button class="secondary" on:click={pickVaultPath}
                    >Choose...</button
                >
            </div>
            <div class="reindex-row">
                <button
                    class="secondary"
                    type="button"
                    on:click={handleReindex}
                    disabled={isIndexing}
                >
                    {isIndexing ? "Indexing..." : "↻ Re-index Vault"}
                </button>
                {#if indexStatus}
                    <span class="index-status">{indexStatus}</span>
                {/if}
            </div>
            <small>Full path to your Obsidian vault</small>
            <small>
                Re-index if images or notes added outside Collector are not
                appearing correctly.
            </small>
        </div>
    </section>

    <section>
        <h2>Daily Notes</h2>
        <div class="field">
            <label for="daily_note_folder">Daily Note Path</label>
            <input
                type="text"
                id="daily_note_folder"
                bind:value={settings.daily_note_folder}
                placeholder="Journal/Notes/"
            />
            <small>Relative path in vault for daily notes</small>
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
    </section>

    <section>
        <h2>Entry Header</h2>
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
    </section>

    <section>
        <h2>Note Pickers</h2>
        <div class="field">
            <label class="checkbox" for="show_note_paths">
                <input
                    type="checkbox"
                    id="show_note_paths"
                    bind:checked={settings.show_note_paths}
                />
                <span>Show file paths in note pickers</span>
            </label>
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
    </section>
</div>

<style>
    .reindex-row {
        display: flex;
        gap: 8px;
        align-items: center;
        margin-top: 8px;
    }

    .index-status {
        color: #666;
        font-size: 11px;
    }
</style>
