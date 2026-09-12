use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use crate::note_metadata::extract_tags;

pub(crate) struct NoteMetadataCache {
    vault_path: String,
    built_at: Instant,
    tags: Vec<String>,
}

impl NoteMetadataCache {
    pub(crate) fn build(vault_path: &str, note_paths: &[PathBuf]) -> Self {
        let built_at = Instant::now();
        let mut seen = HashSet::new();
        let mut tags = Vec::new();

        for note_path in note_paths {
            let Ok(content) = fs::read_to_string(note_path) else {
                continue;
            };

            for tag in extract_tags(&content) {
                let key = tag.to_lowercase();
                if seen.insert(key) {
                    tags.push(tag);
                }
            }
        }

        tags.sort_by_key(|tag| tag.to_lowercase());

        Self {
            vault_path: vault_path.to_string(),
            built_at,
            tags,
        }
    }

    pub(crate) fn tags(&self) -> &[String] {
        &self.tags
    }

    pub(crate) fn vault_path(&self) -> &str {
        &self.vault_path
    }

    pub(crate) fn built_at(&self) -> Instant {
        self.built_at
    }
}
