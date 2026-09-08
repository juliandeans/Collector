<script>
  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {string} [query]
   * @property {number} [matchCount]
   * @property {number} [activeIndex]
   * @property {any} inputRef
   * @property {any} [onStep]
   * @property {any} [onClose]
   * @property {any} [onQueryChange]
   */

  /** @type {Props} */
  let {
    open = false,
    query = "",
    matchCount = 0,
    activeIndex = 0,
    inputRef = $bindable(),
    onStep,
    onClose,
    onQueryChange
  } = $props();

  function handleKeydown(event) {
    if (event.key === "Enter") {
      event.preventDefault();
      onStep?.(event.shiftKey ? -1 : 1);
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      onClose?.();
    }
  }
</script>

{#if open}
  <div class="search-bar">
    <input
      bind:this={inputRef}
      value={query}
      class="search-input"
      placeholder="Search…"
      oninput={(event) => onQueryChange?.(event)}
      onkeydown={handleKeydown}
    />
    <span class="search-count">
      {matchCount > 0
        ? `${activeIndex + 1} of ${matchCount}`
        : query
          ? "0 results"
          : ""}
    </span>
    <button class="search-nav" type="button" onclick={() => onStep?.(-1)}
      >↑</button
    >
    <button class="search-nav" type="button" onclick={() => onStep?.(1)}
      >↓</button
    >
    <button class="search-close" type="button" onclick={() => onClose?.()}
      >✕</button
    >
  </div>
{/if}

<style>
  .search-bar {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 12px;
    flex-shrink: 0;
    width: 100%;
    border-radius: 10px;
    background: rgba(255, 255, 255, 0.04);
    border: 1px solid rgba(255, 255, 255, 0.06);
    -webkit-app-region: no-drag;
  }

  .search-input {
    flex: 1;
    min-width: 0;
    border: 0;
    background: transparent;
    color: var(--app-text-color, #ffffff);
    font: inherit;
    font-size: 13px;
    outline: none;
    -webkit-app-region: no-drag;
  }

  .search-count {
    min-width: 56px;
    color: var(--text-secondary);
    font-size: 11px;
    text-align: right;
    white-space: nowrap;
  }

  .search-nav,
  .search-close {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 22px;
    height: 22px;
    border: 0;
    border-radius: 4px;
    background: transparent;
    color: var(--text-secondary);
    cursor: pointer;
    font-size: 12px;
    transition: background var(--transition-fast);
    -webkit-app-region: no-drag;
  }

  .search-nav:hover,
  .search-close:hover {
    background: rgba(0, 0, 0, 0.08);
    color: var(--app-text-color, #ffffff);
  }
</style>
