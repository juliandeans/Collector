<script>
  let {
    show = false,
    isOcrActive = false,
    hint = "",
  } = $props();
</script>

{#if show}
  {#if isOcrActive}
    <div class="ocr-overlay ocr-active">
      <div class="ocr-badge">
        <svg class="ocr-icon" viewBox="0 0 24 24" width="20" height="20" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
          <path d="M4 7V4h3" />
          <path d="M20 7V4h-3" />
          <path d="M4 17v3h3" />
          <path d="M20 17v3h-3" />
          <line x1="9" y1="12" x2="15" y2="12" />
          <line x1="12" y1="9" x2="12" y2="15" />
        </svg>
        <span class="ocr-text">Extract text</span>
      </div>
    </div>
  {:else if hint}
    <div class="ocr-overlay ocr-hint">
      <div class="ocr-badge hint-badge">
        <span>{hint}</span>
      </div>
    </div>
  {/if}
{/if}

<style>
  .ocr-overlay {
    position: absolute;
    inset: 2px;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
    z-index: 999;
    border-radius: 12px;
    animation: fadeIn 0.15s ease-out;
  }

  .ocr-active {
    background: rgba(139, 92, 246, 0.12);
    border: 2px dashed rgba(139, 92, 246, 0.85);
    backdrop-filter: blur(2px);
  }

  .ocr-hint {
    background: rgba(0, 0, 0, 0.05);
    border: 1px dashed rgba(255, 255, 255, 0.25);
  }

  .ocr-badge {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(139, 92, 246, 0.9);
    color: #ffffff;
    padding: 8px 16px;
    border-radius: 20px;
    font-size: 13px;
    font-weight: 600;
    letter-spacing: 0.2px;
    box-shadow: 0 4px 16px rgba(0, 0, 0, 0.25);
  }

  .hint-badge {
    background: rgba(30, 30, 30, 0.75);
    color: rgba(255, 255, 255, 0.85);
    font-size: 12px;
    font-weight: 500;
    border: 1px solid rgba(255, 255, 255, 0.15);
  }

  .ocr-icon {
    flex-shrink: 0;
  }

  @keyframes fadeIn {
    from {
      opacity: 0;
      transform: scale(0.98);
    }
    to {
      opacity: 1;
      transform: scale(1);
    }
  }
</style>
