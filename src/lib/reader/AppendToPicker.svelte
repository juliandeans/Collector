<script>
    import { createEventDispatcher, tick } from "svelte";
    import CommandPalette from "./CommandPalette.svelte";

    export let open = false;
    export let step = 1;
    export let query = "";
    export let notes = [];
    export let selectedIndex = 0;
    export let selectedNote = null;
    export let headings = [];
    export let headingIndex = 0;
    export let inputRef;

    const dispatch = createEventDispatcher();
    let headingListRef;
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

    $: if (open && step === 1) {
        void tick().then(() => {
            focusActiveStep();
        });
    }

    $: if (open && step === 2) {
        headingIndex;
        headings.length;
        void tick().then(() => {
            headingListRef?.focus({ preventScroll: true });
            ensureSelectedHeadingVisible();
        });
    }
</script>

<div class="append-to-picker-shell">
    <CommandPalette
        open={open && step === 1}
        {query}
        {notes}
        {selectedIndex}
        bind:inputRef
        on:queryChange={(event) => dispatch("queryChange", event.detail)}
        on:selectIndex={(event) => dispatch("selectIndex", event.detail)}
        on:openNote={(event) => dispatch("selectNote", event.detail)}
        on:close={() => dispatch("close")}
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
            on:mousedown={() => dispatch("close")}
        />

        <div class="heading-picker-panel" role="dialog" aria-modal="true">
            <div class="heading-picker-note-name">
                {selectedNote?.name ?? "Selected note"}
            </div>
            <div class="heading-picker-note-path">
                {selectedNote?.relative_path ?? ""}
            </div>

            <div
                class="heading-picker-results"
                bind:this={headingListRef}
                role="listbox"
                aria-label="Append targets"
                tabindex="0"
                on:keydown={handleHeadingKeydown}
            >
                <button
                    type="button"
                    class="heading-picker-item heading-picker-end-item"
                    class:selected={headingIndex === 0}
                    on:pointermove={(event) =>
                        handleHeadingPointerMove(0, event)}
                    on:click={() => dispatch("selectHeading", null)}
                >
                    <span class="heading-picker-label">Append at end</span>
                    <span class="heading-picker-meta">
                        Use the existing append behavior
                    </span>
                </button>

                {#each headings as heading, index (`${heading.lineIndex}-${heading.display}`)}
                    <button
                        type="button"
                        class="heading-picker-item"
                        class:selected={headingIndex === index + 1}
                        on:pointermove={(event) =>
                            handleHeadingPointerMove(index + 1, event)}
                        on:click={() => dispatch("selectHeading", heading)}
                    >
                        <span class="heading-picker-label"
                            >{heading.display}</span
                        >
                        <span class="heading-picker-meta">
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
        border-radius: var(--app-border-radius, 12px);
        background: color-mix(
            in srgb,
            var(--app-background, #1e1e2e) var(--app-transparency, 85%),
            transparent
        );
        backdrop-filter: blur(var(--app-blur, 80px))
            saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
        -webkit-backdrop-filter: blur(var(--app-blur, 80px))
            saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
        color: var(--app-text-color, #ffffff);
        font-family: var(--app-font-family, var(--font-family));
        border: 1px solid rgba(255, 255, 255, 0.08);
        box-shadow: var(--overlay-shadow);
        transform: translateX(-50%);
        z-index: 1;
    }

    .heading-picker-panel::before {
        content: "";
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        height: 2px;
        background: linear-gradient(
            90deg,
            color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent),
            color-mix(in srgb, var(--accent-color, #8b5cf6) 35%, transparent),
            color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent)
        );
        background-size: 200% 100%;
        animation: shimmer 3s linear infinite;
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
        color: rgba(255, 255, 255, 0.58);
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

    .heading-picker-item:hover,
    .heading-picker-item.selected {
        background: color-mix(
            in srgb,
            var(--accent-color, #8b5cf6) 14%,
            transparent
        );
    }

    .heading-picker-end-item .heading-picker-label {
        color: rgba(255, 255, 255, 0.82);
    }

    .heading-picker-label {
        color: var(--app-text-color, #ffffff);
        font-weight: 500;
        font-size: 13px;
    }

    .heading-picker-meta {
        margin-top: 2px;
        color: var(--text-secondary);
        font-size: 10px;
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

    @keyframes shimmer {
        0% {
            background-position: 200% 0;
        }

        100% {
            background-position: -200% 0;
        }
    }
</style>
