<script>
  import { createEventDispatcher } from "svelte";

  export let canGoBack = false;

  const dispatch = createEventDispatcher();
</script>

<div class="bottom-bar">
  <button
    class="bottom-bar-btn"
    class:disabled={!canGoBack}
    type="button"
    title="Back"
    disabled={!canGoBack}
    on:mousedown|stopPropagation
    on:click|stopPropagation={() => canGoBack && dispatch("goBack")}
  >
    <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
      <path
        d="M10 3L5 8l5 5"
        stroke="currentColor"
        stroke-width="1.7"
        stroke-linecap="round"
        stroke-linejoin="round"
      />
    </svg>
  </button>
</div>

<style>
  .bottom-bar {
    display: flex;
    align-items: center;
    justify-content: flex-start;
    padding: 2px 10px 6px;
    gap: 4px;
    flex: 0 0 auto;
    min-height: 22px;
  }

  .bottom-bar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 24px;
    height: 24px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--app-text-color, #ffffff);
    opacity: 0.45;
    cursor: pointer;
    transition:
      opacity 0.15s ease,
      background 0.15s ease;
  }

  .bottom-bar-btn.disabled {
    opacity: 0.15;
    cursor: default;
    pointer-events: none;
  }

  .bottom-bar-btn:not(.disabled):hover {
    opacity: 0.85;
    background: rgba(255, 255, 255, 0.08);
  }

  .bottom-bar-btn:not(.disabled):active {
    opacity: 1;
    background: rgba(255, 255, 255, 0.12);
  }
</style>
