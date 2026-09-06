use std::fs;
use std::path::PathBuf;

use crate::fs_util::atomic_write_text;

use super::Settings;

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

            let (settings, needs_save) = Self::migrate_from_content(&content).or_else(
                |e| -> Result<(Settings, bool), String> {
                    log::warn!("Config corrupted, using defaults: {}", e);
                    let defaults = Self::default();
                    if let Err(save_error) = defaults.save() {
                        log::warn!(
                            "Failed to persist default settings after config recovery: {}",
                            save_error
                        );
                    }
                    Ok((defaults, false))
                },
            )?;

            if needs_save {
                if let Err(save_error) = settings.save() {
                    log::warn!("Failed to persist migrated settings: {}", save_error);
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

        atomic_write_text(&config_path, &content)
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
}
