<script>
    import { X } from "@lucide/svelte";
    import { handleShortcutKeyDown } from "./shortcut-utils.js";

    let {
        id = "",
        value = $bindable(""),
        placeholder = "None",
        onChange = () => {}
    } = $props();

    let inputEl;

    function onKeyDown(event) {
        handleShortcutKeyDown(
            event,
            (val) => {
                value = val;
            },
            null,
            onChange
        );
    }

    function handleClear(event) {
        event.preventDefault();
        event.stopPropagation();
        value = "";
        onChange();
        inputEl?.focus();
    }
</script>

<div class="shortcut-input-wrapper">
    <input
        bind:this={inputEl}
        class="shortcut-input"
        type="text"
        {id}
        bind:value
        {placeholder}
        onkeydown={onKeyDown}
    />
    {#if value && value.trim().length > 0}
        <button
            type="button"
            class="clear-btn"
            tabindex="-1"
            title="Shortcut entfernen (Backspace)"
            aria-label="Shortcut entfernen"
            onclick={handleClear}
        >
            <X size={12} strokeWidth={2.5} />
        </button>
    {/if}
</div>

<style>
    .shortcut-input-wrapper {
        position: relative;
        display: inline-flex;
        align-items: center;
        width: 280px;
        max-width: 100%;
    }

    .shortcut-input {
        width: 100%;
        padding-right: 30px !important;
    }

    .clear-btn {
        position: absolute;
        right: 7px;
        top: 50%;
        transform: translateY(-50%);
        display: inline-flex;
        align-items: center;
        justify-content: center;
        width: 18px;
        height: 18px;
        padding: 0;
        border: none;
        border-radius: 50%;
        background: rgba(128, 128, 128, 0.18);
        color: var(--settings-text-secondary, #9ca3af);
        cursor: pointer;
        opacity: 0;
        pointer-events: none;
        transition: opacity 0.15s ease, background 0.15s, color 0.15s, transform 0.1s;
    }

    .shortcut-input-wrapper:hover .clear-btn,
    .shortcut-input-wrapper:focus-within .clear-btn {
        opacity: 1;
        pointer-events: auto;
    }

    .clear-btn:hover {
        background: rgba(128, 128, 128, 0.35);
        color: var(--settings-text, #ffffff);
    }

    .clear-btn:active {
        transform: translateY(-50%) scale(0.92);
    }
</style>
