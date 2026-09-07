import { tick } from "svelte";
import { normalizeNewlines, preprocessContent } from "./contentProcessing.js";
import { composeContentFromMarkdown } from "./editorSerialization.js";
import { getAutocompleteResults } from "./autocomplete.js";

export function getCurrentMarkdown(bag) {
  const { getEditorComponent, getRawContent, getAppSettings, codeblockMap, hiddenBlockMap, setStrippedFrontmatter } = bag;
  const editorComponent = getEditorComponent();
  if (!editorComponent) {
    return preprocessContent(getRawContent(), {
      appSettings: getAppSettings(),
      codeblockMap,
      hiddenBlockMap,
      setStrippedFrontmatter,
    });
  }
  return editorComponent.getMarkdown();
}

export async function renderContentToEditor(raw = "", bag) {
  const { getEditorComponent, getAppSettings, codeblockMap, hiddenBlockMap, setStrippedFrontmatter } = bag;
  preprocessContent(raw, {
    appSettings: getAppSettings(),
    codeblockMap,
    hiddenBlockMap,
    setStrippedFrontmatter,
  });

  const editorComponent = getEditorComponent();
  if (!editorComponent) return;
  await tick();
  await editorComponent.renderContent(raw);
}

// TODO: keep this in Reader until editor load/reset state can be passed via a dedicated pipeline object.
export async function loadEditorContent(raw = "", bag) {
  const { getEditorComponent, setRawContent, closeAutocomplete, closeSearch } = bag;
  getEditorComponent()?.finalizeBlock?.();
  closeAutocomplete();
  closeSearch();
  const normalized = normalizeNewlines(raw);
  setRawContent(normalized);
  await renderContentToEditor(normalized, bag);
}

export function getCurrentSaveContent(bag) {
  const { getStrippedFrontmatter, hiddenBlockMap, codeblockMap } = bag;
  const markdown = getCurrentMarkdown(bag);
  return composeContentFromMarkdown(markdown, {
    strippedFrontmatter: getStrippedFrontmatter(),
    hiddenBlockMap,
    codeblockMap,
  });
}

export function handleEditorChange(bag) {
  const { setRawContent, updateActiveTabContent, scheduleSave } = bag;
  const content = getCurrentSaveContent(bag);
  setRawContent(content);
  updateActiveTabContent(content);
  scheduleSave(content);
}

export async function handleEditorSaveRequest(bag) {
  await bag.forceSave();
}

export function handleEditorAutocompleteChange(event, bag) {
  const {
    getVaultNotes, getAutocompleteResultLimit, setShowAutocomplete,
    setAutocompleteResults, setAutocompleteRange, setAutocompleteIndex,
  } = bag;
  setShowAutocomplete(event.detail.open);
  setAutocompleteResults(
    event.detail.results ??
      getAutocompleteResults(event.detail.query, getVaultNotes(), getAutocompleteResultLimit()),
  );
  setAutocompleteRange(event.detail.range);
  setAutocompleteIndex(event.detail.index ?? 0);
}
