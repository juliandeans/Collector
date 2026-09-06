<script>
    /**
     * @typedef {Object} Props
     * @property {any} [issue]
     */

    /** @type {Props} */
    let { issue = null } = $props();
</script>

{#if issue?.message}
    <small class="field-issue {issue.type}">{issue.message}</small>
{/if}
