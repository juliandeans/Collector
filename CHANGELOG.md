# Changelog

All notable changes to Collector are documented here.  
Format loosely follows [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [1.3.0] - 2026-09-12

### Added

- Clear Capture Shortcut: `Cmd+Backspace` (configurable under Settings → Shortcuts) empties the Capture window in one keystroke — text, images and the persisted draft — even when draft persistence is on; if the draft file cannot be removed, an error status is shown instead of the success banner.
- List continuation: pressing Enter at the end of a list item (`-`, `*`, `+`, `1.`) in the Capture window starts the next item automatically — checkbox items restart as unchecked, empty items are removed instead of continued, indentation is preserved.
- Reader: "Show Daily Note" checkbox (Settings → Reader → Pinned Notes, on by default) — when off, the Daily Note tab is not shown in the Reader window.
- Reader: pinned notes can be reordered in Settings with ↑/↓ buttons (same pattern as capture rules); tab order follows the list order.
- List indentation: Tab / Shift+Tab on a list item in the Capture window indents/outdents one step, aligned under the parent item's content column.
- Markdown basics: `Cmd+B` / `Cmd+I` in the Capture window wrap the selection with **bold** / *italic* markers, toggle them off when already applied, and insert an empty marker pair on empty selection.
- Capture Rules: capture text containing a user-defined trigger (word or
  hashtag, e.g. `#heute`) can be routed automatically into a chosen note —
  optionally appended under a specific heading, with or without the timestamp
  header. Rules are configured in Settings → Capture → Rules (ordered list,
  first match wins, no rules = standard behavior); while typing, the Capture
  window shows live at the bottom edge which note the text would land in.
- Capture Mode / Auto-Collect: automatically appends copied plain text from
  the clipboard into the Capture window while active. Can be toggled via
  the "Collect" button in the action bar or a global keyboard shortcut.
- Capture draft persistence: unsubmitted text in the Capture window is
  now preserved across closes (Escape, clicking outside, toggle shortcut)
  and restored when reopening. Can be disabled in Settings → Capture.
- Window geometry memory: Capture and Reader windows can now remember their
  custom size and position across sessions (configurable in Settings).
- OCR (image to text): a configurable shortcut (default Cmd+Shift+T) extracts
  text from a clipboard image at the cursor, in both Capture and the Reader.
  Dragging an image while holding a configurable modifier key (default
  Option) does the same instead of the normal image import. Runs fully
  locally via Apple's Vision framework — no image ever leaves the device.
- Configurable "Next Tab" / "Previous Tab" keyboard shortcuts in the Reader
  (default: Option+Right / Option+Left).
- Configurable "Toggle Keep Unsaved Text" shortcut (default: Cmd+Shift+D)
  to toggle draft persistence on and off from anywhere or inside Capture,
  configurable in Settings → Shortcuts → Capture Window.
- Keyboard shortcuts in Settings can now be disabled: press Backspace or
  Delete inside any shortcut field, or click the clear button (✕) that
  appears on hover or focus.
- Tag autocomplete: typing `#` in the Capture textarea or the Reader editor
  opens a suggestion dropdown with every tag found in the vault (inline
  `#hashtags` plus frontmatter `tags:`), filterable while typing and
  selectable with arrow keys (Enter inserts `#tag`, Escape closes).
  A bare `#` right before the caret shows the full tag list, like Obsidian.
  Tags are indexed once per session in the background (best-effort: files
  that cannot be read, e.g. not-yet-downloaded iCloud items, are skipped);
  the list refreshes on window show and manual reindex.

### Changed

- Background blur switched to macOS native `NSVisualEffectView` (WindowServer)
  instead of CSS `backdrop-filter`. Eliminates hard unblurred strips along
  screen edges when windows dock flush, and offers four clean native strengths
  (Off / Light / Medium / Strong) instead of a continuous slider.
- Settings → "Look & Feel" is now "Appearance", reorganized into cleaner
  sections: Background Blur is a 4-step segmented control instead of a slider,
  slider values display beside tracks, Background Brightness has a visible
  zero mark, the color tint is part of "Glass Effect" as "Tint" fields, and
  Corner Radius / "Round all four corners" moved into a new "Window Shape"
  section below Glass Effect.
- By default, the two window corners against the screen edge Capture/Reader
  dock to now stay square instead of rounded — only the corners facing the
  screen's interior are rounded, so the flush edge reads as one clean line.
  Enable "Round all four corners" in Settings → Appearance → Window Shape to
  get the old all-rounded look back.
- The "Save Actions" shortcuts (Save to Daily Note, Create New Note, Append
  to Note) now sit right below "Capture Window" in Settings → Shortcuts
  instead of at the bottom of the tab.
- The wikilink suggestion popups (Reader and Capture), the "append to note"
  picker, and the command palette now share one consistent visual design
  (background, accent highlight bar, text colors) instead of three
  independently-styled looks that had quietly drifted apart.
- The muted/secondary text color used throughout the Reader and its popups
  (note paths, headings, quotes, list markers) now derives automatically
  from your chosen Text Color setting instead of being a fixed color —
  adapts correctly to both dark and light text setups.
- Window dragging vs. text selection: Disabled macOS `setMovableByWindowBackground`
  so clicking and dragging in textareas and the note editor selects text normally
  without moving the window. Moving windows is now performed via `Cmd+Drag` anywhere
  across the window (Capture, Reader, Settings), while standard titlebar dragging
  in the Reader topbar remains intact. Interactive links (such as wikilinks) and
  controls remain fully clickable with Cmd.
- Internal architecture refactoring: modularized the 7 largest files in the
  codebase (shortcuts registration, capture window actions, settings themes
  and styles, capture tests/templates, reader image/content preprocessing,
  and editor/reader coordination), reducing monolithic file sizes by over
  2,300 lines without changing behavior or public APIs.

### Fixed

- Fixed draft persistence shortcut toggle order so switching on immediately writes unsubmitted text to disk before background gates check setting state.
- Tag suggestion dropdowns no longer let the scrollbar poke into the panel's
  rounded corners: the scroll area is inset inside the glass panel
  (Capture wikilink/tag picker).
- Prevented creating an empty note when the Capture window contains only a
  heading.
- Prevented potential race conditions and dropped changes during settings
  auto-save and window geometry debouncing.
- Fixed OCR shortcut (Cmd+Shift+T) not responding globally or when copying image
  files directly in Finder.
- Fixed off-screen window restore bugs on multi-monitor setups.
- The shortcut recorder in Settings now shows "Option" instead of "Alt"
  when capturing a key combo that uses the Option key.
- Wikilink suggestion path text in the Reader rendered in black instead of
  a legible muted color.
- The Reader's wikilink suggestion popup could render partly or fully
  off-screen and become invisible when opened near the right edge of the
  window. It now stays within the window, matching how the Capture
  window's popup already behaved.
- Rounded window corners showed gray, unrounded artifacts in the bottom
  corners of the Capture and Reader windows on macOS 26 (Tahoe). The corner
  radius is now applied directly to the WKWebView's own layer instead of
  reaching into WebKit's internal sublayers, which macOS 26 can replace
  out from under the app.

## [1.2.5] - 2026-06-09

### Added

- Configurable keyboard shortcut for "Open Command Palette" in the Reader
  (default: Cmd+P). Can be set in Settings → Shortcuts.
- Configurable keyboard shortcut for "Open in Obsidian" in the Reader
  (default: Cmd+Shift+O).
- Configurable keyboard shortcut for "Navigate Back" in the Reader
  (disabled by default). Keyboard-only, bottom bar currently disabled.
- Hover close button for unpinned Reader tabs.
- Reader can now create a missing Daily Note via the configured "Save to
  Daily Note" shortcut when automatic Daily Note creation is enabled.

### Changed

- Image drop confirmation toast ("X images added") removed from capture window.
- Reader palette shortcuts are now fully configurable instead of hardcoded
  to Cmd+K/Cmd+P.
- Reader action shortcuts are read from the current settings at keypress time,
  so changes apply without reopening the Reader window.
- Transient "Saved ✓" indicator removed from Reader top bar.
  Saves happen silently.
- Settings validation messages now use softer error and warning colors.

### Fixed

- Memory footprint growing unboundedly during Reader use. Base64-encoded
  image data was cached indefinitely in the frontend — every image ever
  opened in the Reader accumulated for the lifetime of the app, reaching
  1+ GB after extended use. The cache is now capped at 150 entries (LRU
  eviction) and fully cleared when the Reader window closes.
- Internal note index was storing each note's absolute path twice per
  entry, wasting memory across all IPC calls and frontend windows.
- Images larger than 20 MB now return an error instead of being loaded
  into memory for encoding.
- Custom Reader shortcuts saved in Settings now load into the Reader correctly,
  including live updates while the Reader is open.
- Reader shortcut matching now handles aliases generated by the Settings UI,
  including arrow keys and Space.
- Reader tab close controls now use valid button markup, avoiding nested
  interactive elements and inconsistent click/focus behaviour.
- Setting the same keyboard shortcut for two different features (e.g.
  capture shortcut and reader shortcut) now shows a validation error in
  Settings instead of silently breaking one of them.
- Creating a missing Daily Note from the Reader via shortcut could trigger
  multiple Obsidian launches if the shortcut fired in quick succession
  before the file appeared on disk. A re-entry guard now prevents this.
- Reader lost DOM focus after closing Command Palette or SearchBar
  overlay, requiring a click before typing or shortcuts worked again.
- Creating a missing Daily Note from the Reader now automatically creates
  the parent folder if it does not exist, consistent with how the Capture
  window handles missing folders.
- Screenshots captured via Collector now display immediately in the Reader
  without requiring a manual reindex.
- Notes already open in the Reader now re-render correctly after a manual
  reindex, resolving previously blank images.

---

## [1.2.4] - 2026-06-04

### Fixed

- Daily note folder paths with date tokens (e.g. YYYY/MM/) now
  automatically create subdirectories when a new month or year begins
- Notes folder path now supports date tokens (YYYY, MM, DD, HH, mm, ss) —
  e.g. "YYYY/MM/" correctly resolves to "2026/06/" when saving a new note

---

## [1.2.3] - 2026-06-03

### Added

- Captures can now be inserted under a specific heading in the daily
  note (configurable target heading, top or bottom of the section).
  The heading can be auto-created if missing; falls back to appending
  at the end when no target is set.
- Daily notes can now be created automatically when missing —
  Collector opens Obsidian (Advanced URI plugin necessary), waits for
  the file to appear, and writes the capture into the configured
  section. Timeout and stability-check are configurable.
- Action bar at the bottom of the capture window with three buttons:
  Daily Note, New Note, Append — same actions as the keyboard
  shortcuts. Can be toggled off in Settings.
- Settings now save automatically after a short debounce. No Save button
  required.
- Setting sections are now collapsible — click a heading to collapse
  or expand.
- **Dark mode for Settings.** The settings window now follows your
  system appearance (light / dark). Clean, two proper themes with
  custom-styled sliders, adaptive inputs, and consistent colours
  across all panels.
- Three new setting panels: **Vault**, **Capture**, **Reader**.
  The old Obsidian Integration and
  Note/Capture Window panels were split and reorganised.
- `Cmd + ,` now opens Settings from the Reader window too (was
  capture-window only).
- Custom styled checkboxes across all Settings panels — bigger,
  border-radius matching inputs, accent colour, light/dark mode
  support. Orange native checkbox gone.
- Reset All in Settings now shows a native confirmation dialog
  before resetting (vault connection is preserved).
- Pinned notes can now be added via search palette instead of native
  file picker.

### Changed

- Daily note captures now use section-insert logic instead of raw
  file append. Existing behaviour is preserved when no target heading
  is configured.
- Section headings in settings larger (16 px) with more spacing.
- Settings panel labels tidied up: "Look" → "Look & Feel",
  "Note Window" → "Capture Window" throughout.
- Success labels ("✓ Saved", "✓ …", "✓ Appended") removed from all
  three capture actions. Window closing is now the only success
  feedback. Error messages remain and keep the window open.

### Fixed

- Hardcoded "Journal" default for daily note folder removed — user-configured path is now respected; an error is shown if the folder is empty or missing.
- Daily Notes folder is no longer silently created if it doesn't exist — an error is shown immediately instead.
- Daily Note tab now uses a safer mechanism to always reflect the current day
  when the Reader is opened.
- Capture text shortcut (Cmd+Shift+C) now reliably reads selected text
  via clipboard sentinel instead of unreliable AXSelectedText API.
- Blur and saturation had no effect when Brightness was set to 0.
  Empty brightness filter string made the entire `backdrop-filter`
  CSS declaration invalid. Now always emits `brightness(1)` as a
  valid no-op identity filter.
- `append_to_note` failed with "Bad file descriptor (os error 9)"
  when the note already existed. File was opened write-only; added
  `.read(true)` to OpenOptions so the trailing-newline check works.
- Error message when waiting for the daily note no longer contains
  awkward extra whitespace.
- Settings window was white-on-white for dark mode users — now
  detects system preference and renders a proper dark theme.
- Removed unused CSS selector in Settings.
- Reset All now preserves the current vault connection and saves the
  reset automatically.
- Frontend defaults now include `vault_path`, preventing resets from
  producing an undefined vault path.
- Programmatic Settings changes now trigger auto-save for recorded
  shortcuts, excluded edge apps, and pinned note icon/label changes.
- Legacy `daily_note_path` configs are now migrated reliably to
  `daily_note_folder` and `daily_note_format` without overwriting newer
  configs.

---

## [1.2.2] - 2026-05-31

### Fixed

- Append to Note: selecting a note no longer freezes the capture
  window on macOS (WKWebView timing issue during step 1 → step 2
  transition)
- Append to Note: keyboard navigation (arrow keys) now works
  correctly in the heading picker

---

## [1.2.1] - 2026-05-27

### Fixed

- Reader: active line no longer highlights the entire line — only the cursor is shown
- Overlay styling unified across Command Palette, Append Picker, and Wikilink autocomplete (consistent typography, colors, and shadows)
- Global CSS variables now load correctly in the Note Window, fixing path colors and other visual inconsistencies in overlays

### Added

- `[[` Wikilink autocomplete in the Note Window — type `[[` to search and insert vault links while capturing
- Setting: toggle vault-relative paths in all note pickers (Command Palette, Append Picker, Wikilink autocomplete)
- Setting: configure how many results are shown in note pickers (default: 20)

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
