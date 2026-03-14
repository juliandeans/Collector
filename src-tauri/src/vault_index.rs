use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use crate::image_handler;

pub struct VaultIndex {
    images_by_name: HashMap<String, PathBuf>,
    images_by_rel_path: HashMap<String, PathBuf>,
    notes_by_name: HashMap<String, PathBuf>,
    notes_by_rel_path: HashMap<String, PathBuf>,
    pub vault_path: String,
    pub built_at: Instant,
    pub file_count: usize,
}

#[derive(serde::Serialize, Clone)]
pub struct NoteEntry {
    pub name: String,
    pub relative_path: String,
    pub path: String,
    pub absolute_path: String,
}

impl VaultIndex {
    pub fn build(vault_path: &str) -> Result<Self, String> {
        let vault_root = fs::canonicalize(vault_path)
            .map_err(|e| format!("Failed to resolve vault path: {}", e))?;

        let mut index = Self {
            images_by_name: HashMap::new(),
            images_by_rel_path: HashMap::new(),
            notes_by_name: HashMap::new(),
            notes_by_rel_path: HashMap::new(),
            vault_path: vault_root.to_string_lossy().to_string(),
            built_at: Instant::now(),
            file_count: 0,
        };

        index.walk_dir(&vault_root, &vault_root)?;
        Ok(index)
    }

    pub fn resolve_image(&self, path: &str) -> Option<&PathBuf> {
        let normalized = normalize_lookup_path(path);
        if normalized.contains('/') {
            return self.images_by_rel_path.get(&normalized);
        }

        self.images_by_name.get(&normalized)
    }

    pub fn resolve_note(&self, name: &str) -> Option<&PathBuf> {
        let normalized = normalize_lookup_path(name);
        if normalized.contains('/') {
            let with_extension = if normalized.ends_with(".md") {
                normalized.clone()
            } else {
                format!("{normalized}.md")
            };

            return self
                .notes_by_rel_path
                .get(&normalized)
                .or_else(|| self.notes_by_rel_path.get(&with_extension));
        }

        self.notes_by_name.get(&normalize_note_lookup(name))
    }

    pub fn all_notes(&self) -> Vec<NoteEntry> {
        let vault_root = Path::new(&self.vault_path);
        let mut seen = HashSet::new();
        let mut notes = self
            .notes_by_rel_path
            .values()
            .filter(|path| seen.insert(path.to_string_lossy().to_string()))
            .filter_map(|absolute_path| {
                let absolute_string = absolute_path.to_string_lossy().to_string();
                let name = absolute_path.file_stem()?.to_string_lossy().to_string();
                let relative_path = absolute_path
                    .strip_prefix(vault_root)
                    .unwrap_or(absolute_path)
                    .to_string_lossy()
                    .replace('\\', "/");

                Some(NoteEntry {
                    name,
                    relative_path,
                    path: absolute_string.clone(),
                    absolute_path: absolute_string,
                })
            })
            .collect::<Vec<_>>();

        notes.sort_by(|a, b| a.name.cmp(&b.name));

        notes
    }

    fn walk_dir(&mut self, vault_root: &Path, dir: &Path) -> Result<(), String> {
        let mut entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read vault directory: {}", e))?
            .filter_map(Result::ok)
            .collect::<Vec<_>>();

        entries.sort_by(|a, b| {
            a.file_name()
                .to_string_lossy()
                .to_lowercase()
                .cmp(&b.file_name().to_string_lossy().to_lowercase())
        });

        for entry in entries {
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_type = entry
                .file_type()
                .map_err(|e| format!("Failed to inspect vault entry: {}", e))?;

            if file_name.starts_with('.') {
                continue;
            }

            if file_type.is_symlink() {
                continue;
            }

            if file_type.is_dir() {
                self.walk_dir(vault_root, &path)?;
                continue;
            }

            let relative_path = path
                .strip_prefix(vault_root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");

            if path.extension().map(|ext| ext == "md").unwrap_or(false) {
                let note_name = path
                    .file_stem()
                    .map(|stem| stem.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let relative_key = relative_path.to_lowercase();

                self.notes_by_rel_path
                    .entry(relative_key)
                    .or_insert_with(|| path.clone());
                self.notes_by_name
                    .entry(note_name)
                    .or_insert_with(|| path.clone());
                self.file_count += 1;
                continue;
            }

            if image_handler::is_supported_image(&path) {
                let image_name = path
                    .file_name()
                    .map(|name| name.to_string_lossy().to_lowercase())
                    .unwrap_or_default();
                let relative_key = relative_path.to_lowercase();

                self.images_by_rel_path
                    .entry(relative_key)
                    .or_insert_with(|| path.clone());
                self.images_by_name
                    .entry(image_name)
                    .or_insert_with(|| path.clone());
                self.file_count += 1;
            }
        }

        Ok(())
    }
}

fn normalize_lookup_path(path: &str) -> String {
    path.trim().replace('\\', "/").to_lowercase()
}

fn normalize_note_lookup(name: &str) -> String {
    let trimmed = name.trim().replace('\\', "/");
    trimmed
        .strip_suffix(".md")
        .unwrap_or(&trimmed)
        .to_lowercase()
}
