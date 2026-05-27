export function getAutocompleteResults(query = "", vaultNotes = [], limit = 20) {
  if (!query) return vaultNotes.slice(0, limit);

  const lower = query.toLowerCase();
  return vaultNotes
    .filter(
      (note) =>
        note.name.toLowerCase().includes(lower) ||
        note.relative_path.toLowerCase().includes(lower),
    )
    .slice(0, limit);
}
