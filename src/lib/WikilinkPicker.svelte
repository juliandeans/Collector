<script>
    import { tick } from "svelte";
    import "./overlay.css";

    /**
     * @typedef {Object} Props
     * @property {any} [notes]
     * @property {number} [selectedIndex]
     * @property {boolean} [showPaths]
     * @property {any} [onSelect]
     * @property {any} [onHover]
     */

    /** @type {Props} */
    let {
        notes = [],
        selectedIndex = 0,
        showPaths = true,
        onSelect = () => {},
        onHover = () => {}
    } = $props();

    let pickerRef = $state();
    let scrollRef = $state();
    let suppressPointerUntil = 0;
    let lastPointerX = null;
    let lastPointerY = null;


    async function scrollSelectedIntoView() {
        await tick();

        const selectedItem = pickerRef?.querySelector(".wikilink-picker-item.selected");
        if (!scrollRef || !selectedItem) return;

        const pickerRect = scrollRef.getBoundingClientRect();
        const itemRect = selectedItem.getBoundingClientRect();

        if (itemRect.top < pickerRect.top) {
            suppressPointerSelection();
            scrollRef.scrollTop -= pickerRect.top - itemRect.top;
        } else if (itemRect.bottom > pickerRect.bottom) {
            suppressPointerSelection();
            scrollRef.scrollTop += itemRect.bottom - pickerRect.bottom;
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
    $effect.pre(() => {
        selectedIndex;
        notes;
        scrollSelectedIntoView();
    });
</script>

<div class="wikilink-picker overlay-panel" bind:this={pickerRef}>
    <div class="wikilink-picker-scroll" bind:this={scrollRef}>
        {#each notes.slice(0, 20) as note, index (note.path)}
            <button
                class="wikilink-picker-item overlay-item"
                class:selected={index === selectedIndex}
                type="button"
                onmousedown={(event) => {
                    event.preventDefault();
                    onSelect(note);
                }}
                onpointermove={(event) => handlePointerMove(index, event)}
            >
                <span class="overlay-name">{note.name}</span>
                {#if showPaths}
                    <span class="overlay-path">{note.relative_path}</span>
                {/if}
            </button>
        {/each}
    </div>
</div>

<style>
    .wikilink-picker {
        position: relative;
        display: flex;
        flex-direction: column;
        max-height: min(70vh, 400px);
        overflow: hidden;
    }

    /* Inner scroller so the native scrollbar stays clear of the panel's
       rounded corners (clip + small vertical inset). */
    .wikilink-picker-scroll {
        flex: 1 1 auto;
        min-height: 0;
        margin-block: 2px;
        overflow-y: auto;
    }

    .wikilink-picker-item {
        display: flex;
        flex-direction: column;
        gap: 2px;
        width: 100%;
        padding: 8px 12px;
        border: 0;
        color: inherit;
        font: inherit;
        text-align: left;
        cursor: pointer;
        transition: background var(--transition-fast);
    }
</style>
