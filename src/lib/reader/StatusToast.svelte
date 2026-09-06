<script>
  /**
   * @typedef {Object} Props
   * @property {string} [message]
   * @property {string} [type]
   */

  /** @type {Props} */
  let { message = "", type = "" } = $props();
</script>

{#if message}
  <div class="status-toast" class:error={type === "error"}>
    {message}
  </div>
{/if}

<style>
  .status-toast {
    position: absolute;
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
    padding: 8px 16px;
    background: var(--success-bg);
    -webkit-backdrop-filter: blur(20px);
    backdrop-filter: blur(20px);
    border: 0.5px solid var(--success-border);
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    color: var(--success-color);
    animation: fadeInUp 0.2s ease-out;
    white-space: nowrap;
    z-index: 100;
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .status-toast.error {
    background: var(--error-bg);
    border-color: var(--error-border);
    color: var(--error-color);
  }

  @keyframes fadeInUp {
    from {
      opacity: 0;
      transform: translateX(-50%) translateY(10px);
    }

    to {
      opacity: 1;
      transform: translateX(-50%) translateY(0);
    }
  }
</style>
