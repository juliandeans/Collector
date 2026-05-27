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
    <section class="panel-intro">
        <h2>Obsidian Integration</h2>
        <p class="section-description">
            Configure your vault, daily note location and the default header
            Collector inserts into captures.
        </p>
    </section>

    <section>
        <h2>Vault</h2>
        <p class="section-description">
            Choose the Obsidian vault Collector works against and refresh the
            index when files change outside the app.
        </p>

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
        <p class="section-description">
            Define where daily notes live and how their filenames are
            generated.
        </p>

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
        <p class="section-description">
            This Markdown heading is inserted before each saved capture.
        </p>

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
