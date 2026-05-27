<script>
    import { open } from "@tauri-apps/plugin-dialog";
    import {
        getReaderIconComponent,
        readerIconOptions,
    } from "../reader-icons.js";
    import { normalizePinnedNotes } from "./pinned-notes.js";
    import { normalizeComparablePath } from "./path-utils.js";

    export let settings;
    export let showStatus;

    function getFilename(path) {
        return (
            path.replace(/\\/g, "/").split("/").pop()?.replace(/\.md$/i, "") ||
            path
        );
    }

    function toRelativeVaultPath(path = "") {
        const normalizedPath = normalizeComparablePath(path.trim());
        const normalizedVaultPath = normalizeComparablePath(
            settings.vault_path ?? "",
        );

        if (!normalizedPath || !normalizedVaultPath) {
            return "";
        }

        if (normalizedPath === normalizedVaultPath) {
            return "";
        }

        if (normalizedPath.startsWith(`${normalizedVaultPath}/`)) {
            return normalizedPath.slice(normalizedVaultPath.length + 1);
        }

        return "";
    }

    async function addPinnedNotes() {
        const selected = await open({
            multiple: true,
            filters: [{ name: "Markdown", extensions: ["md"] }],
            defaultPath: settings.vault_path || undefined,
        });

        if (selected) {
            const paths = Array.isArray(selected) ? selected : [selected];
            const existing = normalizePinnedNotes(settings.pinned_notes);
            const existingByPath = new Set(existing.map((note) => note.path));
            let skippedOutsideVault = false;
            const additions = paths
                .map((path) => toRelativeVaultPath(path))
                .filter((path) => {
                    if (!path) {
                        skippedOutsideVault = true;
                        return false;
                    }

                    return !existingByPath.has(path);
                })
                .map((path) => ({
                    path,
                    label: getFilename(path),
                    icon: "",
                }));

            settings.pinned_notes = [...existing, ...additions];
            settings = { ...settings };

            if (skippedOutsideVault) {
                showStatus("Pinned notes must be inside the current vault", "error");
            }
        }
    }

    function removePinnedNote(pathToRemove) {
        settings.pinned_notes = normalizePinnedNotes(settings.pinned_notes).filter(
            (note) => note.path !== pathToRemove,
        );
        settings = { ...settings };
    }

    function updatePinnedNote(pathToUpdate, field, value) {
        settings.pinned_notes = normalizePinnedNotes(settings.pinned_notes).map(
            (note) =>
                note.path === pathToUpdate
                    ? {
                          ...note,
                          [field]: value,
                      }
                    : note,
        );
        settings = { ...settings };
    }
</script>

<div class="settings-panel">
    <section>
        <h2>Window Size</h2>
        <div class="field-row">
            <div class="field">
                <label for="reader_width">Width (px)</label>
                <input
                    type="number"
                    id="reader_width"
                    bind:value={settings.reader_width}
                    min="200"
                    max="800"
                />
            </div>
            <div class="field">
                <label for="reader_height">Height (px)</label>
                <input
                    type="number"
                    id="reader_height"
                    bind:value={settings.reader_height}
                    min="200"
                    max="1200"
                />
            </div>
        </div>
    </section>

    <section>
        <h2>Pinned Notes</h2>
        {#if normalizePinnedNotes(settings.pinned_notes).length > 0}
            <div class="note-list">
                {#each normalizePinnedNotes(settings.pinned_notes) as note}
                    <div class="note-list-item">
                        <div class="note-list-copy">
                            <div class="note-list-editors">
                                <input
                                    class="pinned-note-label"
                                    type="text"
                                    value={note.label}
                                    placeholder={getFilename(note.path)}
                                    on:input={(event) =>
                                        updatePinnedNote(
                                            note.path,
                                            "label",
                                            event.currentTarget.value,
                                        )}
                                />
                            </div>
                            <div class="pinned-note-icons">
                                {#each readerIconOptions as iconOption}
                                    <button
                                        class="icon-choice"
                                        class:selected={note.icon === iconOption.id}
                                        type="button"
                                        title={iconOption.label}
                                        aria-label={iconOption.label}
                                        on:click={() =>
                                            updatePinnedNote(
                                                note.path,
                                                "icon",
                                                iconOption.id,
                                            )}
                                    >
                                        {#if iconOption.component}
                                            <svelte:component
                                                this={getReaderIconComponent(
                                                    iconOption.id,
                                                )}
                                                size={14}
                                                strokeWidth={1.9}
                                            />
                                        {:else}
                                            <span class="icon-choice-none">-</span>
                                        {/if}
                                    </button>
                                {/each}
                            </div>
                            <small>{note.path}</small>
                        </div>
                        <button
                            class="remove-note"
                            type="button"
                            on:click={() => removePinnedNote(note.path)}
                        >
                            ✕
                        </button>
                    </div>
                {/each}
            </div>
        {:else}
            <div class="empty-note-list">No pinned notes selected.</div>
        {/if}

        <small>
            Optional icon and label only affect the reader tab display, not the
            actual Markdown file.
        </small>
        <button class="secondary" type="button" on:click={addPinnedNotes}
            >+ Add Note</button
        >
    </section>

    <section>
        <h2>Content Filters</h2>
        <div class="field">
            <label class="checkbox">
                <input
                    type="checkbox"
                    bind:checked={settings.reader_hide_frontmatter}
                />
                Hide YAML Frontmatter
            </label>
            <small>Hides the --- metadata block at the top of notes</small>
        </div>

        <div class="field">
            <label class="checkbox">
                <input
                    type="checkbox"
                    bind:checked={settings.reader_hide_dataview}
                />
                Hide Code Blocks (Dataview, JS, etc.)
            </label>
            <small>Hides all ```language ... ``` code blocks</small>
        </div>

        <div class="field">
            <label class="checkbox">
                <input
                    type="checkbox"
                    bind:checked={settings.reader_hide_obsidian_comments}
                />
                Hide Obsidian Comments
            </label>
            <small>Hides %% ... %% comment blocks</small>
        </div>

        <div class="field">
            <label class="checkbox">
                <input
                    type="checkbox"
                    bind:checked={settings.reader_hide_inline_fields}
                />
                Hide Inline Fields (Dataview)
            </label>
            <small>Hides key:: value fields from note text</small>
        </div>

        <div class="field">
            <label class="checkbox">
                <input type="checkbox" bind:checked={settings.reader_hide_html} />
                Hide HTML Blocks
            </label>
            <small>Hides raw &lt;div&gt;, &lt;span&gt; and other HTML tags</small>
        </div>

        <div class="field">
            <label class="checkbox">
                <input
                    type="checkbox"
                    bind:checked={settings.reader_hide_callouts}
                />
                Hide Callouts
            </label>
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

    .note-list {
        display: flex;
        flex-direction: column;
        gap: 8px;
        margin: 10px 0 12px;
    }

    .note-list-item {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 12px;
        padding: 10px 12px;
        border-radius: 10px;
        background: rgba(0, 0, 0, 0.03);
        border: 1px solid rgba(0, 0, 0, 0.06);
    }

    .note-list-copy {
        min-width: 0;
    }

    .note-list-editors {
        display: block;
    }

    .pinned-note-label {
        width: 100%;
        font-weight: 600;
    }

    .pinned-note-icons {
        display: flex;
        flex-wrap: wrap;
        gap: 6px;
        margin-top: 8px;
    }

    .icon-choice {
        width: 30px;
        height: 30px;
        padding: 0;
        border-radius: 8px;
        border: 1px solid rgba(0, 0, 0, 0.08);
        background: rgba(255, 255, 255, 0.72);
        color: #6b7280;
        display: inline-flex;
        align-items: center;
        justify-content: center;
        box-shadow: none;
    }

    .icon-choice:hover {
        color: #374151;
        border-color: rgba(139, 92, 246, 0.22);
        background: rgba(139, 92, 246, 0.08);
        transform: none;
    }

    .icon-choice.selected {
        color: #7c3aed;
        border-color: rgba(124, 58, 237, 0.28);
        background: rgba(139, 92, 246, 0.14);
    }

    .icon-choice-none {
        font-size: 14px;
        line-height: 1;
    }

    .note-list-copy small {
        display: block;
        margin-top: 3px;
        color: #6b7280;
        overflow-wrap: anywhere;
    }

    .empty-note-list {
        margin: 10px 0 12px;
        padding: 12px;
        border-radius: 10px;
        background: rgba(0, 0, 0, 0.03);
        border: 1px dashed rgba(0, 0, 0, 0.08);
        color: #6b7280;
        font-size: 12px;
    }

    .remove-note {
        flex-shrink: 0;
        width: 28px;
        height: 28px;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: #9ca3af;
        cursor: pointer;
        transition:
            background 0.2s ease,
            color 0.2s ease;
    }

    .remove-note:hover {
        background: rgba(239, 68, 68, 0.1);
        color: #dc2626;
    }

    @media (max-width: 860px) {
        .field-row {
            flex-direction: column;
            gap: 0;
        }
    }
</style>
