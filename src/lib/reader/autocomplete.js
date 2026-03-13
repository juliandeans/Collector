export function getAutocompleteResults(query = "", vaultNotes = []) {
  if (!query) return vaultNotes.slice(0, 10);

  const lower = query.toLowerCase();
  return vaultNotes
    .filter(
      (note) =>
        note.name.toLowerCase().includes(lower) ||
        note.relative_path.toLowerCase().includes(lower),
    )
    .slice(0, 8);
}
