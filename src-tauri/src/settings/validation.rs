use std::path::{Component, Path, PathBuf};

use super::Settings;

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

impl Settings {
    pub fn validate(&self) -> Result<(), String> {
        if self.edge_side != "left" && self.edge_side != "right" {
            return Err("edge_side must be 'left' or 'right'".to_string());
        }

        if !matches!(self.ocr_drag_modifier.as_str(), "Option" | "Shift" | "Control" | "Command") {
            return Err("ocr_drag_modifier must be 'Option', 'Shift', 'Control', or 'Command'".to_string());
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

        if self.note_edge_open_delay_ms < 50 || self.note_edge_open_delay_ms > 10000 {
            return Err("note_edge_open_delay_ms must be between 50 and 10000".to_string());
        }

        if self.reader_edge_open_delay_ms < 50 || self.reader_edge_open_delay_ms > 10000 {
            return Err("reader_edge_open_delay_ms must be between 50 and 10000".to_string());
        }

        if self.reader_edge_side != "left" && self.reader_edge_side != "right" {
            return Err("reader_edge_side must be 'left' or 'right'".to_string());
        }

        if self.reader_edge_side == self.edge_side
            && self.reader_edge_enabled
            && self.edge_detection_enabled
        {
            log::warn!(
                "Reader and Capture edge detection use the same screen edge ({})",
                self.edge_side
            );
        }

        if self.border_radius > 30 {
            return Err("border_radius must be between 0 and 30".to_string());
        }

        if self.font_size < 10 || self.font_size > 24 {
            return Err("font_size must be between 10 and 24".to_string());
        }

        if self.autocomplete_results < 5 || self.autocomplete_results > 50 {
            return Err("autocomplete_results must be between 5 and 50".to_string());
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

        if !is_safe_filename_template(&self.daily_note_format) {
            return Err("daily_note_format must be a filename without path separators".to_string());
        }

        if !is_safe_filename_template(&self.note_filename_template) {
            return Err(
                "note_filename_template must be a filename without path separators".to_string(),
            );
        }

        if !is_safe_filename_template(&self.image_filename) {
            return Err("image_filename must be a filename without path separators".to_string());
        }

        if !self.screenshot_path.is_empty() && !is_safe_relative_path(&self.screenshot_path) {
            return Err("screenshot_path must stay inside the vault".to_string());
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

        if !self.capture_mode_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.capture_mode_shortcut)?;
        }

        if !self.global_close_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.global_close_shortcut)?;
        }

        if !self.save_as_note_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.save_as_note_shortcut)?;
        }

        if !self.reader_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_shortcut)?;
        }

        if !self.reader_close_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_close_shortcut)?;
        }

        if !self.reader_open_in_obsidian_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_open_in_obsidian_shortcut)?;
        }

        if !self.reader_navigate_back_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_navigate_back_shortcut)?;
        }

        if !self.reader_command_palette_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_command_palette_shortcut)?;
        }

        if !self.reader_next_tab_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_next_tab_shortcut)?;
        }

        if !self.reader_previous_tab_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.reader_previous_tab_shortcut)?;
        }

        if !self.save_to_daily_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.save_to_daily_shortcut)?;
        }

        if !self.append_to_note_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.append_to_note_shortcut)?;
        }

        if !self.ocr_shortcut.trim().is_empty() {
            crate::shortcuts::validate_shortcut(&self.ocr_shortcut)?;
        }

        if !self.global_shortcut_closes_window
            && !self.global_shortcut.trim().is_empty()
            && !self.global_close_shortcut.trim().is_empty()
            && crate::shortcuts::normalize_shortcut(&self.global_shortcut)
                == crate::shortcuts::normalize_shortcut(&self.global_close_shortcut)
        {
            return Err(
                "note window open and close shortcuts must be different unless one shortcut is used for both".to_string(),
            );
        }

        if !self.reader_shortcut_closes_window
            && !self.reader_shortcut.trim().is_empty()
            && !self.reader_close_shortcut.trim().is_empty()
            && crate::shortcuts::normalize_shortcut(&self.reader_shortcut)
                == crate::shortcuts::normalize_shortcut(&self.reader_close_shortcut)
        {
            return Err(
                "reader open and close shortcuts must be different unless one shortcut is used for both".to_string(),
            );
        }

        // Cross-slot shortcut conflict check
        let shortcut_fields: [(&str, &str); 8] = [
            ("Note window shortcut", &self.global_shortcut),
            ("Note close shortcut", &self.global_close_shortcut),
            ("Reader shortcut", &self.reader_shortcut),
            ("Reader close shortcut", &self.reader_close_shortcut),
            ("Capture text shortcut", &self.capture_text_shortcut),
            ("Capture mode shortcut", &self.capture_mode_shortcut),
            ("Save as note shortcut", &self.save_as_note_shortcut),
            ("OCR shortcut", &self.ocr_shortcut),
        ];

        for i in 0..shortcut_fields.len() {
            let trimmed_i = shortcut_fields[i].1.trim();
            if trimmed_i.is_empty() {
                continue;
            }
            let norm_i = crate::shortcuts::normalize_shortcut(trimmed_i);

            for j in (i + 1)..shortcut_fields.len() {
                let trimmed_j = shortcut_fields[j].1.trim();
                if trimmed_j.is_empty() {
                    continue;
                }

                // Skip open-vs-close pairs for the same feature (handled by existing checks above)
                if (i == 0 && j == 1) || (i == 2 && j == 3) {
                    continue;
                }

                let norm_j = crate::shortcuts::normalize_shortcut(trimmed_j);
                if norm_i == norm_j {
                    return Err(format!(
                        "Shortcut conflict: {} and {} cannot use the same key combination",
                        shortcut_fields[i].0, shortcut_fields[j].0,
                    ));
                }
            }
        }

        for pinned_note in &self.pinned_notes {
            if pinned_note.path.trim().is_empty() {
                return Err("pinned_notes entries must include a path".to_string());
            }

            if !is_safe_relative_path(&pinned_note.path) {
                return Err("pinned_notes entries must stay inside the vault".to_string());
            }
        }

        if self.overlay_strength > 100 {
            return Err("overlay_strength must be between 0 and 100".to_string());
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

        if self.daily_note_insert_position != "top" && self.daily_note_insert_position != "bottom" {
            return Err("daily_note_insert_position must be 'top' or 'bottom'".to_string());
        }

        if self.daily_note_create_timeout_ms < 1000 || self.daily_note_create_timeout_ms > 60000 {
            return Err("daily_note_create_timeout_ms must be between 1000 and 60000".to_string());
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

    #[test]
    fn rejects_daily_note_format_with_separator() {
        let settings = Settings {
            vault_path: "/tmp/vault".to_string(),
            daily_note_format: "../outside".to_string(),
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_note_edge_delay_below_minimum() {
        let settings = Settings {
            note_edge_open_delay_ms: 25,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_reader_edge_delay_above_maximum() {
        let settings = Settings {
            reader_edge_open_delay_ms: 20000,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_autocomplete_results_below_minimum() {
        let settings = Settings {
            autocomplete_results: 4,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_autocomplete_results_above_maximum() {
        let settings = Settings {
            autocomplete_results: 51,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_note_window_open_and_close_shortcuts_without_toggle() {
        let settings = Settings {
            global_shortcut: "Cmd+Shift+N".to_string(),
            global_close_shortcut: "CommandOrControl+Shift+N".to_string(),
            global_shortcut_closes_window: false,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_duplicate_reader_open_and_close_shortcuts_without_toggle() {
        let settings = Settings {
            reader_shortcut: "Cmd+Shift+R".to_string(),
            reader_close_shortcut: "CommandOrControl+Shift+R".to_string(),
            reader_shortcut_closes_window: false,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn accepts_empty_reader_action_shortcuts() {
        let settings = Settings {
            reader_open_in_obsidian_shortcut: String::new(),
            reader_navigate_back_shortcut: String::new(),
            reader_command_palette_shortcut: String::new(),
            reader_next_tab_shortcut: String::new(),
            reader_previous_tab_shortcut: String::new(),
            ..Default::default()
        };

        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_reader_action_shortcuts() {
        let cases = [
            Settings {
                reader_open_in_obsidian_shortcut: "O".to_string(),
                ..Default::default()
            },
            Settings {
                reader_navigate_back_shortcut: "Cmd".to_string(),
                ..Default::default()
            },
            Settings {
                reader_command_palette_shortcut: "Cmd+Plus".to_string(),
                ..Default::default()
            },
            Settings {
                reader_next_tab_shortcut: "Cmd".to_string(),
                ..Default::default()
            },
            Settings {
                reader_previous_tab_shortcut: "Cmd+Plus".to_string(),
                ..Default::default()
            },
        ];

        for settings in cases {
            assert!(settings.validate().is_err());
        }
    }

    #[test]
    fn rejects_invalid_daily_note_insert_position() {
        let settings = Settings {
            daily_note_insert_position: "left".to_string(),
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn accepts_valid_daily_note_insert_positions() {
        let top = Settings {
            daily_note_insert_position: "top".to_string(),
            ..Default::default()
        };
        assert!(top.validate().is_ok());

        let bottom = Settings {
            daily_note_insert_position: "bottom".to_string(),
            ..Default::default()
        };
        assert!(bottom.validate().is_ok());
    }

    #[test]
    fn rejects_daily_note_create_timeout_below_minimum() {
        let settings = Settings {
            daily_note_create_timeout_ms: 500,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn rejects_daily_note_create_timeout_above_maximum() {
        let settings = Settings {
            daily_note_create_timeout_ms: 61000,
            ..Default::default()
        };

        assert!(settings.validate().is_err());
    }

    #[test]
    fn accepts_default_daily_note_settings() {
        let settings = Settings::default();
        // default daily_note_insert_position = "bottom"
        assert_eq!(settings.daily_note_insert_position, "bottom");
        // default daily_note_create_timeout_ms = 15000
        assert_eq!(settings.daily_note_create_timeout_ms, 15000);
        // default daily_note_create_if_missing = false
        assert!(!settings.daily_note_create_if_missing);
        // default daily_note_create_heading_if_missing = false
        assert!(!settings.daily_note_create_heading_if_missing);
        // default show_capture_action_bar = true
        assert!(settings.show_capture_action_bar);
        // validation must pass with defaults
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_ocr_drag_modifier() {
        let mut settings = Settings::default();
        settings.ocr_drag_modifier = "None".to_string();
        assert!(settings.validate().is_err());

        settings.ocr_drag_modifier = "Alt".to_string();
        assert!(settings.validate().is_err());

        settings.ocr_drag_modifier = "Option".to_string();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_ocr_shortcut_conflicts_with_global_shortcuts() {
        let mut settings = Settings::default();
        // global_shortcut is Cmd+Shift+N
        settings.ocr_shortcut = "Cmd+Shift+N".to_string();
        assert!(settings.validate().is_err());

        // capture_text_shortcut is Cmd+Shift+C
        settings.ocr_shortcut = "Cmd+Shift+C".to_string();
        assert!(settings.validate().is_err());

        // Unique shortcut passes
        settings.ocr_shortcut = "Cmd+Shift+T".to_string();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_capture_mode_shortcut_conflicts_with_global_shortcuts() {
        let mut settings = Settings::default();
        // global_shortcut is Cmd+Shift+N
        settings.capture_mode_shortcut = "Cmd+Shift+N".to_string();
        assert!(settings.validate().is_err());

        // capture_text_shortcut is Cmd+Shift+C
        settings.capture_mode_shortcut = "Cmd+Shift+C".to_string();
        assert!(settings.validate().is_err());

        // Unique shortcut passes
        settings.capture_mode_shortcut = "Cmd+Shift+M".to_string();
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn rejects_invalid_capture_mode_shortcut_syntax() {
        let mut settings = Settings::default();
        settings.capture_mode_shortcut = "NotAShortcut".to_string();
        assert!(settings.validate().is_err());
    }
}
