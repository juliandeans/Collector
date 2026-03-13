<script>
  export let message = "";
  export let type = "";
</script>

{#if message && type === "error"}
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
    background: rgba(52, 199, 89, 0.12);
    backdrop-filter: blur(20px);
    border: 0.5px solid rgba(52, 199, 89, 0.3);
    border-radius: 8px;
    font-size: 12px;
    font-weight: 600;
    color: #34c759;
    animation: fadeInUp 0.2s ease-out;
    white-space: nowrap;
    z-index: 100;
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .status-toast.error {
    background: rgba(255, 59, 48, 0.12);
    border-color: rgba(255, 59, 48, 0.3);
    color: #ff3b30;
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
