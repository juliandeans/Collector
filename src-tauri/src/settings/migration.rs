use std::fs;
use std::path::{Path, PathBuf};

use crate::log_safety::redact_path_str;

use super::Settings;

fn normalize_relative_vault_path(path: &str) -> String {
    path.trim()
        .replace('\\', "/")
        .split('/')
        .filter(|segment| !segment.is_empty() && *segment != ".")
        .collect::<Vec<_>>()
        .join("/")
}

fn relative_path_from_matching_vault_dir(path: &str, vault_path: &str) -> Option<String> {
    let vault_dir = Path::new(vault_path).file_name()?.to_str()?;
    let normalized_path = path.trim().replace('\\', "/");
    let segments = normalized_path
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let vault_index = segments
        .iter()
        .rposition(|segment| segment.eq_ignore_ascii_case(vault_dir))?;

    if vault_index + 1 >= segments.len() {
        return None;
    }

    let relative = segments[vault_index + 1..].join("/");
    let normalized = normalize_relative_vault_path(&relative);

    if normalized.is_empty() {
        None
    } else {
        Some(normalized)
    }
}

fn normalize_pinned_note_path(path: &str, vault_path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let candidate = PathBuf::from(trimmed);
    if !candidate.is_absolute() {
        return normalize_relative_vault_path(trimmed);
    }

    let vault_root = fs::canonicalize(vault_path).unwrap_or_else(|_| PathBuf::from(vault_path));
    if let Ok(relative) = candidate.strip_prefix(&vault_root) {
        let normalized = normalize_relative_vault_path(&relative.to_string_lossy());
        if !normalized.is_empty() {
            return normalized;
        }
    }

    if let Some(relative) =
        relative_path_from_matching_vault_dir(trimmed, &vault_root.to_string_lossy())
    {
        return relative;
    }

    trimmed.replace('\\', "/")
}

fn normalize_screenshot_dir_path(path: &str, vault_path: &str) -> String {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if trimmed == "." {
        return ".".to_string();
    }

    let candidate = PathBuf::from(trimmed);
    if !candidate.is_absolute() {
        let normalized = normalize_relative_vault_path(trimmed);
        return if normalized.is_empty() && trimmed == "." {
            ".".to_string()
        } else {
            normalized
        };
    }

    let vault_root = fs::canonicalize(vault_path).unwrap_or_else(|_| PathBuf::from(vault_path));
    if let Ok(relative) = candidate.strip_prefix(&vault_root) {
        let relative_str = relative.to_string_lossy();
        let normalized = normalize_relative_vault_path(&relative_str);
        return if normalized.is_empty() {
            ".".to_string()
        } else {
            normalized
        };
    }

    if let Some(relative) =
        relative_path_from_matching_vault_dir(trimmed, &vault_root.to_string_lossy())
    {
        return relative;
    }

    trimmed.replace('\\', "/")
}

fn raw_setting_exists(raw_settings: Option<&serde_json::Value>, field: &str) -> bool {
    raw_settings.and_then(|value| value.get(field)).is_some()
}

impl Settings {
    pub(crate) fn normalize_pinned_note_paths(&mut self) -> bool {
        let mut changed = false;

        for pinned_note in &mut self.pinned_notes {
            let normalized = normalize_pinned_note_path(&pinned_note.path, &self.vault_path);
            if pinned_note.path != normalized {
                pinned_note.path = normalized;
                changed = true;
            }
        }

        changed
    }

    pub(crate) fn normalize_screenshot_path(&mut self) -> bool {
        let normalized = normalize_screenshot_dir_path(&self.screenshot_path, &self.vault_path);
        let mut changed = false;

        if self.screenshot_path != normalized {
            self.screenshot_path = normalized.clone();
            changed = true;
        }

        if self.image_folder != self.screenshot_path {
            self.image_folder = self.screenshot_path.clone();
            changed = true;
        }

        changed
    }

    pub(super) fn migrate_from_content(content: &str) -> Result<(Self, bool), String> {
        let raw_settings = serde_json::from_str::<serde_json::Value>(content).ok();
        let raw_settings_ref = raw_settings.as_ref();
        let has_screenshot_path = raw_setting_exists(raw_settings_ref, "screenshot_path");
        let has_daily_note_folder = raw_setting_exists(raw_settings_ref, "daily_note_folder");
        let has_daily_note_format = raw_setting_exists(raw_settings_ref, "daily_note_format");

        let mut settings: Settings = serde_json::from_str(content)
            .map_err(|e| format!("Failed to parse config file: {}", e))?;

        let mut needs_save = false;

        if !has_screenshot_path && !settings.image_folder.trim().is_empty() {
            settings.screenshot_path = settings.image_folder.trim().to_string();
            needs_save = true;
        }

        // Migration: convert old daily_note_path to new fields.
        // Decide from the raw JSON fields, because serde defaults already populate
        // daily_note_folder / daily_note_format before migration runs.
        if !settings.daily_note_path.trim().is_empty() {
            let path = settings.daily_note_path.trim().to_string();

            if !has_daily_note_folder && !has_daily_note_format {
                if let Some(last_slash) = path.rfind('/') {
                    settings.daily_note_folder = path[..=last_slash].to_string();
                    let filename = &path[last_slash + 1..];

                    settings.daily_note_format =
                        filename.strip_suffix(".md").unwrap_or(filename).to_string();
                } else {
                    settings.daily_note_format =
                        path.strip_suffix(".md").unwrap_or(&path).to_string();
                }

                log::info!(
                    "Migrated daily_note_path (folder={}, format_chars={})",
                    redact_path_str(&settings.daily_note_folder),
                    settings.daily_note_format.chars().count()
                );
            }

            settings.daily_note_path = String::new();
            needs_save = true;
        }

        if settings.normalize_pinned_note_paths() {
            log::info!("Migrated pinned note paths to vault-relative form");
            needs_save = true;
        }

        if settings.normalize_screenshot_path() {
            log::info!("Migrated screenshot path to vault-relative form");
            needs_save = true;
        }

        // Migration: if old field names exist in raw config, trigger a save so serde
        // writes only the new names (overlay_color, overlay_strength). serde aliases
        // handle deserialization, so the values are already loaded correctly.
        if let Some(raw) = raw_settings.as_ref() {
            if raw.get("background_color").is_some() && raw.get("overlay_color").is_none() {
                log::info!("Marking config for re-save: background_color → overlay_color");
                needs_save = true;
            }
            if raw.get("window_transparency").is_some() && raw.get("overlay_strength").is_none() {
                log::info!("Marking config for re-save: window_transparency → overlay_strength");
                needs_save = true;
            }
        }

        Ok((settings, needs_save))
    }
}

#[cfg(test)]
mod tests {
    use super::super::defaults::default_daily_note_folder;
    use crate::settings::{PinnedNote, Settings};

    fn config_json_with_legacy_daily_path(daily_note_path: &str) -> String {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();

        object.insert(
            "daily_note_path".to_string(),
            serde_json::json!(daily_note_path),
        );
        object.remove("daily_note_folder");
        object.remove("daily_note_format");

        serde_json::to_string(&value).unwrap()
    }

    #[test]
    fn migrates_absolute_pinned_note_path_inside_current_vault() {
        let mut settings = Settings {
            vault_path: "/tmp/Vault".to_string(),
            pinned_notes: vec![PinnedNote {
                path: "/tmp/Vault/Folder/Note.md".to_string(),
                label: String::new(),
                icon: String::new(),
            }],
            ..Default::default()
        };

        assert!(settings.normalize_pinned_note_paths());
        assert_eq!(settings.pinned_notes[0].path, "Folder/Note.md");
    }

    #[test]
    fn migrates_absolute_pinned_note_path_by_vault_dir_name() {
        let mut settings = Settings {
            vault_path: "/new/location/Vault".to_string(),
            pinned_notes: vec![PinnedNote {
                path: "/old/location/Vault/Folder/Note.md".to_string(),
                label: String::new(),
                icon: String::new(),
            }],
            ..Default::default()
        };

        assert!(settings.normalize_pinned_note_paths());
        assert_eq!(settings.pinned_notes[0].path, "Folder/Note.md");
    }

    #[test]
    fn rejects_absolute_pinned_note_path_on_validate() {
        let settings = Settings {
            pinned_notes: vec![PinnedNote {
                path: "/tmp/Vault/Folder/Note.md".to_string(),
                label: String::new(),
                icon: String::new(),
            }],
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn migrates_absolute_screenshot_path_inside_current_vault() {
        let mut settings = Settings {
            vault_path: "/tmp/Vault".to_string(),
            screenshot_path: "/tmp/Vault/Grafiken/Screenshots".to_string(),
            ..Default::default()
        };

        assert!(settings.normalize_screenshot_path());
        assert_eq!(settings.screenshot_path, "Grafiken/Screenshots");
        assert_eq!(settings.image_folder, "Grafiken/Screenshots");
    }

    #[test]
    fn migrates_absolute_screenshot_path_by_vault_dir_name() {
        let mut settings = Settings {
            vault_path: "/new/location/Vault".to_string(),
            screenshot_path: "/old/location/Vault/Grafiken/Screenshots".to_string(),
            ..Default::default()
        };

        assert!(settings.normalize_screenshot_path());
        assert_eq!(settings.screenshot_path, "Grafiken/Screenshots");
    }

    #[test]
    fn rejects_absolute_screenshot_path_on_validate() {
        let settings = Settings {
            screenshot_path: "/tmp/Vault/Grafiken/Screenshots".to_string(),
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn migrates_legacy_daily_note_path_with_folder() {
        let content = config_json_with_legacy_daily_path("Tagebuch/YYYY-MM-DD.md");
        let (settings, needs_save) = Settings::migrate_from_content(&content).unwrap();

        assert!(needs_save);
        assert_eq!(settings.daily_note_folder, "Tagebuch/");
        assert_eq!(settings.daily_note_format, "YYYY-MM-DD");
        assert_eq!(settings.daily_note_path, "");
    }

    #[test]
    fn migrates_legacy_daily_note_path_without_folder() {
        let content = config_json_with_legacy_daily_path("YYYY-MM-DD.md");
        let (settings, needs_save) = Settings::migrate_from_content(&content).unwrap();

        assert!(needs_save);
        assert_eq!(settings.daily_note_folder, default_daily_note_folder());
        assert_eq!(settings.daily_note_format, "YYYY-MM-DD");
        assert_eq!(settings.daily_note_path, "");
    }

    #[test]
    fn keeps_new_daily_note_fields_when_legacy_path_is_also_present() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();

        object.insert(
            "daily_note_path".to_string(),
            serde_json::json!("Legacy/YYYY-MM-DD.md"),
        );
        object.insert(
            "daily_note_folder".to_string(),
            serde_json::json!("Journal/"),
        );
        object.insert(
            "daily_note_format".to_string(),
            serde_json::json!("YYYY_MM_DD"),
        );

        let content = serde_json::to_string(&value).unwrap();
        let (settings, needs_save) = Settings::migrate_from_content(&content).unwrap();

        assert!(needs_save);
        assert_eq!(settings.daily_note_folder, "Journal/");
        assert_eq!(settings.daily_note_format, "YYYY_MM_DD");
        assert_eq!(settings.daily_note_path, "");
    }

    #[test]
    fn migrates_old_field_names_overlay_color_and_overlay_strength() {
        // Simulate an old config with background_color and window_transparency
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();

        object.insert("background_color".to_string(), serde_json::json!("#123456"));
        object.insert("window_transparency".to_string(), serde_json::json!(42));
        object.remove("overlay_color");
        object.remove("overlay_strength");

        let content = serde_json::to_string(&value).unwrap();
        let (settings, needs_save) = Settings::migrate_from_content(&content).unwrap();

        // Values from old fields are loaded via serde aliases
        assert_eq!(settings.overlay_color, "#123456");
        assert_eq!(settings.overlay_strength, 42);
        assert!(needs_save, "old field names should trigger re-save");

        // After serialization, only new names should appear
        let serialized = serde_json::to_string(&settings).unwrap();
        assert!(
            serialized.contains("\"overlay_color\""),
            "serialized config must use overlay_color"
        );
        assert!(
            serialized.contains("\"overlay_strength\""),
            "serialized config must use overlay_strength"
        );
        assert!(
            !serialized.contains("\"background_color\""),
            "serialized config must NOT contain old background_color"
        );
        assert!(
            !serialized.contains("\"window_transparency\""),
            "serialized config must NOT contain old window_transparency"
        );
    }
}
