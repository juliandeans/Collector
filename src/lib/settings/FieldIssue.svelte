<script>
    export let issue = null;
</script>

{#if issue?.message}
    <small class="field-issue {issue.type}">{issue.message}</small>
{/if}
