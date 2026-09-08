<script>
    import { run } from 'svelte/legacy';

    import { createEventDispatcher, tick } from "svelte";
    import CommandPalette from "./CommandPalette.svelte";
    import "../overlay.css";

    /**
     * @typedef {Object} Props
     * @property {boolean} [open]
     * @property {number} [step]
     * @property {string} [query]
     * @property {any} [notes]
     * @property {number} [selectedIndex]
     * @property {any} [selectedNote]
     * @property {any} [headings]
     * @property {number} [headingIndex]
     * @property {boolean} [showPaths]
     * @property {any} inputRef
     */

    /** @type {Props} */
    let {
        open = false,
        step = 1,
        query = "",
        notes = [],
        selectedIndex = 0,
        selectedNote = null,
        headings = [],
        headingIndex = 0,
        showPaths = true,
        inputRef = $bindable()
    } = $props();

    const dispatch = createEventDispatcher();
    let headingListRef = $state();
    let suppressPointerUntil = 0;
    let lastPointerX = null;
    let lastPointerY = null;

    function ensureSelectedHeadingVisible() {
        if (!headingListRef || !open || step !== 2) return;

        const activeItem = headingListRef.querySelector(
            ".heading-picker-item.selected",
        );
        if (!activeItem) return;

        const styles = getComputedStyle(headingListRef);
        const paddingTop = parseFloat(styles.paddingTop) || 0;
        const paddingBottom = parseFloat(styles.paddingBottom) || 0;
        const itemTop = activeItem.offsetTop;
        const itemBottom = itemTop + activeItem.offsetHeight;
        const viewTop = headingListRef.scrollTop;
        const viewBottom = viewTop + headingListRef.clientHeight;

        if (itemTop < viewTop) {
            suppressPointerSelection();
            headingListRef.scrollTop = Math.max(itemTop - paddingTop, 0);
            return;
        }

        if (itemBottom > viewBottom) {
            suppressPointerSelection();
            headingListRef.scrollTop = Math.max(
                itemBottom - headingListRef.clientHeight + paddingBottom,
                0,
            );
        }
    }

    function suppressPointerSelection() {
        suppressPointerUntil = performance.now() + 180;
    }

    function handleHeadingPointerMove(index, event) {
        const samePointerPosition =
            event.clientX === lastPointerX &&
            event.clientY === lastPointerY;
        const stationaryFirstMove =
            lastPointerX === null &&
            event.movementX === 0 &&
            event.movementY === 0;

        if (samePointerPosition || stationaryFirstMove) {
            lastPointerX = event.clientX;
            lastPointerY = event.clientY;
            return;
        }

        lastPointerX = event.clientX;
        lastPointerY = event.clientY;

        if (index === headingIndex) return;
        if (performance.now() < suppressPointerUntil) return;

        dispatch("headingIndexChange", index);
    }

    function focusActiveStep() {
        if (!open) return;

        if (step === 1) {
            inputRef?.focus();
            return;
        }

        headingListRef?.focus();
    }

    function handleHeadingKeydown(event) {
        event.stopPropagation();

        const maxIndex = headings.length;

        if (event.key === "ArrowDown") {
            event.preventDefault();
            suppressPointerSelection();
            dispatch(
                "headingIndexChange",
                Math.min(headingIndex + 1, maxIndex),
            );
            return;
        }

        if (event.key === "ArrowUp") {
            event.preventDefault();
            suppressPointerSelection();
            dispatch("headingIndexChange", Math.max(headingIndex - 1, 0));
            return;
        }

        if (event.key === "Enter") {
            event.preventDefault();
            dispatch(
                "selectHeading",
                headingIndex === 0 ? null : headings[headingIndex - 1],
            );
            return;
        }

        if (event.key === "Escape") {
            event.preventDefault();
            dispatch("backToStep1");
        }
    }

    run(() => {
        if (open && step === 1) {
            void tick().then(() => {
                focusActiveStep();
            });
        }
    });

    run(() => {
        if (open && step === 2) {
            setTimeout(() => {
                headingListRef?.focus({ preventScroll: true });
            }, 0);
        }
    });
</script>

<div class="append-to-picker-shell">
    <CommandPalette
        open={open && step === 1}
        {query}
        {notes}
        {selectedIndex}
        showPaths={showPaths}
        bind:inputRef
        onQueryChange={(event) => dispatch("queryChange", event)}
        onSelectIndex={(index) => dispatch("selectIndex", index)}
        onOpenNote={(note) => dispatch("selectNote", note)}
        onClose={() => dispatch("close")}
    />

    <div
        class="heading-picker-wrapper"
        class:heading-picker-wrapper-visible={open && step === 2}
        aria-hidden={!(open && step === 2)}
    >
        <button
            type="button"
            class="heading-picker-backdrop"
            aria-label="Close append picker"
            tabindex="-1"
            onmousedown={() => dispatch("close")}
></button>

        <div class="heading-picker-panel overlay-panel" role="dialog" aria-modal="true">
            <div class="heading-picker-note-name">
                {selectedNote?.name ?? "Selected note"}
            </div>
            {#if showPaths}
                <div class="heading-picker-note-path overlay-path">
                    {selectedNote?.relative_path ?? ""}
                </div>
            {/if}

            <div
                class="heading-picker-results"
                bind:this={headingListRef}
                role="listbox"
                aria-label="Append targets"
                tabindex="0"
                onkeydown={handleHeadingKeydown}
            >
                <button
                    type="button"
                    class="heading-picker-item heading-picker-end-item overlay-item"
                    class:selected={headingIndex === 0}
                    onpointermove={(event) =>
                        handleHeadingPointerMove(0, event)}
                    onclick={() => dispatch("selectHeading", null)}
                >
                    <span class="heading-picker-label overlay-name">Append at end</span>
                    <span class="heading-picker-meta overlay-path">
                        Use the existing append behavior
                    </span>
                </button>

                {#each headings as heading, index (`${heading.lineIndex}-${heading.display}`)}
                    <button
                        type="button"
                        class="heading-picker-item overlay-item"
                        class:selected={headingIndex === index + 1}
                        onpointermove={(event) =>
                            handleHeadingPointerMove(index + 1, event)}
                        onclick={() => dispatch("selectHeading", heading)}
                    >
                        <span class="heading-picker-label overlay-name"
                            >{heading.display}</span
                        >
                        <span class="heading-picker-meta overlay-path">
                            Insert after this section
                        </span>
                    </button>
                {/each}
            </div>
        </div>
    </div>
</div>

<style>
    .append-to-picker-shell {
        position: absolute;
        inset: 0;
        pointer-events: none;
        z-index: 121;
        overflow-x: hidden;
    }

    .heading-picker-wrapper {
        position: fixed;
        inset: 0;
        z-index: 1000;
        pointer-events: none;
        opacity: 0;
    }

    .heading-picker-wrapper.heading-picker-wrapper-visible {
        opacity: 1;
        pointer-events: auto;
    }

    .heading-picker-backdrop {
        position: absolute;
        inset: 0;
        border: 0;
        padding: 0;
        appearance: none;
        background: rgba(8, 10, 14, 0.22);
    }

    .heading-picker-panel {
        position: absolute;
        top: 52px;
        left: 50%;
        width: min(calc(100% - 28px), 520px);
        max-height: min(70vh, 520px);
        display: flex;
        flex-direction: column;
        overflow: hidden;
        transform: translateX(-50%);
        z-index: 1;
    }

    .heading-picker-note-name {
        padding: 10px 16px 0;
        font-size: 15px;
        font-weight: 600;
        color: var(--app-text-color, #ffffff);
    }

    .heading-picker-note-path {
        padding: 4px 16px 12px;
        font-size: 12px;
        border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    }

    .heading-picker-results {
        flex: 1;
        max-height: 420px;
        overflow-y: auto;
        padding: 10px 8px 8px;
        outline: none;
    }

    .heading-picker-item {
        border: 0;
        background: transparent;
        color: inherit;
        font: inherit;
        display: flex;
        flex-direction: column;
        gap: 2px;
        width: 100%;
        padding: 8px 12px;
        border-radius: 12px;
        cursor: pointer;
        text-align: left;
        transition: background 0.12s ease;
    }

    .heading-picker-end-item .heading-picker-label {
        color: rgba(255, 255, 255, 0.82);
    }

    .heading-picker-meta {
        margin-top: 2px;
    }

    .heading-picker-results::-webkit-scrollbar {
        width: 6px;
    }

    .heading-picker-results::-webkit-scrollbar-track {
        background: transparent;
    }

    .heading-picker-results::-webkit-scrollbar-thumb {
        background: rgba(0, 0, 0, 0.12);
        border-radius: 3px;
    }
</style>
