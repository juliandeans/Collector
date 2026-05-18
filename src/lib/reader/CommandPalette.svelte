<script>
  import { createEventDispatcher, tick } from "svelte";

  export let open = false;
  export let query = "";
  export let notes = [];
  export let selectedIndex = 0;
  export let inputRef;

  const dispatch = createEventDispatcher();
  let resultsRef;

  function ensureSelectedItemVisible() {
    if (!resultsRef || !open || notes.length === 0) return;

    const activeItem = resultsRef.querySelector(".palette-item.selected");
    if (!activeItem) return;

    const styles = getComputedStyle(resultsRef);
    const paddingTop = parseFloat(styles.paddingTop) || 0;
    const paddingBottom = parseFloat(styles.paddingBottom) || 0;
    const itemTop = activeItem.offsetTop;
    const itemBottom = itemTop + activeItem.offsetHeight;
    const viewTop = resultsRef.scrollTop;
    const viewBottom = viewTop + resultsRef.clientHeight;

    if (itemTop < viewTop) {
      resultsRef.scrollTop = Math.max(itemTop - paddingTop, 0);
      return;
    }

    if (itemBottom > viewBottom) {
      resultsRef.scrollTop = Math.max(
        itemBottom - resultsRef.clientHeight + paddingBottom,
        0,
      );
    }
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
        dispatch("selectIndex", Math.min(selectedIndex + 1, notes.length - 1));
      }
      return;
    }

    if (event.key === "ArrowUp") {
      event.preventDefault();
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
            on:mouseenter={() => dispatch("selectIndex", index)}
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
    border-radius: 12px;
    background: color-mix(
      in srgb,
      var(--app-background, #1e1e2e) var(--app-transparency, 55%),
      transparent
    );
    backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    -webkit-backdrop-filter: blur(var(--app-blur, 80px))
      saturate(var(--app-saturation, 200%)) var(--app-brightness-filter);
    color: var(--app-text-color, #ffffff);
    font-family: var(
      --app-font-family,
      -apple-system,
      BlinkMacSystemFont,
      "SF Pro Display",
      sans-serif
    );
    border: 0.5px solid rgba(0, 0, 0, 0.08);
    box-shadow:
      0 18px 48px rgba(0, 0, 0, 0.24),
      0 6px 18px rgba(0, 0, 0, 0.14);
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
    padding: 10px 8px 8px;
  }

  .palette-item {
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    display: flex;
    flex-direction: column;
    gap: 2px;
    width: 100%;
    padding: 10px 12px;
    border-radius: 12px;
    cursor: pointer;
    text-align: left;
    transition: background var(--transition-fast);
  }

  .palette-item:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .palette-item.selected {
    background: rgba(255, 255, 255, 0.08);
  }

  .palette-name {
    color: var(--app-text-color, #ffffff);
    font-weight: 600;
  }

  .palette-path {
    margin-top: 2px;
    color: rgba(255, 255, 255, 0.58);
    font-size: 12px;
  }

  .palette-empty {
    padding: 18px 12px;
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
