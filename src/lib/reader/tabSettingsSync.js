import { getDailyNotePath } from "./tabIO.js";
import {
  rebuildTabsFromSettings,
  getPinnedNotesSignature,
} from "./tabState.js";
import {
  applyColorSettings,
  applySettings,
  getReaderFilterSettings,
  haveReaderFilterChanges,
} from "./settingsBridge.js";
import { clearImageCache } from "./contentProcessing.js";

export async function applyTabSettings(settings, options = {}, bag) {
  const { preserveOpened = true, forceReloadActive = false } = options;
  const {
    getTabs,
    setTabs,
    getActiveTab,
    setActiveTabIndex,
    loadTab,
    loadEditorContent,
    setMissingFileMessage,
  } = bag;

  const dailyPath = await getDailyNotePath(settings);
  const nextState = rebuildTabsFromSettings(settings, getTabs(), {
    preserveOpened,
    previousActivePath: getActiveTab()?.path ?? null,
    dailyPath,
  });

  setTabs(nextState.tabs);
  setActiveTabIndex(nextState.activeTabIndex);

  const currentTab = nextState.tabs[nextState.activeTabIndex];
  if (!currentTab) return;

  if (forceReloadActive || !currentTab.loaded) {
    await loadTab(nextState.activeTabIndex, true);
    return;
  }

  await loadEditorContent(currentTab.content);
  setMissingFileMessage(
    currentTab.missing ? currentTab.missingMessage || "" : "",
  );
}

export async function handleSettingsChanged(settings, bag) {
  const {
    getAppSettings,
    setAppSettings,
    getRawContent,
    getEditorComponent,
    flushPendingSave,
    renderContentToEditor,
  } = bag;

  const previousSettings = getAppSettings();
  const previousFilters = getReaderFilterSettings(previousSettings);
  const previousPinnedNotes = getPinnedNotesSignature(
    previousSettings.pinned_notes,
  );
  const nextSettings = applySettings(previousSettings, settings);
  setAppSettings(nextSettings);
  applyColorSettings(nextSettings);
  const nextFilters = getReaderFilterSettings(nextSettings);
  const nextPinnedNotes = getPinnedNotesSignature(nextSettings.pinned_notes);
  const filtersChanged = haveReaderFilterChanges(previousFilters, nextFilters);
  // undefined counts as true (legacy configs): normalize before comparing
  const dailyNoteVisibilityChanged =
    (previousSettings.reader_show_daily_note ?? true) !==
    (nextSettings.reader_show_daily_note ?? true);

  if (previousPinnedNotes !== nextPinnedNotes || dailyNoteVisibilityChanged) {
    await flushPendingSave(false);
    await applyTabSettings(
      settings,
      { preserveOpened: true, forceReloadActive: false },
      bag,
    );
    return;
  }

  if (filtersChanged) {
    getEditorComponent()?.finalizeBlock?.();
    clearImageCache();
    await renderContentToEditor(getRawContent());
  }
}

export async function buildInitialTabs(bag) {
  const { invoke, getAppSettings, setAppSettings } = bag;
  const settings = await invoke("load_settings");
  const merged = applySettings(getAppSettings(), settings);
  setAppSettings(merged);
  applyColorSettings(merged);
  await applyTabSettings(
    settings,
    { preserveOpened: false, forceReloadActive: true },
    bag,
  );
}
