<script>
  import { invoke } from "@tauri-apps/api/core";
  import { tick } from "svelte";

  /**
   * @typedef {Object} Props
   * @property {boolean} [show]
   * @property {string[]} [excludedApps]
   * @property {(app: string) => void} [onAdd]
   * @property {() => void} [onClose]
   */

  /** @type {Props} */
  let { show = false, excludedApps = [], onAdd, onClose } = $props();

  let allApps = $state([]);
  let runningSet = $state(new Set());
  let loadedAll = $state(false);
  let query = $state("");
  let selectedIndex = $state(0);
  let inputRef = $state();




  async function loadAllApps() {
    loadedAll = true;
    const [running, installed] = await Promise.all([
      invoke("get_running_apps").catch(() => []),
      invoke("list_applications").catch(() => []),
    ]);

    runningSet = new Set(running);
    const seen = new Set();
    allApps = [...running, ...installed].filter((app) => {
      if (seen.has(app)) return false;
      seen.add(app);
      return true;
    });
  }

  function addApp(app) {
    if (excludedApps.includes(app)) return;
    onAdd?.(app);
  }

  function chooseApp() {
    // Choose is now just a shortcut — all installed apps are already in the list.
    // Focus the search so the user can filter and pick.
    inputRef?.focus();
    inputRef?.select();
  }

  function handleSearchInput() {
    selectedIndex = 0;
  }

  function handleKeydown(event) {
    if (event.key === "ArrowDown") {
      event.preventDefault();
      selectedIndex = Math.min(selectedIndex + 1, filteredApps.length - 1);
    } else if (event.key === "ArrowUp") {
      event.preventDefault();
      selectedIndex = Math.max(selectedIndex - 1, 0);
    } else if (event.key === "Enter") {
      event.preventDefault();
      const app = filteredApps[selectedIndex];
      if (app) addApp(app);
    } else if (event.key === "Escape") {
      event.preventDefault();
      close();
    }
  }

  function close() {
    onClose?.();
  }


  let filteredApps = $derived(query
    ? allApps.filter((app) =>
        app.toLowerCase().includes(query.trim().toLowerCase()),
      )
    : allApps);
  $effect.pre(() => {
    if (show) {
      void tick().then(() => {
        inputRef?.focus();
      });
    }
  });
  $effect.pre(() => {
    if (show && !loadedAll) {
      loadAllApps();
    }
  });
</script>

{#if show}
  <!-- svelte-ignore a11y_click_events_have_key_events, a11y_no_static_element_interactions -->
  <div class="picker-backdrop" onclick={close}>
    <div
      class="picker-panel"
      role="dialog"
      aria-modal="true"
      tabindex="-1"
      onclick={(event) => event.stopPropagation()}
      onkeydown={handleKeydown}
    >
      <div class="picker-search-row">
        <input
          bind:this={inputRef}
          bind:value={query}
          class="picker-search"
          placeholder="Search apps…"
          spellcheck="false"
          oninput={handleSearchInput}
          onkeydown={handleKeydown}
        />
        <button
          class="choose-btn"
          type="button"
          title="Browse all installed applications"
          onclick={chooseApp}
        >
          All Apps
        </button>
      </div>

      <div class="picker-results">
        {#if filteredApps.length === 0}
          <div class="picker-empty">
            {query ? "No matching apps" : "No applications found"}
          </div>
        {:else}
          {#each filteredApps as app, idx}
            {#if !query && idx > 0 && runningSet.has(filteredApps[idx - 1]) && !runningSet.has(app)}
              <div class="picker-divider"></div>
            {/if}
            <button
              class="picker-item"
              class:selected={idx === selectedIndex}
              type="button"
              onclick={() => addApp(app)}
              onpointermove={() => {
                selectedIndex = idx;
              }}
            >
              <span class="picker-name">{app}</span>
              {#if excludedApps.includes(app)}
                <span class="picker-check">✓</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  .picker-backdrop {
    position: fixed;
    inset: 0;
    z-index: 1000;
    background: rgba(0, 0, 0, 0.22);
    display: flex;
    align-items: flex-start;
    justify-content: center;
    padding-top: 60px;
  }

  .picker-panel {
    width: min(calc(100% - 28px), 480px);
    max-height: 420px;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    border-radius: 12px;
    background: var(--settings-bg, #f2f2f2);
    border: 1px solid var(--settings-border, rgba(0, 0, 0, 0.08));
    box-shadow: var(--overlay-shadow);
  }

  .picker-search-row {
    display: flex;
    border-bottom: 1px solid var(--settings-border, rgba(0, 0, 0, 0.08));
  }

  .picker-search {
    flex: 1;
    min-width: 0;
    padding: 12px 14px;
    border: 0;
    background: transparent;
    color: inherit;
    font: inherit;
    font-size: 14px;
    outline: none;
  }

  .picker-search::placeholder {
    color: var(--settings-text-secondary, #6b7280);
  }

  .choose-btn {
    flex-shrink: 0;
    margin: 6px;
    padding: 4px 14px;
    border: 1px solid var(--settings-border, rgba(0, 0, 0, 0.1));
    border-radius: 6px;
    background: var(--settings-btn-bg, rgba(0, 0, 0, 0.07));
    color: var(--settings-btn-text, inherit);
    font-size: 12px;
    font-weight: 500;
    cursor: pointer;
    transition: background 0.1s;
  }

  .choose-btn:hover {
    background: rgba(128, 128, 128, 0.18);
  }

  .picker-results {
    flex: 1;
    overflow-y: auto;
    max-height: 360px;
  }

  .picker-item {
    border: 0;
    border-bottom: 1px solid var(--settings-border, rgba(0, 0, 0, 0.06));
    background: transparent;
    color: inherit;
    font: inherit;
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 14px;
    cursor: pointer;
    text-align: left;
    font-size: 13px;
    transition: background 0.1s;
  }

  .picker-item:last-child {
    border-bottom: 0;
  }

  .picker-item.selected,
  .picker-item:hover {
    background: color-mix(
      in srgb,
      var(--settings-accent, #8b5cf6) 14%,
      transparent
    );
  }

  .picker-name {
    font-weight: 500;
    font-size: 13px;
  }

  .picker-check {
    color: #22c55e;
    font-size: 12px;
  }

  .picker-divider {
    height: 3px;
    margin: 10px 0;
    background: var(--settings-border, rgba(0, 0, 0, 0.08));
  }

  .picker-empty {
    padding: 18px 14px;
    color: var(--settings-text-secondary, #6b7280);
    text-align: center;
    font-size: 13px;
  }

  .picker-results::-webkit-scrollbar {
    width: 6px;
  }

  .picker-results::-webkit-scrollbar-track {
    background: transparent;
  }

  .picker-results::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.12);
    border-radius: 3px;
  }
</style>