<script>

  /**
   * @typedef {Object} Props
   * @property {boolean} [active]
   */

  /** @type {Props} */
  let { active = false } = $props();
</script>

{#if active}
  <div class="loading-indicator"></div>
{/if}

<style>
  .loading-indicator {
    position: absolute;
    top: 2px;
    left: 0;
    right: 0;
    height: 2px;
    background: linear-gradient(
      90deg,
      transparent,
      color-mix(in srgb, var(--accent-color, #8b5cf6) 85%, transparent),
      transparent
    );
    background-size: 200% 100%;
    animation: loading 1s ease-in-out infinite;
    z-index: 100;
  }

  @keyframes loading {
    0% {
      background-position: 200% 0;
    }
    100% {
      background-position: -200% 0;
    }
  }
</style>
