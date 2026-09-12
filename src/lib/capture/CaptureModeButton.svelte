<script>
    let {
        show = false,
        active = false,
        shortcut = "",
        onToggle,
    } = $props();

    const shortcutLabel = $derived(shortcut || "no shortcut");
</script>

{#if show}
    <button
        class="capture-mode-button"
        class:active
        type="button"
        aria-pressed={active}
        title={`Collect copied text · ${shortcutLabel}`}
        onclick={() => onToggle?.()}
    >
        <span class="state-indicator" aria-hidden="true">{active ? "●" : "○"}</span>
        {active ? "Collecting" : "Collect"}
    </button>
{/if}

<style>
    .capture-mode-button {
        align-self: flex-start;
        flex-shrink: 0;
        margin: 0 12px 5px;
        padding: 5px 9px;
        border: 1px solid rgba(255, 255, 255, 0.16);
        border-radius: calc(var(--app-border-radius) / 2);
        background: transparent;
        color: rgba(255, 255, 255, 0.72);
        font: inherit;
        font-size: 11px;
        font-weight: 500;
        line-height: 1;
        letter-spacing: 0.2px;
        cursor: pointer;
        transition:
            border-color 0.15s ease,
            color 0.15s ease,
            background-color 0.15s ease;
    }

    .capture-mode-button:hover {
        color: rgba(255, 255, 255, 0.9);
        border-color: rgba(255, 255, 255, 0.28);
        background: rgba(255, 255, 255, 0.03);
    }

    .capture-mode-button.active {
        color: var(--accent-color, #8b5cf6);
        border-color: color-mix(in srgb, var(--accent-color, #8b5cf6) 55%, transparent);
        background: color-mix(in srgb, var(--accent-color, #8b5cf6) 12%, transparent);
    }

    .state-indicator {
        margin-right: 4px;
    }
</style>
