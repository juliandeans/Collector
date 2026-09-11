<script>
  import { tick } from "svelte";
  import "../overlay.css";

  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {string} [query]
   * @property {any} [notes]
   * @property {number} [selectedIndex]
   * @property {boolean} [showPaths]
   * @property {any} inputRef
   * @property {any} [onSelectIndex]
   * @property {any} [onQueryChange]
   * @property {any} [onOpenNote]
   * @property {any} [onClose]
   */

  /** @type {Props} */
  let {
    open = false,
    query = "",
    notes = [],
    selectedIndex = 0,
    showPaths = true,
    inputRef = $bindable(),
    onSelectIndex,
    onQueryChange,
    onOpenNote,
    onClose
  } = $props();

  let resultsRef = $state();
  let suppressPointerUntil = 0;
  let lastPointerX = null;
  let lastPointerY = null;

  function ensureSelectedItemVisible() {
    if (!resultsRef || !open || notes.length === 0) return;

    const activeItem = resultsRef.querySelector(".palette-item.selected");
    if (!activeItem) return;

    const itemTop = activeItem.offsetTop;
    const itemBottom = itemTop + activeItem.offsetHeight;
    const viewTop = resultsRef.scrollTop;
    const viewBottom = viewTop + resultsRef.clientHeight;

    if (itemTop < viewTop) {
      suppressPointerSelection();
      resultsRef.scrollTop = itemTop;
      return;
    }

    if (itemBottom > viewBottom) {
      suppressPointerSelection();
      resultsRef.scrollTop = itemBottom - resultsRef.clientHeight;
    }
  }

  function suppressPointerSelection() {
    suppressPointerUntil = performance.now() + 180;
  }

  function handlePointerMove(index, event) {
    const samePointerPosition =
      event.clientX === lastPointerX && event.clientY === lastPointerY;
    const stationaryFirstMove =
      lastPointerX === null &&
      event.movementX === 0 &&
      event.movementY === 0;

    if (samePointerPosition || stationaryFirstMove) {
      lastPointerX = event.clientX;
      lastPointerY = event.clientY;
      return;
    }

    lastPointerX = event.clientX;
    lastPointerY = event.clientY;

    if (index === selectedIndex) return;
    if (performance.now() < suppressPointerUntil) return;

    onSelectIndex?.(index);
  }

  $effect.pre(() => {
    if (open && notes.length > 0 && selectedIndex >= 0) {
      void tick().then(() => {
        ensureSelectedItemVisible();
      });
    }
  });

  function handleInput(event) {
    onQueryChange?.(event);
  }

  function handleKeydown(event) {
    event.stopPropagation();

    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (notes.length > 0) {
        suppressPointerSelection();
        onSelectIndex?.(Math.min(selectedIndex + 1, notes.length - 1));
      }
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      suppressPointerSelection();
      onSelectIndex?.(Math.max(selectedIndex - 1, 0));
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      const note = notes[selectedIndex];
      if (note) {
        onOpenNote?.(note);
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      onClose?.();
    }
  }
</script>

<div
  class="palette-wrapper"
  class:palette-visible={open}
  aria-hidden={!open}
>
  <button
    type="button"
    class="palette-backdrop"
    aria-label="Close palette"
    tabindex="-1"
    onmousedown={() => onClose?.()}
  ></button>

  <div class="palette overlay-panel" role="dialog" aria-modal="true">
    <input
      bind:this={inputRef}
      value={query}
      class="palette-input"
      placeholder="Search vault notes..."
      spellcheck="false"
      oninput={handleInput}
      onkeydown={handleKeydown}
    />

    <div class="palette-results" bind:this={resultsRef}>
      {#if notes.length === 0}
        <div class="palette-empty">No matching notes</div>
      {:else}
        {#each notes as note, index (note.path)}
          <button
            class="palette-item overlay-item"
            class:selected={index === selectedIndex}
            type="button"
            onpointermove={(event) => handlePointerMove(index, event)}
            onclick={() => onOpenNote?.(note)}
          >
            <span class="overlay-name">{note.name}</span>
            {#if showPaths}
              <span class="overlay-path">{note.relative_path}</span>
            {/if}
          </button>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .palette-wrapper {
    position: fixed;
    inset: 0;
    z-index: 1000;
    pointer-events: none;
    opacity: 0;
  }

  .palette-wrapper.palette-visible {
    pointer-events: auto;
    opacity: 1;
  }

  .palette-backdrop {
    position: absolute;
    inset: 0;
    border: 0;
    background: rgba(8, 10, 14, 0.22);
    padding: 0;
    appearance: none;
    z-index: 120;
  }

  .palette {
    position: absolute;
    top: 52px;
    left: 50%;
    width: min(calc(100% - 28px), 520px);
    max-width: 520px;
    max-height: min(70vh, 520px);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    transform: translateX(-50%) translateZ(0);
    -webkit-transform: translateX(-50%) translateZ(0);
    z-index: 121;
  }

  .palette-input {
    width: 100%;
    padding: 14px 16px 12px;
    border: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    background: transparent;
    color: var(--app-text-color, #ffffff);
    font: inherit;
    outline: none;
    position: relative;
    z-index: 1;
  }

  .palette-input::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }

  .palette-results {
    flex: 1;
    max-height: 420px;
    overflow-y: auto;
    padding: 0;
  }

  .palette-item {
    border: 0;
    border-bottom: 1px solid rgba(255, 255, 255, 0.06);
    color: inherit;
    font: inherit;
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 8px 12px;
    border-radius: 0;
    cursor: pointer;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .palette-item:last-child {
    border-bottom: 0;
  }

  .palette-empty {
    padding: 18px 16px;
    color: rgba(255, 255, 255, 0.58);
    text-align: center;
  }

  .palette-results::-webkit-scrollbar {
    width: 6px;
  }

  .palette-results::-webkit-scrollbar-track {
    background: transparent;
  }

  .palette-results::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.12);
    border-radius: 3px;
  }
</style>
