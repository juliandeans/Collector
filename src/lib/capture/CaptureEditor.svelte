<script>
  import { createBubbler } from 'svelte/legacy';

  const bubble = createBubbler();




  /**
   * @typedef {Object} Props
   * @property {string} [content]
   * @property {boolean} [isLoading]
   * @property {string} [placeholder]
   * @property {HTMLTextAreaElement|null} [textareaRef]
   */

  /** @type {Props} */
  let {
    content = $bindable(""),
    isLoading = false,
    placeholder = "Enter note or drag image here...",
    textareaRef = $bindable()
  } = $props();
</script>

<div
  class="content-area"
  role="presentation"
  ondrop={bubble('drop')}
  ondragover={bubble('dragover')}
  ondragenter={bubble('dragenter')}
>
  <textarea
    bind:this={textareaRef}
    bind:value={content}
    onkeydown={bubble('keydown')}
    oninput={bubble('input')}
    onblur={bubble('blur')}
    {placeholder}
    disabled={isLoading}
    spellcheck="false"
  ></textarea>
</div>

<style>
  .content-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    padding: 12px 16px 16px;
    overflow: hidden;
  }

  textarea {
    flex: 1;
    width: 100%;
    border: none;
    outline: none;
    resize: none;
    background: transparent;
    font-family: inherit;
    font-size: var(--app-font-size, 15px);
    line-height: 1.6;
    color: var(--app-text-color, #ffffff);
    caret-color: var(--accent-color, #8b5cf6);
  }

  textarea::placeholder {
    color: rgba(255, 255, 255, 0.4);
  }

  textarea:disabled {
    opacity: 0.6;
  }

  textarea::-webkit-scrollbar {
    width: 6px;
  }

  textarea::-webkit-scrollbar-track {
    background: transparent;
  }

  textarea::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.12);
    border-radius: 5px;
  }
</style>
