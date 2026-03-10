# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the Svelte frontend for the capture and settings windows. Main entry points are `src/main.js`, `src/settings-main.js`, `src/App.svelte`, and `src/Settings.svelte`. Shared browser utilities live in `src/lib/`.

`src-tauri/` contains the Rust backend and native integration. Keep Tauri commands and platform logic in `src-tauri/src/`, and app packaging/config in `src-tauri/tauri.conf.json`. Static assets live in `public/`, while bundled app icons live in `src-tauri/icons/`.

## Build, Test, and Development Commands
Run `npm install` once to install the Svelte, Vite, and Tauri CLI dependencies.

- `npm run dev` starts the Vite frontend only.
- `npm run tauri dev` runs the desktop app with the Rust backend and live frontend.
- `npm run build` builds the web frontend into `dist/`.
- `npm run tauri build` creates a macOS app bundle under `src-tauri/target/release/bundle/`.
- `cargo test --manifest-path src-tauri/Cargo.toml` runs backend tests when present.
- `cargo fmt --manifest-path src-tauri/Cargo.toml` formats Rust sources.

## Coding Style & Naming Conventions
Follow the existing split: Svelte components use PascalCase file names, JS helpers use lower camel case, and Rust modules use snake_case. Default to 2-space indentation in `src/` and let `rustfmt` handle Rust formatting in `src-tauri/`.

Match the current frontend style: ES modules, semicolons, and descriptive function names such as `handleSave` or `normalizeImageResult`. Keep platform-specific code isolated in Rust modules instead of embedding it in Svelte components.

## Testing Guidelines
There is no dedicated JS test suite configured yet. For frontend changes, validate behavior manually with `npm run tauri dev`, including capture flow, settings persistence, drag-and-drop, and tray interactions.

For Rust logic, prefer small unit tests next to the module you change and run `cargo test --manifest-path src-tauri/Cargo.toml` before opening a PR. Treat successful app startup and core capture/save flows as the minimum regression check.

## Commit & Pull Request Guidelines
Recent history uses short, Title Case subjects such as `Release Fix` and `Readme Changes 1.1`. Keep commit messages short, imperative, and scoped to one change, for example `Fix screenshot path validation`.

PRs should describe user-visible behavior, list manual test coverage, and include screenshots or GIFs for UI changes in `src/` or `public/screenshot/`. Link related issues when applicable and call out any macOS-specific setup or permission changes.
