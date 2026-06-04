<script>
    import Section from "./Section.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import { invoke } from "@tauri-apps/api/core";
    import { open } from "@tauri-apps/plugin-dialog";

    export let settings;
    export let showStatus;
    export let validation = {};
    export let onChange = () => {};

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
            onChange();
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
    <Section title="Vault">
        <div class="field">
            <label for="vault_name">Vault Name</label>
            <input
                type="text"
                id="vault_name"
                bind:value={settings.vault_name}
                placeholder="Vault"
            />
            <FieldIssue issue={validation.vault_name} />
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
            <FieldIssue issue={validation.vault_path} />
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
    </Section>
</div>

<style>
    .reindex-row {
        display: flex;
        gap: 8px;
        align-items: center;
        margin-top: 8px;
    }

    .index-status {
        color: var(--settings-text-secondary, #666);
        font-size: 11px;
    }
</style>
