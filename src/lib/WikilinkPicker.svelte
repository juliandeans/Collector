<script>
    import { tick } from "svelte";

    export let notes = [];
    export let selectedIndex = 0;
    export let showPaths = true;
    export let onSelect = () => {};
    export let onHover = () => {};

    let pickerRef;
    let suppressPointerUntil = 0;
    let lastPointerX = null;
    let lastPointerY = null;

    $: {
        selectedIndex;
        notes;
        scrollSelectedIntoView();
    }

    async function scrollSelectedIntoView() {
        await tick();

        const selectedItem = pickerRef?.querySelector(".wikilink-picker-item.selected");
        if (!pickerRef || !selectedItem) return;

        const pickerRect = pickerRef.getBoundingClientRect();
        const itemRect = selectedItem.getBoundingClientRect();

        if (itemRect.top < pickerRect.top) {
            suppressPointerSelection();
            pickerRef.scrollTop -= pickerRect.top - itemRect.top;
        } else if (itemRect.bottom > pickerRect.bottom) {
            suppressPointerSelection();
            pickerRef.scrollTop += itemRect.bottom - pickerRect.bottom;
        }
    }

    function suppressPointerSelection() {
        suppressPointerUntil = performance.now() + 180;
    }

    function handlePointerMove(index, event) {
        const samePointerPosition =
            event.clientX === lastPointerX && event.clientY === lastPointerY;
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

        if (index === selectedIndex) return;
        if (performance.now() < suppressPointerUntil) return;

        onHover(index);
    }
</script>

<div class="wikilink-picker" bind:this={pickerRef}>
    {#each notes.slice(0, 20) as note, index (note.path)}
        <button
            class="wikilink-picker-item"
            class:selected={index === selectedIndex}
            type="button"
            on:mousedown|preventDefault={() => onSelect(note)}
            on:pointermove={(event) => handlePointerMove(index, event)}
        >
            <span class="wikilink-picker-name">{note.name}</span>
            {#if showPaths}
                <span class="wikilink-picker-path">{note.relative_path}</span>
            {/if}
        </button>
    {/each}
</div>

<style>
    .wikilink-picker {
        position: relative;
        display: flex;
        flex-direction: column;
        max-height: min(70vh, 400px);
        overflow-y: auto;
        border: 1px solid rgba(255, 255, 255, 0.08);
        border-radius: var(--app-border-radius, 12px);
        background: color-mix(
            in srgb,
            var(--app-background, #1e1e2e) var(--app-transparency, 85%),
            transparent
        );
        color: var(--app-text-color, #ffffff);
        font-family: var(--app-font-family, var(--font-family));
        backdrop-filter: blur(var(--app-blur, 80px))
            saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
        -webkit-backdrop-filter: blur(var(--app-blur, 80px))
            saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
        box-shadow: var(--overlay-shadow);
    }

    .wikilink-picker::before {
        content: "";
        position: sticky;
        top: 0;
        z-index: 1;
        flex: 0 0 2px;
        background: linear-gradient(
            90deg,
            color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent),
            color-mix(in srgb, var(--accent-color, #8b5cf6) 35%, transparent),
            color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent)
        );
        background-size: 200% 100%;
        animation: shimmer 3s linear infinite;
    }

    .wikilink-picker-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
        width: 100%;
        padding: 8px 12px;
        border: 0;
        background: transparent;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
        transition: background var(--transition-fast);
    }

    .wikilink-picker-item:hover,
    .wikilink-picker-item.selected {
        background: color-mix(
            in srgb,
            var(--accent-color, #8b5cf6) 14%,
            transparent
        );
    }

    .wikilink-picker-name {
        color: var(--app-text-color, #ffffff);
        font-size: 13px;
        font-weight: 500;
    }

    .wikilink-picker-path {
        overflow: hidden;
        color: var(--text-secondary);
        font-size: 10px;
        white-space: nowrap;
        text-overflow: ellipsis;
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
