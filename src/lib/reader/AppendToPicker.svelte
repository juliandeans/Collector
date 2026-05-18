<script>
    import { createEventDispatcher } from "svelte";
    import CommandPalette from "./CommandPalette.svelte";

    export let open = false;
    export let query = "";
    export let notes = [];
    export let selectedIndex = 0;
    export let inputRef;

    const dispatch = createEventDispatcher();
</script>

<div class="append-to-picker-shell">
    <CommandPalette
        {open}
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
        class="append-to-picker-header"
        class:append-to-picker-header-visible={open}
        aria-hidden="true"
    >
        Append to...
    </div>
</div>

<style>
    .append-to-picker-shell {
        position: absolute;
        inset: 0;
        pointer-events: none;
        z-index: 121;
    }

    .append-to-picker-header {
        position: absolute;
        top: 60px;
        left: 50%;
        width: min(calc(100% - 28px), 520px);
        padding: 12px 16px 0;
        transform: translateX(-50%);
        color: rgba(255, 255, 255, 0.68);
        font-size: 11px;
        font-weight: 700;
        letter-spacing: 0.1em;
        opacity: 0;
        pointer-events: none;
        text-transform: uppercase;
        visibility: hidden;
    }

    .append-to-picker-header.append-to-picker-header-visible {
        opacity: 1;
        visibility: visible;
    }

    :global(.append-to-picker-shell .palette) {
        padding-top: 24px;
    }
</style>
