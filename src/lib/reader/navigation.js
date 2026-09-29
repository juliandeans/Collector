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

function normalizeWikilinkLookup(target = "") {
  const cleanTarget = normalizeTarget(target);
  if (!cleanTarget) return null;
  return cleanTarget.toLowerCase().replace(/\\/g, "/");
}

export function findExactWikilinkNote(target, vaultNotes = []) {
  const normalizedTarget = normalizeWikilinkLookup(target);
  if (!normalizedTarget) return null;

  const withExtension = normalizedTarget.endsWith(".md")
    ? normalizedTarget
    : `${normalizedTarget}.md`;

  const found = vaultNotes.find((note) => {
    const relativePath = note.relative_path.toLowerCase().replace(/\\/g, "/");
    return (
      note.name.toLowerCase() === normalizedTarget ||
      relativePath === normalizedTarget ||
      relativePath === withExtension
    );
  });

  return found ?? null;
}

export function resolveWikilink(target, vaultNotes = []) {
  const exact = findExactWikilinkNote(target, vaultNotes);
  if (exact) return exact;

  const normalizedTarget = normalizeWikilinkLookup(target);
  if (!normalizedTarget) return null;

  const found = vaultNotes.find((note) =>
    note.name.toLowerCase().includes(normalizedTarget),
  );

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

  const notePath = note.relative_path ?? note.path;

  if (isObsidianOnlyPath(note.path) || isObsidianOnlyPath(note.relative_path)) {
    return { action: "openInObsidian", note };
  }

  const openNewTab = forceNewTab || currentTab.isPinned;

  if (openNewTab) {
    const existingIndex = tabs.findIndex((tab) => tab.path === notePath);
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
