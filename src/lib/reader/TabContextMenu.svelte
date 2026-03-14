<script>
  import { createEventDispatcher } from "svelte";

  export let open = false;
  export let x = 0;
  export let y = 0;

  const dispatch = createEventDispatcher();
</script>

{#if open}
  <div
    class="tab-context-backdrop"
    role="presentation"
    on:click={() => dispatch("close")}
    on:contextmenu|preventDefault={() => dispatch("close")}
  >
    <div
      class="tab-context-menu"
      role="menu"
      style={`left: ${x}px; top: ${y}px;`}
    >
      <button
        class="tab-context-item"
        type="button"
        role="menuitem"
        on:click={() => dispatch("closeTab")}
      >
        Close
      </button>
    </div>
  </div>
{/if}

<style>
  .tab-context-backdrop {
    position: absolute;
    inset: 0;
    z-index: 115;
  }

  .tab-context-menu {
    position: absolute;
    display: inline-flex;
    color: var(--app-text-color, #ffffff);
  }

  .tab-context-item {
    position: relative;
    width: 100%;
    display: flex;
    align-items: center;
    min-height: 32px;
    padding: 8px 10px;
    border-radius: 9px;
    text-align: left;
    cursor: pointer;
    color: inherit;
    font: inherit;
    border: 0;
    background: transparent;
    overflow: hidden;
    isolation: isolate;
    transition:
      background var(--transition-fast),
      transform var(--transition-fast);
  }

  .tab-context-item::before {
    content: "";
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(
      90deg,
      color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent),
      color-mix(in srgb, var(--accent-color, #8b5cf6) 35%, transparent),
      color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent)
    );
    background-size: 200% 100%;
    animation: shimmer 3s linear infinite;
    pointer-events: none;
    z-index: 2;
  }

  .tab-context-item::after {
    content: "";
    position: absolute;
    inset: 0;
    border-radius: inherit;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) 28%,
      transparent
    );
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.08),
      0 2px 8px rgba(0, 0, 0, 0.04);
    backdrop-filter: blur(28px) saturate(135%) brightness(0.92);
    -webkit-backdrop-filter: blur(28px) saturate(135%) brightness(0.92);
    pointer-events: none;
    z-index: 0;
  }

  .tab-context-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .tab-context-item:active {
    transform: translateY(1px);
  }

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }

    100% {
      background-position: -200% 0;
    }
  }
</style>
