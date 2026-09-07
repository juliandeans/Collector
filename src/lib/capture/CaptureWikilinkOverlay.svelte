<script>
  import WikilinkPicker from "../WikilinkPicker.svelte";




  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {Array} [matches]
   * @property {number} [selectedIndex]
   * @property {{ left: number, top: number }} [position]
   * @property {boolean} [showPaths]
   * @property {(index: number) => void} [onHover]
   * @property {(note: any) => void} [onSelect]
   */

  /** @type {Props} */
  let {
    open = false,
    matches = [],
    selectedIndex = 0,
    position = { left: 0, top: 0 },
    showPaths = true,
    onHover = () => {},
    onSelect
  } = $props();

  function handleSelect(note) {
    onSelect?.(note);
  }
</script>

{#if open && matches.length > 0}
  <div
    class="wikilink-picker-position"
    style="left: {position.left}px; top: {position.top}px;"
  >
    <WikilinkPicker
      notes={matches}
      {selectedIndex}
      {showPaths}
      onSelect={handleSelect}
      {onHover}
    />
  </div>
{/if}

<style>
  .wikilink-picker-position {
    position: fixed;
    z-index: 200;
    width: min(320px, calc(100vw - 24px));
  }
</style>
