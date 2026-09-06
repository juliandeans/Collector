use serde::{Deserialize, Deserializer, Serialize};

mod defaults;
use self::defaults::*;

mod validation;
mod migration;
mod persistence;

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
    #[serde(alias = "background_color")]
    pub overlay_color: String,
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
    #[serde(default)]
    pub daily_note_target_heading: String,
    #[serde(default = "default_daily_note_insert_position")]
    pub daily_note_insert_position: String,
    #[serde(default = "default_false")]
    pub daily_note_create_heading_if_missing: bool,
    #[serde(default = "default_false")]
    pub daily_note_create_if_missing: bool,
    #[serde(default = "default_daily_note_create_timeout_ms")]
    pub daily_note_create_timeout_ms: u64,
    #[serde(default = "default_true")]
    pub show_capture_action_bar: bool,
    #[serde(default = "default_true")]
    pub show_note_paths: bool,
    #[serde(default = "default_autocomplete_results")]
    pub autocomplete_results: u32,
    pub global_shortcut: String,
    #[serde(default = "default_false")]
    pub global_shortcut_closes_window: bool,
    #[serde(default)]
    pub global_close_shortcut: String,
    #[serde(default = "default_capture_text_shortcut")]
    pub capture_text_shortcut: String,
    pub compression_max_kb: u32,
    #[serde(default = "default_edge_enabled")]
    pub edge_detection_enabled: bool,
    #[serde(default = "default_reaction_time_ms")]
    pub edge_reaction_time_ms: u64,
    #[serde(default = "default_false")]
    pub note_edge_open_delay_enabled: bool,
    #[serde(default = "default_edge_open_delay_ms")]
    pub note_edge_open_delay_ms: u64,
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
    #[serde(default = "default_append_to_note_shortcut")]
    pub append_to_note_shortcut: String,
    #[serde(default, deserialize_with = "deserialize_pinned_notes")]
    pub pinned_notes: Vec<PinnedNote>,
    #[serde(default = "default_reader_shortcut")]
    pub reader_shortcut: String,
    #[serde(default = "default_false")]
    pub reader_shortcut_closes_window: bool,
    #[serde(default)]
    pub reader_close_shortcut: String,
    #[serde(default = "default_reader_open_in_obsidian_shortcut")]
    pub reader_open_in_obsidian_shortcut: String,
    #[serde(default)]
    pub reader_navigate_back_shortcut: String,
    #[serde(default = "default_reader_command_palette_shortcut")]
    pub reader_command_palette_shortcut: String,
    #[serde(default = "default_true")]
    pub reader_edge_enabled: bool,
    #[serde(default = "default_false")]
    pub reader_edge_open_delay_enabled: bool,
    #[serde(default = "default_edge_open_delay_ms")]
    pub reader_edge_open_delay_ms: u64,
    #[serde(default = "default_reader_edge_side")]
    pub reader_edge_side: String,
    #[serde(default)]
    pub reader_edge_modifier_keys: Vec<String>,
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
    #[serde(default = "default_true")]
    pub reader_hide_callouts: bool,
    #[serde(default = "default_note_filename_template")]
    pub note_filename_template: String,
    #[serde(default = "default_note_template")]
    pub note_template: String,
    #[serde(default = "default_overlay_strength")]
    #[serde(alias = "window_transparency")]
    pub overlay_strength: u32,
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
            overlay_color: "#ffffff".to_string(),
            font_family: "-apple-system, BlinkMacSystemFont, SF Pro Display".to_string(),
            font_size: 14,
            daily_note_folder: default_daily_note_folder(),
            daily_note_format: default_daily_note_format(),
            daily_note_path: String::new(),
            image_folder: default_screenshot_path(),
            image_filename: "screenshot-YYYY-MM-DD-HHmmss".to_string(),
            default_image_width: default_image_width(),
            entry_header: "#### HH:mm".to_string(),
            daily_note_target_heading: String::new(),
            daily_note_insert_position: default_daily_note_insert_position(),
            daily_note_create_heading_if_missing: default_false(),
            daily_note_create_if_missing: default_false(),
            daily_note_create_timeout_ms: default_daily_note_create_timeout_ms(),
            show_capture_action_bar: default_true(),
            show_note_paths: default_true(),
            autocomplete_results: default_autocomplete_results(),
            global_shortcut: "Cmd+Shift+N".to_string(),
            global_shortcut_closes_window: default_false(),
            global_close_shortcut: String::new(),
            capture_text_shortcut: default_capture_text_shortcut(),
            compression_max_kb: 200,
            edge_detection_enabled: true,
            edge_reaction_time_ms: default_reaction_time_ms(),
            note_edge_open_delay_enabled: default_false(),
            note_edge_open_delay_ms: default_edge_open_delay_ms(),
            edge_modifier_keys: Vec::new(),
            edge_excluded_apps: Vec::new(),
            notes_folder: default_notes_folder(),
            save_to_daily_shortcut: default_save_to_daily_shortcut(),
            save_as_note_shortcut: default_save_as_note_shortcut(),
            append_to_note_shortcut: default_append_to_note_shortcut(),
            pinned_notes: Vec::new(),
            reader_shortcut: default_reader_shortcut(),
            reader_shortcut_closes_window: default_false(),
            reader_close_shortcut: String::new(),
            reader_open_in_obsidian_shortcut: default_reader_open_in_obsidian_shortcut(),
            reader_navigate_back_shortcut: String::new(),
            reader_command_palette_shortcut: default_reader_command_palette_shortcut(),
            reader_edge_enabled: default_true(),
            reader_edge_open_delay_enabled: default_false(),
            reader_edge_open_delay_ms: default_edge_open_delay_ms(),
            reader_edge_side: "left".to_string(),
            reader_edge_modifier_keys: Vec::new(),
            reader_hide_frontmatter: default_true(),
            reader_hide_dataview: default_true(),
            reader_hide_obsidian_comments: default_true(),
            reader_hide_inline_fields: default_true(),
            reader_hide_html: default_true(),
            reader_hide_callouts: default_true(),
            note_filename_template: default_note_filename_template(),
            note_template: default_note_template(),
            overlay_strength: default_overlay_strength(),
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


