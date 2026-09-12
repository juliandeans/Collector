<script>
    /** @type {{ target?: string }} */
    let { target = "" } = $props();
</script>

{#if target}
    <div class="capture-rule-hint" title={target}>→ {target}</div>
{/if}

<style>
    .capture-rule-hint {
        /* Normaler Fluss direkt ueber der Action-Row: sitzt so immer direkt
           ueber den Buttons — oder am unteren Rand, wenn keine Buttons da
           sind. Kein absolute-Offset, das zur Action-Row-Abhaengigkeit
           festsitzt. */
        flex-shrink: 0;
        overflow: hidden;
        padding: 0 16px 2px;
        color: var(--accent-color, #8b5cf6);
        font-size: 11px;
        line-height: 1.3;
        text-align: center;
        text-overflow: ellipsis;
        white-space: nowrap;
        pointer-events: none;
    }
</style>
