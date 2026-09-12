<script>
    let {
        id,
        label,
        value = $bindable(),
        min,
        max,
        step = 1,
        formatValue = (v) => v,
        showZeroTick = false,
    } = $props();

    const zeroTickPercent = $derived(((0 - min) / (max - min)) * 100);
</script>

<label for={id}>{label}</label>
<div class="slider-row">
    <div class="slider-track-wrap">
        <input type="range" {id} bind:value {min} {max} {step} />
        {#if showZeroTick}
            <span class="zero-tick" style={`left: ${zeroTickPercent}%`}></span>
        {/if}
    </div>
    <span class="slider-value">{formatValue(value)}</span>
</div>

<style>
    .slider-row {
        display: flex;
        align-items: center;
        gap: 10px;
    }

    .slider-track-wrap {
        position: relative;
        flex: 1;
    }

    .slider-value {
        min-width: 3em;
        text-align: right;
        font-size: 13px;
        font-weight: 500;
        color: var(--settings-label, #111827);
        font-variant-numeric: tabular-nums;
    }

    .zero-tick {
        position: absolute;
        top: 50%;
        width: 2px;
        height: 8px;
        transform: translate(-50%, -50%);
        background: var(--settings-input-border, rgba(0, 0, 0, 0.35));
        pointer-events: none;
    }

    input[type="range"] {
        width: 100%;
        margin: 4px 0;
        -webkit-appearance: none;
        appearance: none;
        background: transparent;
    }

    input[type="range"]::-webkit-slider-runnable-track {
        height: 4px;
        border-radius: 2px;
        background: var(--settings-input-border, rgba(0, 0, 0, 0.15));
    }

    input[type="range"]::-webkit-slider-thumb {
        -webkit-appearance: none;
        appearance: none;
        width: 14px;
        height: 14px;
        border-radius: 50%;
        background: var(--settings-text, #1a1a1a);
        margin-top: -5px;
        border: none;
        cursor: pointer;
    }

    input[type="range"]:focus {
        outline: none;
    }

    input[type="range"]:focus::-webkit-slider-thumb {
        box-shadow:
            0 0 0 2px var(--settings-input-bg, white),
            0 0 0 4px rgba(139, 92, 246, 0.3);
    }
</style>
