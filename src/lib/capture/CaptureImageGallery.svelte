<script>
  import { createBubbler } from 'svelte/legacy';

  const bubble = createBubbler();
  import { createEventDispatcher } from "svelte";


  /**
   * @typedef {Object} Props
   * @property {Array} [images]
   */

  /** @type {Props} */
  let { images = [] } = $props();

  const dispatch = createEventDispatcher();

  function handleRemove(id) {
    dispatch("remove", id);
  }
</script>

{#if images.some((img) => img.preview)}
  <div
    class="image-gallery"
    role="presentation"
    ondrop={bubble('drop')}
    ondragover={bubble('dragover')}
    ondragenter={bubble('dragenter')}
  >
    {#each images.filter((img) => img.preview) as image (image.id)}
      <div class="image-preview">
        <img src={image.preview} alt={image.filename} />
        <button
          class="remove-btn"
          onclick={() => handleRemove(image.id)}
          title="Remove"
        >×</button>
        <div class="image-name">{image.filename}</div>
      </div>
    {/each}
  </div>
{/if}

<style>
  .image-gallery {
    display: flex;
    gap: 8px;
    padding: 12px 12px 8px;
    overflow-x: auto;
    overflow-y: hidden;
    flex-shrink: 0;
  }

  .image-gallery::-webkit-scrollbar {
    height: 4px;
  }

  .image-gallery::-webkit-scrollbar-track {
    background: transparent;
  }

  .image-gallery::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.15);
    border-radius: 2px;
  }

  .image-preview {
    position: relative;
    flex-shrink: 0;
    width: 80px;
    height: 80px;
    border-radius: 8px;
    overflow: hidden;
    background: rgba(0, 0, 0, 0.03);
    border: 1px solid rgba(0, 0, 0, 0.08);
    transition: all 0.2s;
  }

  .image-preview:hover {
    transform: scale(1.05);
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  }

  .image-preview img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .remove-btn {
    position: absolute;
    top: 4px;
    right: 4px;
    width: 20px;
    height: 20px;
    border-radius: 50%;
    border: none;
    background: rgba(255, 59, 48, 0.9);
    color: white;
    font-size: 16px;
    line-height: 1;
    cursor: pointer;
    display: flex;
    align-items: center;
    justify-content: center;
    opacity: 0;
    transition: opacity 0.2s;
    padding: 0;
  }

  .image-preview:hover .remove-btn {
    opacity: 1;
  }

  .remove-btn:hover {
    background: rgba(255, 59, 48, 1);
    transform: scale(1.1);
  }

  .image-name {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    padding: 4px;
    background: rgba(0, 0, 0, 0.7);
    color: white;
    font-size: 9px;
    text-align: center;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    opacity: 0;
    transition: opacity 0.2s;
  }

  .image-preview:hover .image-name {
    opacity: 1;
  }
</style>
