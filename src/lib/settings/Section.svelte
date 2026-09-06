<script>
    let { title, children } = $props();
    let collapsed = $state(false);

    function toggle() {
        collapsed = !collapsed;
    }
</script>

<section class:collapsed>
    <button type="button" class="section-header" onclick={toggle}>
        <span class="section-title">{title}</span>
    </button>
    {#if !collapsed}
        <div class="section-body">
            {@render children?.()}
        </div>
    {/if}
</section>

<style>
    section {
        border-top: 1px solid var(--settings-border, rgba(0, 0, 0, 0.07));
        padding: 0 0 12px 0;
    }

    section:first-of-type {
        border-top: none;
    }

    section.collapsed {
        padding-bottom: 0;
    }

    .section-header {
        display: flex;
        align-items: center;
        gap: 6px;
        width: 100%;
        padding: 22px 0 6px;
        border: none;
        background: none;
        cursor: pointer;
        font-family: inherit;
        text-align: left;
        color: inherit;
    }

    .section-header:hover {
        opacity: 0.8;
    }

    .section-title {
        font-size: 16px;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.06em;
        color: var(--settings-section-title, #9ca3af);
    }

    .section-body {
        padding-top: 2px;
    }
</style>
