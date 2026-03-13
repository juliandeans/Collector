export function fileLabel(path = "") {
  const filename = path.split("/").pop() || path;
  return filename.replace(/\.md$/i, "");
}

export function normalizePinnedNotes(pinnedNotes = []) {
  return pinnedNotes
    .map((entry) => {
      if (typeof entry === "string") {
        return {
          path: entry,
          label: fileLabel(entry),
          icon: "",
        };
      }

      return {
        path: entry?.path ?? "",
        label: entry?.label ?? "",
        icon: entry?.icon ?? "",
      };
    })
    .filter((entry) => entry.path.trim() !== "")
    .map((entry) => ({
      path: entry.path,
      label: entry.label.trim(),
      icon: entry.icon.trim(),
    }));
}

export function getPinnedNotesSignature(pinnedNotes = []) {
  return JSON.stringify(
    normalizePinnedNotes(pinnedNotes).map((note) => ({
      path: note.path,
      label: note.label,
      icon: note.icon,
    })),
  );
}

export function createTab({
  kind = "opened",
  path,
  label = "",
  icon = "",
  isPinned = kind === "daily" || kind === "pinned",
  history = [],
  existingTab = null,
} = {}) {
  const fallbackLabel =
    kind === "daily" ? "Daily" : kind === "pinned" ? "" : fileLabel(path);

  return {
    kind,
    path,
    label: label.trim() || fallbackLabel,
    icon: icon.trim(),
    content: existingTab?.content ?? "",
    loaded: existingTab?.loaded ?? false,
    missing: existingTab?.missing ?? false,
    missingMessage: existingTab?.missingMessage ?? "",
    isPinned: existingTab?.isPinned ?? isPinned,
    history: [...(existingTab?.history ?? history)],
  };
}

export function rebuildTabsFromSettings(
  settings,
  currentTabs = [],
  { preserveOpened = true, previousActivePath = null, dailyPath = "" } = {},
) {
  const pinnedNotes = normalizePinnedNotes(settings?.pinned_notes ?? []);
  const existingByPath = new Map(currentTabs.map((tab) => [tab.path, tab]));

  const nextTabs = [
    createTab({
      kind: "daily",
      path: dailyPath,
      label: "Daily",
      isPinned: true,
      existingTab:
        currentTabs.find((tab) => tab.kind === "daily") ??
        existingByPath.get(dailyPath),
    }),
    ...pinnedNotes.map((note) =>
      createTab({
        kind: "pinned",
        path: note.path,
        label: note.label,
        icon: note.icon,
        isPinned: true,
        existingTab: existingByPath.get(note.path),
      }),
    ),
  ];

  if (preserveOpened) {
    const reservedPaths = new Set(nextTabs.map((tab) => tab.path));
    currentTabs
      .filter((tab) => tab.kind === "opened" && !reservedPaths.has(tab.path))
      .forEach((tab) => {
        nextTabs.push({ ...tab });
      });
  }

  const nextActiveIndex = previousActivePath
    ? nextTabs.findIndex((tab) => tab.path === previousActivePath)
    : 0;

  return {
    tabs: nextTabs,
    activeTabIndex: nextActiveIndex >= 0 ? nextActiveIndex : 0,
  };
}
