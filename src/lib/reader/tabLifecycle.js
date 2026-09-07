import { loadTabContent, saveTabContent } from "./tabIO.js";
import { normalizeNewlines, warmImagesInText } from "./contentProcessing.js";
import { getMissingState, normalizeError } from "./errorHandling.js";

export function applyTabUpdate(tabs, index, updates) {
  return tabs.map((tab, tabIndex) =>
    tabIndex === index ? { ...tab, ...updates } : tab,
  );
}

export function applyLoadedTab(tabs, index, content) {
  return applyTabUpdate(tabs, index, {
    content,
    loaded: true,
    missing: false,
    missingMessage: "",
  });
}

export function applyMissingTab(tabs, index, missingState) {
  return applyTabUpdate(tabs, index, {
    content: "",
    loaded: true,
    missing: missingState.missing,
    missingMessage: missingState.missingMessage,
  });
}

export function saveScrollPosition({ getActiveTab, getEditorComponent, scrollPositions }) {
  const currentTab = getActiveTab();
  const editorComponent = getEditorComponent();
  if (!currentTab || !editorComponent) return;
  scrollPositions.set(currentTab.path, editorComponent.getScrollTop());
}

export async function restoreScrollPosition(path, { getEditorComponent, scrollPositions }) {
  const editorComponent = getEditorComponent();
  if (!editorComponent) return;
  await editorComponent.restoreScroll(path, scrollPositions);
}

async function syncActiveTabView(index, path, content, missingMessage = "", bag) {
  const { getActiveTabIndex, loadEditorContent, setMissingFileMessage, getEditorComponent, scrollPositions } = bag;
  if (index !== getActiveTabIndex()) return;
  await loadEditorContent(content);
  setMissingFileMessage(missingMessage);
  await restoreScrollPosition(path, { getEditorComponent, scrollPositions });
}

export async function loadTab(index, forceReload, bag) {
  const { getTabs, setTabs, getActiveTabIndex, loadEditorContent, setMissingFileMessage, getEditorComponent, scrollPositions, showStatus } = bag;
  const tab = getTabs()[index];
  if (!tab) return;

  if (!forceReload && tab.loaded) {
    await syncActiveTabView(index, tab.path, tab.content, tab.missing ? tab.missingMessage || "" : "", bag);
    return;
  }

  try {
    const content = await loadTabContent(tab.path);
    setTabs(applyLoadedTab(getTabs(), index, content));
    await syncActiveTabView(index, tab.path, content, "", bag);
  } catch (error) {
    const missingState = getMissingState(error);
    setTabs(applyMissingTab(getTabs(), index, missingState));
    await syncActiveTabView(index, tab.path, "", missingState.missingMessage, bag);
    if (!missingState.missing) showStatus(missingState.message, "error", 2200);
  }
}

export async function syncTabWithDisk(index, bag) {
  const { getTabs, setTabs, getActiveTabIndex, renderContentToEditor, setMissingFileMessage, showStatus } = bag;
  const tab = getTabs()[index];
  if (!tab) return;

  if (!tab.loaded) {
    await loadTab(index, true, bag);
    return;
  }

  try {
    const content = normalizeNewlines(await loadTabContent(tab.path));
    const currentContent = normalizeNewlines(tab.content ?? "");

    if (!tab.missing && content === currentContent) {
      if (index === getActiveTabIndex()) {
        setMissingFileMessage("");
        const hasNewImages = await warmImagesInText(content);
        if (hasNewImages) {
          await renderContentToEditor(content);
        }
      }
      return;
    }

    setTabs(applyLoadedTab(getTabs(), index, content));
    await syncActiveTabView(index, tab.path, content, "", bag);
  } catch (error) {
    const missingState = getMissingState(error);
    const isUnchangedMissingState =
      missingState.missing &&
      tab.missing &&
      normalizeNewlines(tab.content ?? "") === "" &&
      (tab.missingMessage || "") === missingState.missingMessage;

    if (isUnchangedMissingState) {
      if (index === getActiveTabIndex()) setMissingFileMessage(missingState.missingMessage);
      return;
    }

    setTabs(applyMissingTab(getTabs(), index, missingState));
    await syncActiveTabView(index, tab.path, "", missingState.missingMessage, bag);
    if (!missingState.missing) showStatus(missingState.message, "error", 2200);
  }
}

export async function activateTab(index, forceReload, bag) {
  const { getActiveTab, getActiveTabIndex, setActiveTabIndex, getEditorComponent, scrollPositions, closeSearch } = bag;
  saveScrollPosition({ getActiveTab, getEditorComponent, scrollPositions });
  closeSearch();
  if (index !== getActiveTabIndex()) {
    getEditorComponent()?.finalizeBlock?.();
    await flushPendingSave(false, bag);
  }

  setActiveTabIndex(index);
  await loadTab(index, forceReload, bag);
}

export async function saveTabByIndex(index, content, showConfirmation = true, bag) {
  const { getTabs, setTabs, getActiveTabIndex, setMissingFileMessage, setIsSaving, showStatus } = bag;
  const tab = getTabs()[index];
  if (!tab) return;

  setIsSaving(true);

  try {
    await saveTabContent(tab.path, content);

    setTabs(applyTabUpdate(getTabs(), index, {
      content,
      missing: false,
      missingMessage: "",
      loaded: true,
    }));

    if (index === getActiveTabIndex()) {
      setMissingFileMessage("");
    }
  } catch (error) {
    showStatus(normalizeError(error), "error", 2200);
  } finally {
    setIsSaving(false);
  }
}

export function scheduleSave(content, bag) {
  const { saveScheduler, getActiveTabIndex } = bag;
  saveScheduler.schedule(
    { index: getActiveTabIndex(), content },
    (job) => {
      saveTabByIndex(job.index, job.content, true, bag);
    },
  );
}

export async function flushPendingSave(showConfirmation = false, bag) {
  const { saveScheduler } = bag;
  const job = saveScheduler.flush();
  if (!job) return;
  await saveTabByIndex(job.index, job.content, showConfirmation, bag);
}

export async function forceSave(showConfirmation = true, bag) {
  const { getActiveTab, saveScheduler, getCurrentSaveContent, setRawContent, getTabs, setTabs, getActiveTabIndex } = bag;
  if (!getActiveTab()) return;

  saveScheduler.clear();

  const content = getCurrentSaveContent();
  setRawContent(content);
  setTabs(applyTabUpdate(getTabs(), getActiveTabIndex(), { content, loaded: true }));
  await saveTabByIndex(getActiveTabIndex(), content, showConfirmation, bag);
}

export async function closeActiveTab(bag) {
  const { getActiveTab, getActiveTabIndex, setActiveTabIndex, getTabs, setTabs, getEditorComponent, scrollPositions } = bag;
  const activeTab = getActiveTab();
  if (!activeTab || activeTab.kind !== "opened" || activeTab.isPinned) return;

  saveScrollPosition({ getActiveTab, getEditorComponent, scrollPositions });
  await flushPendingSave(false, bag);
  scrollPositions.delete(activeTab.path);

  const tabs = getTabs();
  const activeTabIndex = getActiveTabIndex();
  const nextTabs = tabs.filter((_, index) => index !== activeTabIndex);
  const nextIndex = Math.max(activeTabIndex - 1, 0);
  setTabs(nextTabs);
  setActiveTabIndex(nextIndex);
  await loadTab(nextIndex, false, bag);
}

export async function closeTabByIndex(index, bag) {
  const { getTabs, setTabs, getActiveTabIndex, setActiveTabIndex, closeTabContextMenu, getActiveTab, getEditorComponent, scrollPositions } = bag;
  const tab = getTabs()[index];
  if (!tab || tab.kind !== "opened") return;

  closeTabContextMenu();
  saveScrollPosition({ getActiveTab, getEditorComponent, scrollPositions });
  await flushPendingSave(false, bag);
  scrollPositions.delete(tab.path);

  const tabs = getTabs();
  const activeTabIndex = getActiveTabIndex();
  const nextTabs = tabs.filter((_, tabIndex) => tabIndex !== index);
  const nextIndex =
    activeTabIndex > index
      ? activeTabIndex - 1
      : Math.min(activeTabIndex, nextTabs.length - 1);

  setTabs(nextTabs);
  const resolvedIndex = Math.max(nextIndex, 0);
  setActiveTabIndex(resolvedIndex);

  if (nextTabs[resolvedIndex]) {
    await loadTab(resolvedIndex, false, bag);
  }
}
