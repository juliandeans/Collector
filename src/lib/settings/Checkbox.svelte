<script>
    import { createBubbler } from 'svelte/legacy';

    const bubble = createBubbler();
    
    /**
     * @typedef {Object} Props
     * @property {boolean} [checked] - Custom styled checkbox – adapts to light/dark mode via CSS vars.
Wraps a hidden native input for accessibility and form behaviour.
Props mirror a native checkbox: checked, disabled, id, name, value.
`on:change` works as usual.
     * @property {boolean} [disabled]
     * @property {any} [id]
     * @property {any} [name]
     * @property {any} [value]
     * @property {import('svelte').Snippet} [children]
     */

    /** @type {Props} */
    let {
        checked = $bindable(false),
        disabled = false,
        id = undefined,
        name = undefined,
        value = undefined,
        children
    } = $props();
</script>

<label class="checkbox-wrapper" class:disabled>
    <input
        type="checkbox"
        {id}
        {name}
        {value}
        bind:checked
        {disabled}
        onchange={bubble('change')}
        onclick={bubble('click')}
        onfocus={bubble('focus')}
        onblur={bubble('blur')}
    />
    <span class="checkbox-visual" aria-hidden="true">
        {#if checked}
            <svg
                viewBox="0 0 16 16"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
            >
                <path
                    d="M3 8.5L6.5 12L13 4.5"
                    stroke="currentColor"
                    stroke-width="2.5"
                    stroke-linecap="round"
                    stroke-linejoin="round"
                />
            </svg>
        {/if}
    </span>
    <span class="checkbox-label">
        {@render children?.()}
    </span>
</label>

<style>
    .checkbox-wrapper {
        display: inline-flex;
        align-items: center;
        gap: 8px;
        cursor: pointer;
        user-select: none;
        -webkit-user-select: none;
    }

    .checkbox-wrapper.disabled {
        opacity: 0.45;
        cursor: not-allowed;
    }

    .checkbox-wrapper input {
        position: absolute;
        opacity: 0;
        width: 0;
        height: 0;
        pointer-events: none;
    }

    .checkbox-visual {
        flex-shrink: 0;
        display: flex;
        align-items: center;
        justify-content: center;
        width: 18px;
        height: 18px;
        border-radius: 6px;
        border: 1.5px solid var(--settings-input-border, rgba(0, 0, 0, 0.12));
        background: var(--settings-input-bg, #ffffff);
        color: transparent;
        transition: all 0.18s ease;
        box-sizing: border-box;
    }

    .checkbox-visual svg {
        width: 14px;
        height: 14px;
        stroke: white;
        filter: drop-shadow(0 0 1px rgba(0, 0, 0, 0.08));
    }

    /* hover */
    .checkbox-wrapper:not(.disabled) input:not(:checked) ~ .checkbox-visual:hover {
        border-color: var(--settings-accent, #8b5cf6);
        background: color-mix(
            in srgb,
            var(--settings-accent, #8b5cf6) 8%,
            transparent
        );
    }

    /* checked */
    .checkbox-wrapper input:checked ~ .checkbox-visual {
        border-color: var(--settings-accent, #8b5cf6);
        background: var(--settings-accent, #8b5cf6);
        color: #fff;
    }

    /* hover on checked */
    .checkbox-wrapper:not(.disabled)
        input:checked
        ~ .checkbox-visual:hover {
        filter: brightness(1.1);
    }

    /* focus ring */
    .checkbox-wrapper input:focus-visible ~ .checkbox-visual {
        outline: 2px solid var(--settings-accent, #8b5cf6);
        outline-offset: 2px;
    }

    .checkbox-label {
        font-size: 13px;
        font-weight: 500;
        color: var(--settings-label, #111827);
        line-height: 1.4;
    }
</style>
