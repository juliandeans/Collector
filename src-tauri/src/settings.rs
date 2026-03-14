use serde::{Deserialize, Deserializer, Serialize};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::log_safety::redact_path_str;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinnedNote {
    pub path: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PinnedNoteInput {
    LegacyPath(String),
    Structured(PinnedNote),
}

fn deserialize_pinned_notes<'de, D>(deserializer: D) -> Result<Vec<PinnedNote>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries = Vec::<PinnedNoteInput>::deserialize(deserializer)?;

    Ok(entries
        .into_iter()
        .filter_map(|entry| match entry {
            PinnedNoteInput::LegacyPath(path) => {
                let trimmed = path.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(PinnedNote {
                        path: trimmed,
                        label: String::new(),
                        icon: String::new(),
                    })
                }
            }
            PinnedNoteInput::Structured(note) => {
                if note.path.trim().is_empty() {
                    None
                } else {
                    Some(PinnedNote {
                        path: note.path.trim().to_string(),
                        label: note.label.trim().to_string(),
                        icon: note.icon.trim().to_string(),
                    })
                }
            }
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub vault_name: String,

    #[serde(default = "default_vault_path")]
    pub vault_path: String,

    #[serde(default = "default_screenshot_path")]
    pub screenshot_path: String,

    pub edge_side: String,
    #[serde(default = "default_window_width")]
    pub window_width: u32,
    #[serde(default = "default_window_height")]
    pub window_height: u32,
    #[serde(default = "default_reader_width")]
    pub reader_width: u32,
    #[serde(default = "default_reader_height")]
    pub reader_height: u32,
    pub border_radius: u32,
    pub background_color: String,
    pub font_family: String,
    pub font_size: u32,
    #[serde(default = "default_daily_note_folder")]
    pub daily_note_folder: String,
    #[serde(default = "default_daily_note_format")]
    pub daily_note_format: String,
    #[serde(default)]
    pub daily_note_path: String,
    pub image_folder: String,
    pub image_filename: String,
    #[serde(default = "default_image_width")]
    pub default_image_width: String,
    pub entry_header: String,
    pub global_shortcut: String,
    #[serde(default = "default_false")]
    pub global_shortcut_closes_window: bool,
    #[serde(default = "default_capture_text_shortcut")]
    pub capture_text_shortcut: String,
    pub compression_max_kb: u32,
    #[serde(default = "default_edge_enabled")]
    pub edge_detection_enabled: bool,
    #[serde(default = "default_reaction_time_ms")]
    pub edge_reaction_time_ms: u64,
    #[serde(default)]
    pub edge_modifier_keys: Vec<String>,
    #[serde(default)]
    pub edge_excluded_apps: Vec<String>,
    #[serde(default = "default_notes_folder")]
    pub notes_folder: String,
    #[serde(default = "default_save_to_daily_shortcut")]
    pub save_to_daily_shortcut: String,
    #[serde(default = "default_save_as_note_shortcut")]
    pub save_as_note_shortcut: String,
    #[serde(default, deserialize_with = "deserialize_pinned_notes")]
    pub pinned_notes: Vec<PinnedNote>,
    #[serde(default = "default_reader_shortcut")]
    pub reader_shortcut: String,
    #[serde(default = "default_false")]
    pub reader_shortcut_closes_window: bool,
    #[serde(default = "default_true")]
    pub reader_edge_enabled: bool,
    #[serde(default = "default_true")]
    pub reader_hide_frontmatter: bool,
    #[serde(default = "default_true")]
    pub reader_hide_dataview: bool,
    #[serde(default = "default_true")]
    pub reader_hide_obsidian_comments: bool,
    #[serde(default = "default_true")]
    pub reader_hide_inline_fields: bool,
    #[serde(default = "default_true")]
    pub reader_hide_html: bool,
    #[serde(default = "default_note_filename_template")]
    pub note_filename_template: String,
    #[serde(default = "default_note_template")]
    pub note_template: String,
    #[serde(default = "default_window_transparency")]
    pub window_transparency: u32,
    #[serde(default = "default_window_blur")]
    pub window_blur: u32,
    #[serde(default = "default_window_saturation")]
    pub window_saturation: u32,
    #[serde(default = "default_window_brightness")]
    pub window_brightness: i32,
    #[serde(default = "default_autostart_enabled")]
    pub autostart_enabled: bool,
    #[serde(default = "default_text_color")]
    pub text_color: String,
    #[serde(default = "default_accent_color")]
    pub accent_color: String,
    #[serde(default = "default_internal_link_color")]
    pub internal_link_color: String,
    #[serde(default = "default_external_link_color")]
    pub external_link_color: String,
}

fn default_autostart_enabled() -> bool {
    false
}
fn default_edge_enabled() -> bool {
    true
}

fn default_true() -> bool {
    true
}

fn default_false() -> bool {
    false
}

fn default_reaction_time_ms() -> u64 {
    50
}

fn default_vault_path() -> String {
    dirs::home_dir()
        .map(|h| h.join("Vault").to_string_lossy().to_string())
        .unwrap_or_else(|| "/Users/Vault".to_string())
}

fn default_screenshot_path() -> String {
    dirs::home_dir()
        .map(|h| {
            h.join("Vault/Grafiken/Screenshots")
                .to_string_lossy()
                .to_string()
        })
        .unwrap_or_else(|| "/Users/Vault/Grafiken/Screenshots".to_string())
}

fn default_notes_folder() -> String {
    "Notes".to_string()
}

fn default_save_to_daily_shortcut() -> String {
    "Cmd+Enter".to_string()
}

fn default_save_as_note_shortcut() -> String {
    "Cmd+Shift+Enter".to_string()
}

fn default_window_width() -> u32 {
    350
}

fn default_window_height() -> u32 {
    600
}

fn default_reader_shortcut() -> String {
    "Cmd+Shift+R".to_string()
}

fn default_note_filename_template() -> String {
    "note-YYYY-MM-DD-HHmmss".to_string()
}

fn default_note_template() -> String {
    "---\ncreated: <% tp.date.now(\"YYYY-MM-DD hh:mm\") %>\nmodified: \ndaily: \"[[<% tp.date.now(\"YYYY-MM-DD\") %>]]\"\ntags: inbox\ntype: inbox\n---".to_string()
}

fn default_image_width() -> String {
    "600".to_string()
}

fn default_window_transparency() -> u32 {
    10
}

fn default_window_blur() -> u32 {
    55
}

fn default_window_saturation() -> u32 {
    85
}

fn default_window_brightness() -> i32 {
    -85
}

fn default_reader_width() -> u32 {
    400
}

fn default_reader_height() -> u32 {
    800
}

fn default_text_color() -> String {
    "#ffffff".to_string()
}

fn default_accent_color() -> String {
    "#8b5cf6".to_string()
}

fn default_internal_link_color() -> String {
    "#a78bfa".to_string()
}

fn default_external_link_color() -> String {
    "#60a5fa".to_string()
}

fn default_capture_text_shortcut() -> String {
    "Cmd+Shift+C".to_string()
}

fn default_daily_note_folder() -> String {
    "Journal/".to_string()
}

fn default_daily_note_format() -> String {
    "YYYY-MM-DD".to_string()
}

fn is_safe_relative_path(path: &str) -> bool {
    let candidate = Path::new(path);
    !candidate.is_absolute()
        && candidate
            .components()
            .all(|component| matches!(component, Component::Normal(_) | Component::CurDir))
}

fn is_safe_filename_template(template: &str) -> bool {
    !template.trim().is_empty()
        && !template.contains('/')
        && !template.contains('\\')
        && !template.contains("..")
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            vault_name: "Vault".to_string(),
            vault_path: default_vault_path(),
            screenshot_path: default_screenshot_path(),
            edge_side: "right".to_string(),
            window_width: default_window_width(),
            window_height: default_window_height(),
            reader_width: default_reader_width(),
            reader_height: default_reader_height(),
            border_radius: 12,
            background_color: "#ffffff".to_string(),
            font_family: "-apple-system, BlinkMacSystemFont, SF Pro Display".to_string(),
            font_size: 14,
            daily_note_folder: default_daily_note_folder(),
            daily_note_format: default_daily_note_format(),
            daily_note_path: String::new(),
            image_folder: "assets/screenshots".to_string(),
            image_filename: "screenshot-YYYY-MM-DD-HHmmss".to_string(),
            default_image_width: default_image_width(),
            entry_header: "#### HH:mm".to_string(),
            global_shortcut: "Cmd+Shift+N".to_string(),
            global_shortcut_closes_window: default_false(),
            capture_text_shortcut: default_capture_text_shortcut(),
            compression_max_kb: 200,
            edge_detection_enabled: true,
            edge_reaction_time_ms: default_reaction_time_ms(),
            edge_modifier_keys: Vec::new(),
            edge_excluded_apps: Vec::new(),
            notes_folder: default_notes_folder(),
            save_to_daily_shortcut: default_save_to_daily_shortcut(),
            save_as_note_shortcut: default_save_as_note_shortcut(),
            pinned_notes: Vec::new(),
            reader_shortcut: default_reader_shortcut(),
            reader_shortcut_closes_window: default_false(),
            reader_edge_enabled: default_true(),
            reader_hide_frontmatter: default_true(),
            reader_hide_dataview: default_true(),
            reader_hide_obsidian_comments: default_true(),
            reader_hide_inline_fields: default_true(),
            reader_hide_html: default_true(),
            note_filename_template: default_note_filename_template(),
            note_template: default_note_template(),
            window_transparency: default_window_transparency(),
            window_blur: default_window_blur(),
            window_saturation: default_window_saturation(),
            window_brightness: default_window_brightness(),
            autostart_enabled: default_autostart_enabled(),
            text_color: default_text_color(),
            accent_color: default_accent_color(),
            internal_link_color: default_internal_link_color(),
            external_link_color: default_external_link_color(),
        }
    }
}

impl Settings {
    pub fn config_path() -> Result<PathBuf, String> {
        let config_dir =
            dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;
        let app_dir = config_dir.join("collector");
        Ok(app_dir.join("config.json"))
    }

    pub fn load() -> Result<Self, String> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)
                .map_err(|e| format!("Failed to read config file: {}", e))?;

            let mut settings =
                serde_json::from_str(&content).or_else(|e| -> Result<Settings, String> {
                    log::warn!("Config corrupted, using defaults: {}", e);
                    let defaults = Self::default();
                    if let Err(save_error) = defaults.save() {
                        log::warn!(
                            "Failed to persist default settings after config recovery: {}",
                            save_error
                        );
                    }
                    Ok(defaults)
                })?;

            // Migration: convert old daily_note_path to new fields.
            if !settings.daily_note_path.is_empty() && settings.daily_note_folder.is_empty() {
                let path = &settings.daily_note_path;

                if let Some(last_slash) = path.rfind('/') {
                    settings.daily_note_folder = path[..=last_slash].to_string();
                    let filename = &path[last_slash + 1..];

                    settings.daily_note_format =
                        filename.strip_suffix(".md").unwrap_or(filename).to_string();
                } else {
                    settings.daily_note_format =
                        path.strip_suffix(".md").unwrap_or(path).to_string();
                }

                log::info!(
                    "Migrated daily_note_path (folder={}, format_chars={})",
                    redact_path_str(&settings.daily_note_folder),
                    settings.daily_note_format.chars().count()
                );

                settings.daily_note_path = String::new();

                if let Err(save_error) = settings.save() {
                    log::warn!(
                        "Failed to persist migrated daily note settings: {}",
                        save_error
                    );
                }
            }

            Ok(settings)
        } else {
            let settings = Self::default();
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let config_path = Self::config_path()?;

        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&config_path, content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        if !config_path.exists() {
            return Err("Config file was not created after write operation".to_string());
        }

        let written_content = fs::read_to_string(&config_path)
            .map_err(|e| format!("Failed to read back config file for verification: {}", e))?;

        if !written_content.contains("\"window_blur\"") {
            log::warn!("window_blur field not found in written config file!");
        }

        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.edge_side != "left" && self.edge_side != "right" {
            return Err("edge_side must be 'left' or 'right'".to_string());
        }

        if self.window_width < 200 || self.window_width > 800 {
            return Err("window_width must be between 200 and 800".to_string());
        }
        if self.window_height < 200 || self.window_height > 1200 {
            return Err("window_height must be between 200 and 1200".to_string());
        }

        if self.reader_width < 200 || self.reader_width > 800 {
            return Err("reader_width must be between 200 and 800".to_string());
        }
        if self.reader_height < 200 || self.reader_height > 1200 {
            return Err("reader_height must be between 200 and 1200".to_string());
        }

        if self.edge_reaction_time_ms < 50 || self.edge_reaction_time_ms > 1000 {
            return Err("edge_reaction_time_ms must be between 50 and 1000".to_string());
        }

        if self.border_radius > 30 {
            return Err("border_radius must be between 0 and 30".to_string());
        }

        if self.font_size < 10 || self.font_size > 24 {
            return Err("font_size must be between 10 and 24".to_string());
        }

        if self.compression_max_kb < 50 || self.compression_max_kb > 2000 {
            return Err("compression_max_kb must be between 50 and 2000".to_string());
        }

        if !self.default_image_width.trim().is_empty() {
            let width = self
                .default_image_width
                .trim()
                .parse::<u32>()
                .map_err(|_| {
                    "default_image_width must be empty or a positive number".to_string()
                })?;
            if width == 0 {
                return Err("default_image_width must be empty or a positive number".to_string());
            }
        }

        if self.vault_name.trim().is_empty() {
            return Err("vault_name cannot be empty".to_string());
        }

        if !self.vault_path.is_empty() {
            let vault_path = PathBuf::from(&self.vault_path);
            if !vault_path.exists() {
                log::warn!("Vault path does not exist");
            }
        }

        if !is_safe_relative_path(&self.notes_folder) {
            return Err("notes_folder must stay inside the vault".to_string());
        }

        if !is_safe_relative_path(&self.daily_note_folder) {
            return Err("daily_note_folder must stay inside the vault".to_string());
        }

        if !is_safe_filename_template(&self.note_filename_template) {
            return Err(
                "note_filename_template must be a filename without path separators".to_string(),
            );
        }

        if !is_safe_filename_template(&self.image_filename) {
            return Err("image_filename must be a filename without path separators".to_string());
        }

        if !self.screenshot_path.is_empty() {
            let screenshot_path = PathBuf::from(&self.screenshot_path);
            let vault_path = PathBuf::from(&self.vault_path);
            let absolute_screenshot_path = if screenshot_path.is_absolute() {
                screenshot_path
            } else {
                vault_path.join(screenshot_path)
            };

            let normalized =
                absolute_screenshot_path
                    .components()
                    .fold(PathBuf::new(), |mut acc, component| {
                        match component {
                            Component::Prefix(prefix) => acc.push(prefix.as_os_str()),
                            Component::RootDir => acc.push(component.as_os_str()),
                            Component::CurDir => {}
                            Component::Normal(part) => acc.push(part),
                            Component::ParentDir => {
                                acc.pop();
                            }
                        }
                        acc
                    });

            if !normalized.starts_with(&vault_path) {
                return Err("screenshot_path must stay inside the vault".to_string());
            }
        }

        if !self.global_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.global_shortcut)?;
        }

        if !self.capture_text_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.capture_text_shortcut)?;
        }

        if !self.save_as_note_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.save_as_note_shortcut)?;
        }

        if !self.reader_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_shortcut)?;
        }

        for pinned_note in &self.pinned_notes {
            if pinned_note.path.trim().is_empty() {
                return Err("pinned_notes entries must include a path".to_string());
            }
        }

        if self.window_transparency > 100 {
            return Err("window_transparency must be between 0 and 100".to_string());
        }

        if self.window_blur > 200 {
            return Err("window_blur must be between 0 and 200".to_string());
        }

        if self.window_saturation > 300 {
            return Err("window_saturation must be between 0 and 300".to_string());
        }

        if self.window_brightness < -100 || self.window_brightness > 100 {
            return Err("window_brightness must be between -100 and 100".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_notes_folder_escape() {
        let settings = Settings {
            vault_path: "/tmp/vault".to_string(),
            notes_folder: "../outside".to_string(),
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_image_filename_with_separator() {
        let settings = Settings {
            vault_path: "/tmp/vault".to_string(),
            image_filename: "nested/file".to_string(),
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }
}
