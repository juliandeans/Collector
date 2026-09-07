<script>






  /**
   * @typedef {Object} Props
   * @property {boolean} [show]
   * @property {boolean} [isLoading]
   * @property {string} [saveToDailyShortcut]
   * @property {string} [saveAsNoteShortcut]
   * @property {string} [appendToNoteShortcut]
   * @property {() => void} [onDaily]
   * @property {() => void} [onNewNote]
   * @property {() => void} [onAppend]
   */

  /** @type {Props} */
  let {
    show = true,
    isLoading = false,
    saveToDailyShortcut = "",
    saveAsNoteShortcut = "",
    appendToNoteShortcut = "",
    onDaily,
    onNewNote,
    onAppend
  } = $props();
</script>

{#if show}
  <div class="action-bar">
    <button
      class="action-btn"
      onclick={() => onDaily?.()}
      disabled={isLoading}
      title={saveToDailyShortcut}
    >
      Daily Note
    </button>
    <button
      class="action-btn"
      onclick={() => onNewNote?.()}
      disabled={isLoading}
      title={saveAsNoteShortcut}
    >
      New Note
    </button>
    <button
      class="action-btn"
      onclick={() => onAppend?.()}
      disabled={isLoading}
      title={appendToNoteShortcut}
    >
      Append
    </button>
  </div>
{/if}

<style>
  .action-bar {
    display: flex;
    gap: 6px;
    padding: 4px 12px 6px;
    flex-shrink: 0;
    justify-content: center;
  }

  .action-btn {
    padding: 5px 10px;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: calc(var(--app-border-radius) / 2);
    background: transparent;
    color: rgba(255, 255, 255, 0.72);
    font-size: 11px;
    font-weight: 500;
    cursor: pointer;
    transition:
      border-color 0.15s ease,
      color 0.15s ease,
      background-color 0.15s ease,
      opacity 0.15s ease;
    letter-spacing: 0.2px;
    line-height: 1;
  }

  .action-btn:hover:not(:disabled) {
    color: rgba(255, 255, 255, 0.9);
    border-color: rgba(255, 255, 255, 0.28);
    background: rgba(255, 255, 255, 0.03);
  }

  .action-btn:active:not(:disabled) {
    background: rgba(255, 255, 255, 0.06);
    border-color: rgba(255, 255, 255, 0.32);
  }

  .action-btn:disabled {
    opacity: 0.45;
    cursor: default;
  }
</style>
