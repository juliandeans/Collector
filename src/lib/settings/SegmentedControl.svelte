<script>
    /**
     * @typedef {Object} SegmentOption
     * @property {string} label
     * @property {number|string} value
     */

    let { id = undefined, options = [], value, onSelect } = $props();
</script>

<div class="segmented-control" {id}>
    {#each options as option (option.value)}
        <button
            type="button"
            class="segment"
            class:active={option.value === value}
            aria-pressed={option.value === value}
            onclick={() => onSelect?.(option.value)}
        >
            {option.label}
        </button>
    {/each}
</div>

<style>
    .segmented-control {
        display: flex;
        gap: 2px;
        padding: 2px;
        border: 1.5px solid var(--settings-input-border, rgba(0, 0, 0, 0.15));
        border-radius: 8px;
        background: var(--settings-input-bg, #ffffff);
    }

    .segment {
        flex: 1;
        padding: 6px 0;
        border: none;
        border-radius: 6px;
        background: transparent;
        color: var(--settings-label, #111827);
        font-size: 13px;
        font-weight: 500;
        font-family: inherit;
        cursor: pointer;
        transition:
            background-color 0.15s ease,
            color 0.15s ease;
    }

    .segment:hover:not(.active) {
        background: color-mix(
            in srgb,
            var(--settings-accent, #8b5cf6) 10%,
            transparent
        );
    }

    .segment.active {
        background: var(--settings-accent, #8b5cf6);
        color: #fff;
    }

    .segment:focus-visible {
        outline: 2px solid var(--settings-accent, #8b5cf6);
        outline-offset: 2px;
    }
</style>
