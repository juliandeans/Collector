<script>
  import { createEventDispatcher } from "svelte";
  import WikilinkPicker from "../WikilinkPicker.svelte";







  const dispatch = createEventDispatcher();


  /**
   * @typedef {Object} Props
   * @property {boolean} [open]
   * @property {Array} [matches]
   * @property {number} [selectedIndex]
   * @property {{ left: number, top: number }} [position]
   * @property {boolean} [showPaths]
   * @property {(index: number) => void} [onHover]
   */

  /** @type {Props} */
  let {
    open = false,
    matches = [],
    selectedIndex = 0,
    position = { left: 0, top: 0 },
    showPaths = true,
    onHover = () => {}
  } = $props();

  function handleSelect(note) {
    dispatch("select", note);
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
