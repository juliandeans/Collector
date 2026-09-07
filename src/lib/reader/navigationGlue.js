import { ensureVaultNotes, navigateToWikilink, openInObsidian } from "./navigation.js";
import { createTab, fileLabel } from "./tabState.js";
import { getOpenVaultNoteIntent } from "./paletteLogic.js";
import { normalizeError } from "./errorHandling.js";
import { applyTabUpdate } from "./tabLifecycle.js";

export function getVaultNoteTabPath(note = {}) {
  return note.relative_path ?? note.path ?? "";
}

export function createOpenedNoteTab(note, history = []) {
  return createTab({
    kind: "opened",
    path: getVaultNoteTabPath(note),
    label: note.name,
    isPinned: false,
    history,
  });
}

export async function applyOpenVaultNoteIntent(intent, bag) {
  const { getTabs, setTabs, activateTab } = bag;
  if (intent.action === "activateExisting") {
    await activateTab(intent.index, true);
    return true;
  }
  if (intent.action === "newTab") {
    setTabs([...getTabs(), createOpenedNoteTab(intent.note)]);
    await activateTab(getTabs().length - 1, true);
    return true;
  }
  return false;
}

export async function handleNavigateToWikilink(target, forceNewTab = false, bag) {
  const { getVaultNotes, setVaultNotes, getTabs, setTabs, getActiveTabIndex, getAppSettings, showStatus, forceSave, loadTab, saveScrollPosition } = bag;

  let vaultNotes;
  try {
    vaultNotes = await ensureVaultNotes(getVaultNotes());
    setVaultNotes(vaultNotes);
  } catch (error) {
    showStatus(normalizeError(error), "error", 2200);
    return;
  }

  const tabs = getTabs();
  const activeTabIndex = getActiveTabIndex();
  const intent = navigateToWikilink(target, tabs, activeTabIndex, { vaultNotes, forceNewTab });

  if (intent.action === "notFound") {
    showStatus(`Note not found: [[${target}]]`, "error", 2200);
    return;
  }

  if (intent.action === "openInObsidian") {
    showStatus("Opening in Obsidian...", "success", 1200);
    try {
      await openInObsidian(
        getVaultNoteTabPath(intent.note),
        getAppSettings().vault_name ?? "Vault",
        getAppSettings().vault_path ?? "",
      );
    } catch (error) {
      showStatus("Failed to open in Obsidian", "error", 2200);
    }
    return;
  }

  if (intent.action === "noop") return;

  saveScrollPosition();

  if (await applyOpenVaultNoteIntent(intent, bag)) return;

  if (intent.action === "replaceCurrent") {
    await forceSave(false);
    setTabs(applyTabUpdate(getTabs(), activeTabIndex, {
      path: getVaultNoteTabPath(intent.note),
      label: intent.note.name,
      content: "",
      loaded: false,
      missing: false,
      missingMessage: "",
      history: intent.history,
      isPinned: false,
    }));
    await loadTab(activeTabIndex, true);
  }
}

export async function navigateBack(bag) {
  const { getTabs, setTabs, getActiveTabIndex, getVaultNotes, saveScrollPosition, forceSave, loadTab } = bag;
  const tabs = getTabs();
  const activeTabIndex = getActiveTabIndex();
  const currentTab = tabs[activeTabIndex];
  if (!currentTab?.history?.length) return;

  saveScrollPosition();
  await forceSave(false);
  const previousPath = currentTab.history[currentTab.history.length - 1];
  const note = getVaultNotes().find((entry) => getVaultNoteTabPath(entry) === previousPath);

  setTabs(applyTabUpdate(tabs, activeTabIndex, {
    path: previousPath,
    label: note?.name ?? fileLabel(previousPath),
    content: "",
    loaded: false,
    missing: false,
    missingMessage: "",
    history: currentTab.history.slice(0, -1),
    isPinned: false,
  }));
  await loadTab(activeTabIndex, true);
}

export function openPalette(bag) {
  const { setShowPalette, setPaletteQuery, setSelectedPaletteIndex, focusPaletteInput, getVaultNotes, setVaultNotes, showStatus } = bag;
  setShowPalette(true);
  setPaletteQuery("");
  setSelectedPaletteIndex(0);
  focusPaletteInput();

  if (getVaultNotes().length === 0) {
    ensureVaultNotes(getVaultNotes())
      .then((notes) => setVaultNotes(notes))
      .catch((error) => showStatus(normalizeError(error), "error", 2200));
  }
}

export function closePalette(bag) {
  const { setShowPalette, setPaletteQuery, setSelectedPaletteIndex, focusEditor } = bag;
  setShowPalette(false);
  setPaletteQuery("");
  setSelectedPaletteIndex(0);
  focusEditor();
}

export async function openVaultNote(note, bag) {
  closePalette(bag);
  await applyOpenVaultNoteIntent(getOpenVaultNoteIntent(note, bag.getTabs()), bag);
}

export async function handleOpenInObsidian(bag) {
  const { getActiveTab, getAppSettings, showStatus } = bag;
  const tab = getActiveTab();
  if (!tab?.path) return;
  try {
    await openInObsidian(tab.path, getAppSettings().vault_name ?? "Vault", getAppSettings().vault_path ?? "");
  } catch (error) {
    showStatus("Failed to open in Obsidian", "error", 2200);
  }
}
