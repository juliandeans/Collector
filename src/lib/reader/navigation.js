import { invoke } from "@tauri-apps/api/core";

export const OBSIDIAN_ONLY_EXTENSIONS = [".excalidraw", ".canvas"];

function normalizeTarget(target = "") {
  const withoutDisplay = target.split("|")[0].trim();
  return withoutDisplay.split("#")[0].trim();
}

function isObsidianOnlyPath(path = "") {
  const normalizedPath = path.toLowerCase();
  return OBSIDIAN_ONLY_EXTENSIONS.some((extension) =>
    normalizedPath.endsWith(extension),
  );
}

export async function ensureVaultNotes(vaultNotes = []) {
  if (vaultNotes.length > 0) return vaultNotes;
  return await invoke("list_vault_notes");
}

export function resolveWikilink(target, vaultNotes = []) {
  const cleanTarget = normalizeTarget(target);
  if (!cleanTarget) return null;

  const normalizedTarget = cleanTarget.toLowerCase().replace(/\\/g, "/");
  const withExtension = normalizedTarget.endsWith(".md")
    ? normalizedTarget
    : `${normalizedTarget}.md`;

  let found = vaultNotes.find((note) => {
    const relativePath = note.relative_path.toLowerCase().replace(/\\/g, "/");
    return (
      note.name.toLowerCase() === normalizedTarget ||
      relativePath === normalizedTarget ||
      relativePath === withExtension
    );
  });

  if (!found) {
    found = vaultNotes.find((note) =>
      note.name.toLowerCase().includes(normalizedTarget),
    );
  }

  return found ?? null;
}

export function navigateToWikilink(
  target,
  tabs = [],
  activeTabIndex = 0,
  { vaultNotes = [], forceNewTab = false } = {},
) {
  const note = resolveWikilink(target, vaultNotes);
  if (!note) {
    return { action: "notFound", target };
  }

  const currentTab = tabs[activeTabIndex];
  if (!currentTab) {
    return { action: "noop" };
  }

  if (isObsidianOnlyPath(note.path) || isObsidianOnlyPath(note.relative_path)) {
    return { action: "openInObsidian", note };
  }

  const openNewTab = forceNewTab || currentTab.isPinned;

  if (openNewTab) {
    const existingIndex = tabs.findIndex((tab) => tab.path === note.path);
    if (existingIndex >= 0) {
      return { action: "activateExisting", index: existingIndex, note };
    }

    return { action: "newTab", note };
  }

  return {
    action: "replaceCurrent",
    note,
    history: [...(currentTab.history ?? []), currentTab.path],
  };
}

export async function openInObsidian(
  path,
  vaultName = "Vault",
  vaultPath = "",
) {
  if (!path) return;

  const normalizedVaultPath = vaultPath.replace(/[\\/]$/, "").replace(/\\/g, "/");
  let relativePath = path.replace(/\\/g, "/");

  if (normalizedVaultPath && relativePath.startsWith(normalizedVaultPath)) {
    relativePath = relativePath
      .slice(normalizedVaultPath.length)
      .replace(/^\/+/, "");
  }

  const noteRef = relativePath.replace(/\.md$/i, "");
  const encodedVault = encodeURIComponent(vaultName);
  const encodedNote = encodeURIComponent(noteRef).replace(/%2F/g, "/");
  const obsidianUrl = `obsidian://open?vault=${encodedVault}&file=${encodedNote}`;

  await invoke("open_external_url", { url: obsidianUrl });
}
