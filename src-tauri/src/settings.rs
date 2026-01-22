use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::log_safety::{redact_path, redact_path_str, summarize_bytes};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub vault_name: String,

    #[serde(default = "default_vault_path")]
    pub vault_path: String,

    #[serde(default = "default_screenshot_path")]
    pub screenshot_path: String,

    pub edge_side: String,
    pub window_width: u32,
    pub window_height: u32,
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
    #[serde(default = "default_capture_text_shortcut")]
    pub capture_text_shortcut: String,
    pub compression_max_kb: u32,
    #[serde(default = "default_edge_enabled")]
    pub edge_detection_enabled: bool,
    #[serde(default = "default_notes_folder")]
    pub notes_folder: String,
    #[serde(default = "default_save_to_daily_shortcut")]
    pub save_to_daily_shortcut: String,
    #[serde(default = "default_save_as_note_shortcut")]
    pub save_as_note_shortcut: String,
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
}

fn default_autostart_enabled() -> bool {
    false
}
fn default_edge_enabled() -> bool {
    true
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
    55
}

fn default_window_blur() -> u32 {
    80
}

fn default_window_saturation() -> u32 {
    200
}

fn default_window_brightness() -> i32 {
    0
}

fn default_text_color() -> String {
    "#ffffff".to_string()
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

impl Default for Settings {
    fn default() -> Self {
        Self {
            vault_name: "Vault".to_string(),
            vault_path: default_vault_path(),
            screenshot_path: default_screenshot_path(),
            edge_side: "right".to_string(),
            window_width: 330,
            window_height: 600,
            border_radius: 12,
            background_color: "#1e1e2e".to_string(),
            font_family: "-apple-system, BlinkMacSystemFont, SF Pro Display".to_string(),
            font_size: 15,
            daily_note_folder: default_daily_note_folder(),
            daily_note_format: default_daily_note_format(),
            daily_note_path: String::new(),
            image_folder: "assets/screenshots".to_string(),
            image_filename: "screenshot-YYYY-MM-DD-HHmmss".to_string(),
            default_image_width: default_image_width(),
            entry_header: "#### HH:mm".to_string(),
            global_shortcut: "Cmd+Shift+N".to_string(),
            capture_text_shortcut: default_capture_text_shortcut(),
            compression_max_kb: 200,
            edge_detection_enabled: true,
            notes_folder: default_notes_folder(),
            save_to_daily_shortcut: default_save_to_daily_shortcut(),
            save_as_note_shortcut: default_save_as_note_shortcut(),
            note_filename_template: default_note_filename_template(),
            note_template: default_note_template(),
            window_transparency: default_window_transparency(),
            window_blur: default_window_blur(),
            window_saturation: default_window_saturation(),
            window_brightness: default_window_brightness(),
            autostart_enabled: default_autostart_enabled(),
            text_color: default_text_color(),
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

            let mut settings = serde_json::from_str(&content).or_else(|e| -> Result<Settings, String> {
                log::warn!("Config corrupted, using defaults: {}", e);
                let defaults = Self::default();
                let _ = defaults.save();
                Ok(defaults)
            })?;

            // Migration: convert old daily_note_path to new fields.
            if !settings.daily_note_path.is_empty() && settings.daily_note_folder.is_empty() {
                let path = &settings.daily_note_path;

                if let Some(last_slash) = path.rfind('/') {
                    settings.daily_note_folder = path[..=last_slash].to_string();
                    let filename = &path[last_slash + 1..];

                    settings.daily_note_format = filename
                        .strip_suffix(".md")
                        .unwrap_or(filename)
                        .to_string();
                } else {
                    settings.daily_note_format = path
                        .strip_suffix(".md")
                        .unwrap_or(path)
                        .to_string();
                }

                log::info!(
                    "Migrated daily_note_path (folder={}, format_chars={})",
                    redact_path_str(&settings.daily_note_folder),
                    settings.daily_note_format.chars().count()
                );

                settings.daily_note_path = String::new();

                let _ = settings.save();
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

        log::info!(
            "Writing settings (file={}, size={})",
            redact_path(&config_path),
            summarize_bytes(content.len())
        );
        log::info!("window_blur in serialized content: {}", self.window_blur);

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

        log::info!(
            "Settings saved (file={}, verified)",
            redact_path(&config_path)
        );
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.edge_side != "left" && self.edge_side != "right" {
            return Err("edge_side must be 'left' or 'right'".to_string());
        }

        if self.window_width < 200 || self.window_width > 800 {
            return Err("window_width must be between 200 and 800".to_string());
        }
        if self.window_height < 80 || self.window_height > 1200 {
            return Err("window_height must be between 80 and 1200".to_string());
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
                .map_err(|_| "default_image_width must be empty or a positive number".to_string())?;
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

        if !self.screenshot_path.is_empty() {
            let screenshot_path = PathBuf::from(&self.screenshot_path);
            if let Some(parent) = screenshot_path.parent() {
                if !parent.exists() {
                    log::warn!("Screenshot path parent directory does not exist");
                }
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
