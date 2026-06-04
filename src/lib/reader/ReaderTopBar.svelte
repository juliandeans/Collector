<script>
  import { createEventDispatcher } from "svelte";
  import { getReaderIconComponent } from "../reader-icons.js";

  export let tabs = [];
  export let activeTabIndex = 0;
  export let canOpenInObsidian = false;

  const dispatch = createEventDispatcher();

  function getTabIcon(tab) {
    return getReaderIconComponent(tab?.icon);
  }
</script>

<div class="accent-line" role="presentation"></div>

<div class="reader-topbar" data-tauri-drag-region>
  <div class="topbar-row">
    <div class="tab-strip">
      <button
        class="tab-action"
        type="button"
        title="Open Command Palette"
        on:mousedown|stopPropagation
        on:click|stopPropagation={() => dispatch("newTab")}
      >
        +
      </button>

      <div class="tab-list">
        {#each tabs as tab, index (tab.path)}
          <div class="tab-wrapper" class:pinned={tab.isPinned}>
            <button
              class="tab-button"
              class:active={index === activeTabIndex}
              class:pinned={tab.isPinned}
              type="button"
              title={tab.path}
              on:mousedown|stopPropagation
              on:click|stopPropagation={() => dispatch("activateTab", index)}
              on:contextmenu|stopPropagation={(event) =>
                dispatch("tabContextMenu", {
                  index,
                  x: event.clientX,
                  y: event.clientY,
                })}
            >
              {#if getTabIcon(tab)}
                <span class="tab-icon" aria-hidden="true">
                  <svelte:component
                    this={getTabIcon(tab)}
                    size={14}
                    strokeWidth={1.9}
                  />
                </span>
              {/if}
              <span class="tab-label">{tab.label}</span>
            </button>
            {#if !tab.isPinned}
              <button
                class="tab-close"
                type="button"
                aria-label={`Close ${tab.label}`}
                title="Close tab"
                on:mousedown|stopPropagation
                on:click|stopPropagation={() => dispatch("closeTab", index)}
              >
                ✕
              </button>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <div class="topbar-actions">
      {#if canOpenInObsidian}
        <button
          class="topbar-btn obsidian-btn"
          type="button"
          title="Open in Obsidian"
          on:mousedown|stopPropagation
          on:click|stopPropagation={() => dispatch("openInObsidian")}
        >
          <svg width="14" height="14" viewBox="0 0 100 100" fill="none">
            <path
              d="M73.8 13.8C67.4 7 58.4 3 49 3c-9.4 0-18.4 4-24.8 10.8L10 30.5c-6.4 6.8-9.5 16-8.6 25.2l2.8 28.7C5 93.5 11.5 99 19.2 99h61.6c7.7 0 14.2-5.5 15-13.1l2.8-28.7c.9-9.2-2.2-18.4-8.6-25.2L73.8 13.8z"
              fill="currentColor"
              opacity="0.9"
            />
            <path
              d="M50 25c-8.3 0-15 6.7-15 15s6.7 15 15 15 15-6.7 15-15-6.7-15-15-15zm0 22c-3.9 0-7-3.1-7-7s3.1-7 7-7 7 3.1 7 7-3.1 7-7 7z"
              fill="currentColor"
              opacity="0.6"
            />
          </svg>
        </button>
      {/if}

      <button
        class="topbar-btn close-btn"
        type="button"
        title="Close Reader"
        on:mousedown|stopPropagation
        on:click|stopPropagation={() => dispatch("closeReader")}
      >
        ✕
      </button>
    </div>
  </div>
</div>

<style>
  .accent-line {
    height: 2px;
    background: linear-gradient(
      90deg,
      color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent),
      color-mix(in srgb, var(--accent-color, #8b5cf6) 35%, transparent),
      color-mix(in srgb, var(--accent-color, #8b5cf6) 70%, transparent)
    );
    background-size: 200% 100%;
    animation: shimmer 3s linear infinite;
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .reader-topbar {
    display: flex;
    flex-direction: column;
    align-items: stretch;
    justify-content: flex-start;
    min-height: 40px;
    padding: 8px 12px 8px;
    gap: 6px;
    background: transparent;
    transition:
      filter 0.18s ease,
      opacity 0.18s ease,
      transform 0.18s ease;
  }

  .topbar-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    width: 100%;
    min-width: 0;
  }

  .tab-strip {
    display: flex;
    align-items: center;
    gap: 8px;
    min-width: 0;
    flex: 1;
  }

  .tab-list {
    display: flex;
    align-items: center;
    gap: 6px;
    min-width: 0;
    overflow-x: auto;
    scrollbar-width: none;
  }

  .tab-list::-webkit-scrollbar {
    display: none;
  }

  .tab-button,
  .tab-action,
  .tab-close {
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
  }

  .tab-wrapper {
    position: relative;
    flex: 0 0 auto;
    max-width: 120px;
    min-width: 0;
  }

  .tab-button {
    position: relative;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    width: 100%;
    min-width: 0;
    padding: 8px 10px 10px;
    border-radius: 10px 10px 0 0;
    color: rgba(255, 255, 255, 0.7);
    cursor: pointer;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-icon {
    flex: 0 0 auto;
    line-height: 1;
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }

  .tab-label {
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .tab-button.active {
    color: var(--app-text-color, #ffffff);
    background: rgba(255, 255, 255, 0.04);
  }

  .tab-button.active::after {
    content: "";
    position: absolute;
    left: 10px;
    right: 10px;
    bottom: 0;
    height: 2px;
    border-radius: 999px;
    background: var(--accent-color, #8b5cf6);
  }

  .tab-action {
    width: 26px;
    height: 26px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.05);
    cursor: pointer;
    transition:
      background var(--transition-fast),
      transform var(--transition-fast);
  }

  .topbar-actions {
    display: flex;
    align-items: center;
    gap: 2px;
    margin-left: auto;
    flex: 0 0 auto;
    -webkit-app-region: no-drag;
  }

  .topbar-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 26px;
    height: 26px;
    border: 0;
    border-radius: 5px;
    background: transparent;
    color: var(--app-text-color, #ffffff);
    fill: var(--app-text-color, #ffffff);
    stroke: var(--app-text-color, #ffffff);
    opacity: 0.75;
    font: inherit;
    font-size: 14px;
    cursor: pointer;
    transition:
      background var(--transition-fast),
      opacity var(--transition-fast);
  }

  .topbar-btn:hover {
    background: rgba(0, 0, 0, 0.08);
    opacity: 1;
  }

  .tab-action:hover,
  .tab-wrapper:hover .tab-button,
  .tab-button:hover {
    background: rgba(255, 255, 255, 0.08);
  }

  .tab-action:active {
    transform: translateY(1px);
  }

  .tab-close {
    position: absolute;
    top: 1px;
    right: 1px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    padding: 0;
    border: 0;
    border-radius: 50%;
    background: transparent;
    color: rgba(255, 255, 255, 0.6);
    font-size: 9px;
    line-height: 1;
    cursor: pointer;
    opacity: 0;
    transition:
      opacity 0.12s ease,
      background 0.12s ease;
    z-index: 2;
  }

  .tab-wrapper:hover .tab-close,
  .tab-close:focus-visible {
    opacity: 1;
  }

  .tab-close:hover {
    background: rgba(255, 255, 255, 0.08);
    color: var(--app-text-color, #ffffff);
  }

  .tab-close:active {
    background: rgba(255, 255, 255, 0.12);
  }
</style>
