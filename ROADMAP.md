# Roadmap

This is a rough outline of what's planned for Collector.
Features may shift between versions based on feedback.

---

## In Progress / Next

- `[[` Wikilink autocomplete in the Note Window
- Light / Dark Mode for all windows

---

## Planned

### Reader
- **Daily Note toggle** — option to disable Daily Note functionality in the Reader entirely for users who don't use it
- **Pinned notes sorting** — ability to reorder pinned notes in Settings

### Capture
- **Capture content retention** — setting to choose whether the Capture window content is cleared after saving or preserved for the next session
- **Smart capture routing rules** — user-defined rules that automatically route captured content based on text patterns. Example: any capture containing #random is appended to a configured note; any capture containing @badezimmer is routed to the note "Renovierung" under the section "## Badezimmer". Intended for power users who want context-aware capture destinations without manual selection.
- OCR: capture text from a screen region to note window
- Source URL: automatically append the active browser tab URL
- basic markdown support (bold, italic, etc.)

### Vault
- **Multi-vault support** — ability to configure and switch between multiple Obsidian vaults

### Settings & Integrations
- Obsidian theme sync: import colors and fonts from your active theme
- Daily Note template support
- URL scheme: `collector://capture?text=...` for Alfred / Raycast

---

## Maybe
- App-based auto-tagging
- Windows / Linux support
