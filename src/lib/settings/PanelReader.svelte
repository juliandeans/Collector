<script>
    import { tick } from "svelte";
    import Section from "./Section.svelte";
    import Checkbox from "./Checkbox.svelte";
    import FieldIssue from "./FieldIssue.svelte";
    import {
        getReaderIconComponent,
        readerIconOptions,
    } from "../reader-icons.js";
    import { normalizePinnedNotes } from "./pinned-notes.js";
    import { filterPaletteNotes } from "../reader/paletteLogic.js";

    export let settings;
    export let validation = {};
    export let onChange = () => {};
    export let vaultNotes = [];

    let notePickerOpen = false;
    let pickerQuery = "";
    let pickerIndex = 0;
    let pickerInputRef;

    function getFilename(path) {
        return (
            path.replace(/\\/g, "/").split("/").pop()?.replace(/\.md$/i, "") ||
            path
        );
    }

    let pickerResultsRef;

    $: filteredPickerNotes = filterPaletteNotes(vaultNotes, pickerQuery, 20);

    function ensurePickerItemVisible() {
        if (!pickerResultsRef) return;
        const active = pickerResultsRef.querySelector(".picker-item.selected");
        if (!active) return;

        const itemTop = active.offsetTop;
        const itemBottom = itemTop + active.offsetHeight;
        const viewTop = pickerResultsRef.scrollTop;
        const viewBottom = viewTop + pickerResultsRef.clientHeight;

        if (itemTop < viewTop) {
            pickerResultsRef.scrollTop = itemTop;
        } else if (itemBottom > viewBottom) {
            pickerResultsRef.scrollTop = itemBottom - pickerResultsRef.clientHeight;
        }
    }

    $: if (notePickerOpen && filteredPickerNotes.length > 0 && pickerIndex >= 0) {
        void tick().then(ensurePickerItemVisible);
    }

    function openNotePicker() {
        pickerQuery = "";
        pickerIndex = 0;
        notePickerOpen = true;
    }

    $: if (notePickerOpen) {
        void tick().then(() => pickerInputRef?.focus());
    }

    function closeNotePicker() {
        notePickerOpen = false;
        pickerInputRef?.blur();
    }

    function pickNote(note) {
        const existing = normalizePinnedNotes(settings.pinned_notes);
        const existingByPath = new Set(existing.map((n) => n.path));

        if (!existingByPath.has(note.relative_path)) {
            settings.pinned_notes = [
                ...existing,
                { path: note.relative_path, label: getFilename(note.relative_path), icon: "" },
            ];
            settings = { ...settings };
            onChange();
        }

        closeNotePicker();
    }

    function handlePickerKeydown(event) {
        if (event.key === "ArrowDown") {
            event.preventDefault();
            pickerIndex = Math.min(pickerIndex + 1, filteredPickerNotes.length - 1);
        } else if (event.key === "ArrowUp") {
            event.preventDefault();
            pickerIndex = Math.max(pickerIndex - 1, 0);
        } else if (event.key === "Enter") {
            event.preventDefault();
            const note = filteredPickerNotes[pickerIndex];
            if (note) pickNote(note);
        } else if (event.key === "Escape") {
            event.preventDefault();
            closeNotePicker();
        }
    }

    function removePinnedNote(pathToRemove) {
        settings.pinned_notes = normalizePinnedNotes(settings.pinned_notes).filter(
            (note) => note.path !== pathToRemove,
        );
        settings = { ...settings };
        onChange();
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
        onChange();
    }
</script>

<div class="settings-panel">
    <Section title="Window Size">
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
                <FieldIssue issue={validation.reader_width} />
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
                <FieldIssue issue={validation.reader_height} />
            </div>
        </div>
    </Section>

    <Section title="Pinned Notes">
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
        <FieldIssue issue={validation.pinned_notes} />
        <button class="secondary" type="button" on:click={openNotePicker}
            >+ Add Note</button
        >
    </Section>

    {#if notePickerOpen}
        <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_static_element_interactions -->
        <div class="picker-backdrop" on:click={closeNotePicker}>
            <div
                class="picker-panel"
                role="dialog"
                aria-modal="true"
                on:click|stopPropagation
                on:keydown={handlePickerKeydown}
            >
                <input
                    bind:this={pickerInputRef}
                    bind:value={pickerQuery}
                    class="picker-input"
                    placeholder="Search vault notes..."
                    spellcheck="false"
                    on:input={() => {
                        pickerIndex = 0;
                    }}
                    on:keydown={handlePickerKeydown}
                />
                <div class="picker-results" bind:this={pickerResultsRef}>
                    {#if filteredPickerNotes.length === 0}
                        <div class="picker-empty">No matching notes</div>
                    {:else}
                        {#each filteredPickerNotes as note, idx}
                            <button
                                class="picker-item"
                                class:selected={idx === pickerIndex}
                                type="button"
                                on:click={() => pickNote(note)}
                                on:pointermove={() => {
                                    pickerIndex = idx;
                                }}
                            >
                                <span class="picker-name">{note.name}</span>
                                <span class="picker-path">{note.relative_path}</span>
                            </button>
                        {/each}
                    {/if}
                </div>
            </div>
        </div>
    {/if}

    <Section title="Content Filters">
        <div class="field">
            <Checkbox bind:checked={settings.reader_hide_frontmatter}>
                Hide YAML Frontmatter
            </Checkbox>
            <small>Hides the --- metadata block at the top of notes</small>
        </div>

        <div class="field">
            <Checkbox bind:checked={settings.reader_hide_dataview}>
                Hide Code Blocks (Dataview, JS, etc.)
            </Checkbox>
            <small>Hides all ```language ... ``` code blocks</small>
        </div>

        <div class="field">
            <Checkbox bind:checked={settings.reader_hide_obsidian_comments}>
                Hide Obsidian Comments
            </Checkbox>
            <small>Hides %% ... %% comment blocks</small>
        </div>

        <div class="field">
            <Checkbox bind:checked={settings.reader_hide_inline_fields}>
                Hide Inline Fields (Dataview)
            </Checkbox>
            <small>Hides key:: value fields from note text</small>
        </div>

        <div class="field">
            <Checkbox bind:checked={settings.reader_hide_html}>
                Hide HTML Blocks
            </Checkbox>
            <small>Hides raw &lt;div&gt;, &lt;span&gt; and other HTML tags</small>
        </div>

        <div class="field">
            <Checkbox bind:checked={settings.reader_hide_callouts}>
                Hide Callouts
            </Checkbox>
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
        background: rgba(128, 128, 128, 0.06);
        border: 1px solid var(--settings-border, rgba(0, 0, 0, 0.06));
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
        border: 1px solid var(--settings-border, rgba(0, 0, 0, 0.08));
        background: var(--settings-surface, rgba(255, 255, 255, 0.72));
        color: var(--settings-text-secondary, #6b7280);
        display: inline-flex;
        align-items: center;
        justify-content: center;
        box-shadow: none;
    }

    .icon-choice:hover {
        color: var(--settings-nav-text, #374151);
        border-color: rgba(139, 92, 246, 0.22);
        background: rgba(139, 92, 246, 0.08);
        transform: none;
    }

    .icon-choice.selected {
        color: var(--settings-accent, #7c3aed);
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
        color: var(--settings-text-secondary, #6b7280);
        overflow-wrap: anywhere;
    }

    .empty-note-list {
        margin: 10px 0 12px;
        padding: 12px;
        border-radius: 10px;
        background: rgba(128, 128, 128, 0.06);
        border: 1px dashed var(--settings-border, rgba(0, 0, 0, 0.08));
        color: var(--settings-text-secondary, #6b7280);
        font-size: 12px;
    }

    .remove-note {
        flex-shrink: 0;
        width: 28px;
        height: 28px;
        border: none;
        border-radius: 8px;
        background: transparent;
        color: var(--settings-section-title, #9ca3af);
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

    .picker-backdrop {
        position: fixed;
        inset: 0;
        z-index: 1000;
        background: rgba(0, 0, 0, 0.22);
        display: flex;
        align-items: flex-start;
        justify-content: center;
        padding-top: 60px;
    }

    .picker-panel {
        width: min(calc(100% - 28px), 480px);
        max-height: 420px;
        display: flex;
        flex-direction: column;
        overflow: hidden;
        border-radius: 12px;
        background: var(--settings-bg, #f2f2f2);
        border: 1px solid var(--settings-border, rgba(0, 0, 0, 0.08));
        box-shadow: var(--overlay-shadow);
    }

    .picker-input {
        width: 100%;
        padding: 12px 14px;
        border: 0;
        border-bottom: 1px solid var(--settings-border, rgba(0, 0, 0, 0.08));
        background: transparent;
        color: var(--settings-text, inherit);
        font: inherit;
        font-size: 14px;
        outline: none;
        box-sizing: border-box;
    }

    .picker-input::placeholder {
        color: var(--settings-text-secondary, #6b7280);
    }

    .picker-results {
        flex: 1;
        overflow-y: auto;
        max-height: 360px;
    }

    .picker-item {
        border: 0;
        border-bottom: 1px solid var(--settings-border, rgba(0, 0, 0, 0.06));
        background: transparent;
        color: var(--settings-text, inherit);
        font: inherit;
        display: flex;
        flex-direction: column;
        gap: 2px;
        width: 100%;
        padding: 8px 14px;
        cursor: pointer;
        text-align: left;
        transition: background 0.1s ease;
    }

    .picker-item:last-child {
        border-bottom: 0;
    }

    .picker-item.selected,
    .picker-item:hover {
        background: color-mix(in srgb, var(--settings-accent, #8b5cf6) 14%, transparent);
    }

    .picker-name {
        font-weight: 500;
        font-size: 13px;
    }

    .picker-path {
        font-size: 11px;
        color: var(--settings-text-secondary, #6b7280);
    }

    .picker-empty {
        padding: 18px 14px;
        color: var(--settings-text-secondary, #6b7280);
        text-align: center;
        font-size: 13px;
    }

    .picker-results::-webkit-scrollbar {
        width: 6px;
    }

    .picker-results::-webkit-scrollbar-track {
        background: transparent;
    }

    .picker-results::-webkit-scrollbar-thumb {
        background: rgba(0, 0, 0, 0.12);
        border-radius: 3px;
    }
</style>
