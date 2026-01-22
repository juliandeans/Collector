<script>
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import { getSystemFonts } from "./lib/utils.js";
  import { defaultSettings } from "./lib/stores.js";

  let settings = { ...defaultSettings };
  let originalSettings = { ...defaultSettings };
  let isSaving = false;
  let statusMessage = "";
  let statusType = "";
  let showTemplateEditor = false;

  const systemFonts = getSystemFonts();

  async function loadSettings() {
    try {
      const loaded = await invoke("load_settings");
      settings = { ...loaded };
      originalSettings = { ...loaded };
    } catch (e) {
      console.error("Failed to load settings:", e);
      showStatus("Failed to load settings", "error");
    }
  }

  onMount(async () => {
    await loadSettings();
  });

  function showStatus(message, type = "success") {
    statusMessage = message;
    statusType = type;
    setTimeout(() => {
      statusMessage = "";
      statusType = "";
    }, 3000);
  }

  async function pickVaultPath() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: settings.vault_path || undefined,
    });
    if (selected) {
      settings.vault_path = selected;
    }
  }

  async function pickScreenshotPath() {
    const selected = await open({
      directory: true,
      multiple: false,
      defaultPath: settings.screenshot_path || undefined,
    });
    if (selected) {
      settings.screenshot_path = selected;
    }
  }

  async function handleSave() {
    isSaving = true;

    try {
      await invoke("save_settings", { newSettings: settings });

      originalSettings = { ...settings };

      showStatus("Settings saved!", "success");
    } catch (e) {
      console.error("Failed to save settings:", e);
      console.error("Error details:", JSON.stringify(e));
      showStatus("Error: " + e.toString(), "error");
    } finally {
      isSaving = false;
      settings = { ...settings };
    }
  }

  async function handleCancel() {
    settings = { ...originalSettings };

    statusMessage = "";
    statusType = "";

    try {
      await invoke("close_settings");
    } catch (e) {
      console.error("Failed to close settings window:", e);
    }
  }

  function handleReset() {
    if (confirm("Reset all settings to default?")) {
      settings = { ...defaultSettings };
    }
  }

  function handleShortcutKeyDown(event, field) {
    event.preventDefault();

    if (["Control", "Shift", "Alt", "Meta", "Command"].includes(event.key)) {
      return;
    }

    const modifiers = [];
    if (event.metaKey) modifiers.push("Cmd");
    if (event.ctrlKey) modifiers.push("Ctrl");
    if (event.shiftKey) modifiers.push("Shift");
    if (event.altKey) modifiers.push("Alt");

    let key = event.code;

    const keyMap = {
      Space: "Space",
      Escape: "Escape",
      Enter: "Enter",
      Backspace: "Backspace",
      Delete: "Delete",
      ArrowUp: "Up",
      ArrowDown: "Down",
      ArrowLeft: "Left",
      ArrowRight: "Right",
      Tab: "Tab",
      Home: "Home",
      End: "End",
      PageUp: "PageUp",
      PageDown: "PageDown",
    };

    if (keyMap[key]) {
      key = keyMap[key];
    } else if (key.startsWith("Key")) {
      key = key.substring(3).toUpperCase();
    } else if (key.startsWith("Digit")) {
      key = key.substring(5);
    } else if (key.startsWith("F") && key.length <= 3) {
      key = key;
    } else {
      return;
    }

    if (modifiers.length === 0) {
      return;
    }

    const shortcut = [...modifiers, key].join("+");
    settings[field] = shortcut;
  }

  $: hasChanges = JSON.stringify(settings) !== JSON.stringify(originalSettings);
</script>

<div class="settings-container">
  <header>
    <h1>Settings</h1>
    {#if statusMessage}
      <div class="status {statusType}">{statusMessage}</div>
    {/if}
  </header>

  <main>
    <section>
      <h2>Obsidian Integration</h2>
      <div class="field">
        <label for="vault_name">Vault Name</label>
        <input
          type="text"
          id="vault_name"
          bind:value={settings.vault_name}
          placeholder="Vault"
        />
        <small>Name of your Obsidian vault</small>
      </div>
      <div class="field">
        <label for="vault_path">Vault Path</label>
        <div class="path-picker">
          <input
            type="text"
            id="vault_path"
            bind:value={settings.vault_path}
            placeholder="/Users/username/Vault"
            readonly
          />
          <button class="secondary" on:click={pickVaultPath}>Choose...</button>
        </div>
        <small>Full path to your Obsidian vault</small>
      </div>
      <div class="field">
        <label for="daily_note_folder">Daily Note Path</label>
        <input
          type="text"
          id="daily_note_folder"
          bind:value={settings.daily_note_folder}
          placeholder="Journal/Notes/"
        />
        <small>Relative path in vault for daily notes</small>
      </div>
      <div class="field">
        <label for="daily_note_format">Daily Note Format</label>
        <input
          type="text"
          id="daily_note_format"
          bind:value={settings.daily_note_format}
          placeholder="YYYY-MM-DD"
        />
        <small>Filename format (e.g. YYYY-MM-DD). Supports: YYYY, MM, DD</small>
      </div>
      <div class="field">
        <label for="entry_header">Entry Header</label>
        <input
          type="text"
          id="entry_header"
          bind:value={settings.entry_header}
          placeholder="#### HH:mm"
        />
        <small>Markdown header for each entry (HH:mm for time)</small>
      </div>
    </section>

    <section>
      <h2>Images</h2>
      <div class="field">
        <label for="screenshot_path">Image Folder</label>
        <div class="path-picker">
          <input
            type="text"
            id="screenshot_path"
            bind:value={settings.screenshot_path}
            placeholder="/Users/username/Vault/Images/Screenshots"
            readonly
          />
          <button class="secondary" on:click={pickScreenshotPath}
            >Choose...</button
          >
        </div>
        <small>Folder for images (will be created automatically)</small>
      </div>
      <div class="field">
        <label for="image_filename">Filename Template</label>
        <input
          type="text"
          id="image_filename"
          bind:value={settings.image_filename}
          placeholder="screenshot-YYYY-MM-DD-HHmmss"
        />
        <small>Supports: YYYY, MM, DD, HH, mm, ss</small>
      </div>
      <div class="field">
        <label for="compression_max_kb">Max. Image Size (KB)</label>
        <input
          type="number"
          id="compression_max_kb"
          bind:value={settings.compression_max_kb}
          min="50"
          max="1000"
          step="50"
        />
        <small>Images will be compressed to this size</small>
      </div>
      <div class="field">
        <label for="default_image_width">Default Image Width</label>
        <input
          type="text"
          id="default_image_width"
          bind:value={settings.default_image_width}
          placeholder="600"
          inputmode="numeric"
        />
        <small
          >Optional width in pixels for new image links (leave empty for no
          width)</small
        >
      </div>
    </section>

    <section>
      <h2>New Note</h2>
      <div class="field">
        <label for="notes_folder">Notes Folder</label>
        <input
          type="text"
          id="notes_folder"
          bind:value={settings.notes_folder}
          placeholder="Notes/"
        />
        <small>Relative path in vault for new notes</small>
      </div>
      <div class="field">
        <label for="note_template">Template Text</label>
        <button
          class="secondary"
          on:click={() => (showTemplateEditor = !showTemplateEditor)}
        >
          {showTemplateEditor ? "Hide Template" : "Edit Template"}
        </button>
        {#if showTemplateEditor}
          <textarea
            id="note_template"
            bind:value={settings.note_template}
            placeholder="---&#10;created: <% tp.date.now(&quot;YYYY-MM-DD hh:mm&quot;) %>&#10;modified: &#10;daily: &quot;[[<% tp.date.now(&quot;YYYY-MM-DD&quot;) %>]]&quot;&#10;tags: inbox&#10;type: inbox&#10;---"
            rows="8"
            style="margin-top: 8px;"
          />
          <small
            >This text will be inserted at the beginning of each new note (e.g.
            for frontmatter/properties)</small
          >
        {/if}
      </div>
    </section>

    <section>
      <h2>Window</h2>
      <div class="field">
        <fieldset class="radio-group">
          <legend>Screen Edge</legend>
          <label class="radio"
            ><input type="radio" bind:group={settings.edge_side} value="left" />
            Left</label
          >
          <label class="radio"
            ><input
              type="radio"
              bind:group={settings.edge_side}
              value="right"
            /> Right</label
          >
        </fieldset>
      </div>
      <div class="field-row">
        <div class="field">
          <label for="window_width">Width (px)</label>
          <input
            type="number"
            id="window_width"
            bind:value={settings.window_width}
            min="200"
            max="800"
          />
        </div>
        <div class="field">
          <label for="window_height">Height (px)</label>
          <input
            type="number"
            id="window_height"
            bind:value={settings.window_height}
            min="80"
            max="400"
          />
        </div>
      </div>
      <div class="field">
        <label for="border_radius"
          >Corner Radius: {settings.border_radius}px</label
        >
        <input
          type="range"
          id="border_radius"
          bind:value={settings.border_radius}
          min="0"
          max="12"
        />
      </div>
      <div class="field">
        <label for="background_color">Background Color</label>
        <div class="color-input">
          <input
            type="color"
            id="background_color"
            bind:value={settings.background_color}
          />
          <input
            type="text"
            bind:value={settings.background_color}
            pattern="^#[0-9A-Fa-f]{6}$"
          />
        </div>
      </div>
      <div class="field">
        <label for="window_transparency"
          >Transparency: {settings.window_transparency ?? 55}%</label
        >
        <input
          type="range"
          id="window_transparency"
          bind:value={settings.window_transparency}
          min="0"
          max="100"
        />
        <small>Transparency of window background</small>
      </div>
      <div class="field">
        <label for="window_blur">Blur: {settings.window_blur ?? 80}px</label>
        <input
          type="range"
          id="window_blur"
          bind:value={settings.window_blur}
          min="0"
          max="200"
        />
        <small>Background blur effect</small>
      </div>
      <div class="field">
        <label for="window_saturation"
          >Saturation: {settings.window_saturation ?? 200}%</label
        >
        <input
          type="range"
          id="window_saturation"
          bind:value={settings.window_saturation}
          min="0"
          max="300"
        />
        <small>Background color intensity</small>
      </div>
      <div class="field">
        <label for="window_brightness"
          >Brightness: {(settings.window_brightness ?? 0 > 0)
            ? ""
            : ""}{settings.window_brightness ?? 0}</label
        >
        <input
          type="range"
          id="window_brightness"
          bind:value={settings.window_brightness}
          min="-100"
          max="100"
        />
        <small>Brightens dark areas or darkens light areas</small>
      </div>
    </section>

    <section>
      <h2>Font</h2>
      <div class="field">
        <label for="font_family">Font Family</label>
        <select id="font_family" bind:value={settings.font_family}>
          {#each systemFonts as font}
            <option value={font}>{font}</option>
          {/each}
        </select>
      </div>
      <div class="field">
        <label for="font_size">Font Size: {settings.font_size}px</label>
        <input
          type="range"
          id="font_size"
          bind:value={settings.font_size}
          min="10"
          max="20"
        />
      </div>
      <div class="field">
        <label for="text_color">Text Color</label>
        <div class="color-input">
          <input
            type="color"
            id="text_color"
            bind:value={settings.text_color}
          />
          <input
            type="text"
            bind:value={settings.text_color}
            pattern="^#[0-9A-Fa-f]{6}$"
          />
        </div>
        <small>Default: White (#ffffff)</small>
      </div>
    </section>

    <section>
      <h2>Shortcuts</h2>
      <div class="field">
        <label for="global_shortcut">Open Window</label>
        <input
          type="text"
          id="global_shortcut"
          bind:value={settings.global_shortcut}
          placeholder="Cmd+Shift+N"
          on:keydown={(e) => handleShortcutKeyDown(e, "global_shortcut")}
        />
        <small>Click in the field and press the desired key combination</small>
      </div>
      <div class="field">
        <label for="capture_text_shortcut">Copy Text to Collector</label>
        <input
          type="text"
          id="capture_text_shortcut"
          bind:value={settings.capture_text_shortcut}
          placeholder="Cmd+Shift+C"
          on:keydown={(e) => handleShortcutKeyDown(e, "capture_text_shortcut")}
        />
        <small>Click in the field and press the desired key combination</small>
      </div>
      <div class="field">
        <label for="save_to_daily_shortcut">Save to Daily Note</label>
        <input
          type="text"
          id="save_to_daily_shortcut"
          bind:value={settings.save_to_daily_shortcut}
          placeholder="Cmd+Enter"
          on:keydown={(e) => handleShortcutKeyDown(e, "save_to_daily_shortcut")}
        />
        <small>Click in the field and press the desired key combination</small>
      </div>
      <div class="field">
        <label for="save_as_note_shortcut">Create New Note</label>
        <input
          type="text"
          id="save_as_note_shortcut"
          bind:value={settings.save_as_note_shortcut}
          placeholder="Cmd+Shift+Enter"
          on:keydown={(e) => handleShortcutKeyDown(e, "save_as_note_shortcut")}
        />
        <small>Click in the field and press the desired key combination</small>
      </div>
      <div class="field">
        <label class="checkbox">
          <input
            type="checkbox"
            bind:checked={settings.edge_detection_enabled}
          />
          Edge Detection Enabled
        </label>
        <small>Window opens when moving mouse to screen edge</small>
      </div>
    </section>
  </main>

  <footer>
    <button class="secondary" on:click={handleReset}>Reset</button>
    <div class="spacer"></div>
    <button class="secondary" on:click={handleCancel}>Cancel</button>
    <button
      class="primary"
      on:click={handleSave}
      disabled={isSaving || !hasChanges}
    >
      {isSaving ? "Saving..." : "Save"}
    </button>
  </footer>
</div>

<style>
  .settings-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
    font-size: 13px;
    color: #1a1a1a;
    background: #f5f5f7;
  }

  header {
    padding: 20px 24px;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    border-bottom: 1px solid rgba(0, 0, 0, 0.08);
    display: flex;
    align-items: center;
    gap: 16px;
  }

  header h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
    letter-spacing: -0.3px;
  }

  .status {
    padding: 8px 14px;
    border-radius: 8px;
    font-size: 12px;
    font-weight: 500;
    letter-spacing: 0.2px;
    animation: slideIn 0.3s ease;
  }

  @keyframes slideIn {
    from {
      opacity: 0;
      transform: translateX(-8px);
    }
  }

  .status.success {
    background: rgba(52, 211, 153, 0.15);
    color: #059669;
    border: 1px solid rgba(52, 211, 153, 0.2);
  }

  .status.error {
    background: rgba(239, 68, 68, 0.15);
    color: #dc2626;
    border: 1px solid rgba(239, 68, 68, 0.2);
  }

  main {
    flex: 1;
    overflow-y: auto;
    padding: 20px 24px;
  }

  section {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    border-radius: 12px;
    padding: 20px;
    margin-bottom: 16px;
    border: 1px solid rgba(0, 0, 0, 0.08);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.05);
    transition: all 0.2s ease;
  }

  section:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
    border-color: rgba(0, 0, 0, 0.1);
  }

  section h2 {
    font-size: 15px;
    font-weight: 600;
    margin: 0 0 16px 0;
    color: #374151;
    letter-spacing: -0.2px;
  }

  .field {
    margin-bottom: 12px;
  }

  .field:last-child {
    margin-bottom: 0;
  }

  .field label {
    display: block;
    font-weight: 500;
    margin-bottom: 4px;
  }

  .field small {
    display: block;
    color: #888;
    font-size: 11px;
    margin-top: 4px;
  }

  .field-row {
    display: flex;
    gap: 12px;
  }

  .field-row .field {
    flex: 1;
  }

  input[type="text"],
  input[type="number"],
  select,
  textarea {
    width: 100%;
    padding: 9px 12px;
    border: 1.5px solid rgba(0, 0, 0, 0.1);
    border-radius: 8px;
    font-size: 13px;
    background: white;
    transition: all 0.2s ease;
    font-family: -apple-system, BlinkMacSystemFont, "SF Pro", sans-serif;
  }

  textarea {
    resize: vertical;
    min-height: 100px;
    font-family: "SF Mono", Menlo, Monaco, monospace;
    font-size: 12px;
  }

  input[type="text"]:focus,
  input[type="number"]:focus,
  select:focus,
  textarea:focus {
    outline: none;
    border-color: #8b5cf6;
    box-shadow: 0 0 0 3px rgba(139, 92, 246, 0.1);
  }

  input[type="range"] {
    width: 100%;
    margin: 4px 0;
  }

  .color-input {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .color-input input[type="color"] {
    width: 40px;
    height: 32px;
    padding: 2px;
    border: 1px solid #ddd;
    border-radius: 4px;
    cursor: pointer;
  }

  .color-input input[type="text"] {
    flex: 1;
    font-family: monospace;
  }

  .radio-group {
    display: flex;
    gap: 16px;
    border: none;
    padding: 0;
    margin: 0;
  }

  .radio-group legend {
    padding: 0;
    margin: 0 0 6px 0;
    font-weight: 500;
  }

  .radio,
  .checkbox {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
    font-weight: normal;
  }

  footer {
    padding: 16px 24px;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(40px);
    -webkit-backdrop-filter: blur(40px);
    border-top: 1px solid rgba(0, 0, 0, 0.08);
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .spacer {
    flex: 1;
  }

  button {
    padding: 8px 16px;
    border-radius: 6px;
    font-size: 13px;
    font-weight: 500;
    cursor: pointer;
    border: none;
    transition: all 0.2s ease;
  }

  button.primary {
    background: linear-gradient(135deg, #8b5cf6 0%, #7c3aed 100%);
    color: white;
    box-shadow: 0 2px 8px rgba(139, 92, 246, 0.25);
  }

  button.primary:hover:not(:disabled) {
    background: linear-gradient(135deg, #7c3aed 0%, #6d28d9 100%);
    box-shadow: 0 4px 12px rgba(139, 92, 246, 0.3);
    transform: translateY(-1px);
  }

  button.primary:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  button.secondary {
    background: #e5e5e5;
    color: #333;
  }

  button.secondary:hover {
    background: #d5d5d5;
  }

  .path-picker {
    display: flex;
    gap: 8px;
  }

  .path-picker input {
    flex: 1;
    font-family: "SF Mono", Menlo, Monaco, monospace;
    font-size: 12px;
    background: #f9f9f9;
  }

  .path-picker button {
    padding: 8px 16px;
    white-space: nowrap;
  }
</style>
