<script>
  import { createEventDispatcher, tick } from "svelte";

  export let open = false;
  export let query = "";
  export let notes = [];
  export let selectedIndex = 0;
  export let inputRef;

  const dispatch = createEventDispatcher();
  let resultsRef;
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

    dispatch("selectIndex", index);
  }

  $: if (open && notes.length > 0 && selectedIndex >= 0) {
    void tick().then(() => {
      ensureSelectedItemVisible();
    });
  }

  function handleInput(event) {
    dispatch("queryChange", event);
  }

  function handleKeydown(event) {
    event.stopPropagation();

    if (event.key === "ArrowDown") {
      event.preventDefault();
      if (notes.length > 0) {
        suppressPointerSelection();
        dispatch("selectIndex", Math.min(selectedIndex + 1, notes.length - 1));
      }
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
      suppressPointerSelection();
      dispatch("selectIndex", Math.max(selectedIndex - 1, 0));
      return;
    }

    if (event.key === "Enter") {
      event.preventDefault();
      const note = notes[selectedIndex];
      if (note) {
        dispatch("openNote", note);
      }
      return;
    }

    if (event.key === "Escape") {
      event.preventDefault();
      dispatch("close");
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
    on:mousedown={() => dispatch("close")}
  />

  <div class="palette" role="dialog" aria-modal="true">
    <input
      bind:this={inputRef}
      value={query}
      class="palette-input"
      placeholder="Search vault notes..."
      spellcheck="false"
      on:input={handleInput}
      on:keydown={handleKeydown}
    />

    <div class="palette-results" bind:this={resultsRef}>
      {#if notes.length === 0}
        <div class="palette-empty">No matching notes</div>
      {:else}
        {#each notes as note, index (note.path)}
          <button
            class="palette-item"
            class:selected={index === selectedIndex}
            type="button"
            on:pointermove={(event) => handlePointerMove(index, event)}
            on:click={() => dispatch("openNote", note)}
          >
            <span class="palette-name">{note.name}</span>
            <span class="palette-path">{note.relative_path}</span>
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
    border-radius: var(--app-border-radius, 12px);
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) var(--app-transparency, 85%),
      transparent
    );
    backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    -webkit-backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    color: var(--app-text-color, #ffffff);
    font-family: var(--app-font-family, var(--font-family));
    border: 1px solid rgba(255, 255, 255, 0.08);
    box-shadow: var(--overlay-shadow);
    transform: translateX(-50%) translateZ(0);
    -webkit-transform: translateX(-50%) translateZ(0);
    z-index: 121;
  }

  .palette::before {
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
    z-index: 1;
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
    background: transparent;
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

  .palette-item:hover,
  .palette-item.selected {
    background: color-mix(
      in srgb,
      var(--accent-color, #8b5cf6) 14%,
      transparent
    );
  }

  .palette-name {
    color: var(--app-text-color, #ffffff);
    font-weight: 500;
    font-size: 13px;
  }

  .palette-path {
    margin-top: 2px;
    color: var(--text-secondary);
    font-size: 10px;
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

  @keyframes shimmer {
    0% {
      background-position: 200% 0;
    }

    100% {
      background-position: -200% 0;
    }
  }
</style>
