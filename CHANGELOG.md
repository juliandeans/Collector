# Changelog

All notable changes to Collector are documented here.  
Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [1.2.0] - 2026-05-27
### Fixed
- Pinned Reader notes are now stored as vault-relative paths, so they keep working after moving the vault
- Image folder is now stored as a vault-relative path; existing absolute paths are migrated automatically where possible
- Copy Text to Collector no longer inserts stale clipboard content when nothing is selected
- Copy Text to Collector now shows proper feedback when Accessibility permission is missing
- Append Picker: keyboard navigation no longer resets selection to the top when the list scrolls
- Save button in Settings now works correctly in all cases
- Open delay inputs in Settings keep the ms label aligned correctly

### Added
- *Command Palette* — open any note in your vault with Cmd+P or Cmd+K directly from the capture window
- *Append to Note* — two-step flow: choose any note in your vault, then append at the end or insert after a specific heading
- Separate open delay controls for the Note Window and Reader Window
- Global shortcuts for the Note Window and Reader Window can now optionally close the window with the same shortcut or a separate one
- Saving as a new note uses a leading # Heading as the note title and filename
- Entry header now supports 12-hour time format: h / hh (12h), a / A (am/pm)
- Note filename template now also supports 12-hour time tokens
- Update notifications: tray menu shows a notification when a new version is available on GitHub
- Settings redesigned into a less vibecoded and cleaner layout
- Activation: separate open delay controls and app exclusions are now configurable in Settings
---

## [1.1.1] - 2025

### Fixed
- Daily note folder paths now support date placeholders (e.g. `Journal/YYYY/MM`)
- Screenshot previews now display correctly in the capture window
- Blank preview box no longer appears when image preview fails
- Reader text color now applies consistently to all elements including headings and callouts
- Drag overlay in the Reader no longer gets stuck

### Added
- Note filename template is now configurable in Settings
- Option to hide callouts in the Reader
- Dual monitor support: edge detection and window positioning now work correctly across all monitors

---

## [1.1.0] - 2025

### Added
- Reader Panel: tab-based reading view with Daily Note tab and configurable pinned notes
- Hybrid Markdown editing: rendered reading view with raw Markdown editing on the active line
- Image rendering: inline support for `![[image.png]]` and standard Markdown image links
- Wikilink navigation: open `[[Note Links]]` in the current tab or a new tab with `Cmd+Click`
- Open in Obsidian: jump from the Reader straight into the current note
- Command palette: open any vault note with `Cmd+P` or `Cmd+K`
- Wikilink autocomplete: inline `[[` completion while typing
- Inline search: find and step through matches with `Cmd+F`
- Content filters: hide frontmatter, Dataview blocks, inline fields, HTML, and Obsidian comments
- Reader image import: drop or paste images directly into the Reader

---

## [1.0.x] - 2024–2025

Initial public releases. Core capture flow, Daily Note append, screenshot compression, edge detection, global shortcut, menu bar integration.
