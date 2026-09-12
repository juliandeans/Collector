use serde::{Deserialize, Deserializer, Serialize};

mod defaults;
use self::defaults::*;

mod migration;
mod persistence;
mod validation;

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
#[serde(tag = "type", rename_all = "snake_case")]
enum CaptureRuleConditionKnown {
    Contains {
        value: String,
        #[serde(rename = "caseSensitive")]
        case_sensitive: bool,
    },
    Hashtag {
        value: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CaptureRuleConditionInput {
    Known(CaptureRuleConditionKnown),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureRuleCondition {
    Contains { value: String, case_sensitive: bool },
    Hashtag { value: String },
    Unknown(serde_json::Value),
}

impl From<CaptureRuleConditionKnown> for CaptureRuleCondition {
    fn from(value: CaptureRuleConditionKnown) -> Self {
        match value {
            CaptureRuleConditionKnown::Contains {
                value,
                case_sensitive,
            } => Self::Contains {
                value,
                case_sensitive,
            },
            CaptureRuleConditionKnown::Hashtag { value } => Self::Hashtag { value },
        }
    }
}

impl Serialize for CaptureRuleCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Contains {
                value,
                case_sensitive,
            } => CaptureRuleConditionKnown::Contains {
                value: value.clone(),
                case_sensitive: *case_sensitive,
            }
            .serialize(serializer),
            Self::Hashtag { value } => CaptureRuleConditionKnown::Hashtag {
                value: value.clone(),
            }
            .serialize(serializer),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for CaptureRuleCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match CaptureRuleConditionInput::deserialize(deserializer)? {
            CaptureRuleConditionInput::Known(value) => Ok(value.into()),
            CaptureRuleConditionInput::Unknown(value) => Ok(Self::Unknown(value)),
        }
    }
}

fn deserialize_capture_rule_conditions<'de, D>(
    deserializer: D,
) -> Result<Vec<CaptureRuleCondition>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries = Vec::<CaptureRuleConditionInput>::deserialize(deserializer)?;

    Ok(entries
        .into_iter()
        .map(|entry| match entry {
            CaptureRuleConditionInput::Known(value) => value.into(),
            CaptureRuleConditionInput::Unknown(value) => CaptureRuleCondition::Unknown(value),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CaptureRuleActionKnown {
    Append {
        note: String,
        #[serde(default)]
        heading: String,
        #[serde(default = "default_rule_position")]
        position: String,
        #[serde(default, rename = "createHeadingIfMissing")]
        create_heading_if_missing: bool,
        #[serde(default = "default_true", rename = "stripTrigger")]
        strip_trigger: bool,
        #[serde(default = "default_true", rename = "showTimestamp")]
        show_timestamp: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CaptureRuleActionInput {
    Known(CaptureRuleActionKnown),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureRuleAction {
    Append {
        note: String,
        heading: String,
        position: String,
        create_heading_if_missing: bool,
        strip_trigger: bool,
        show_timestamp: bool,
    },
    Unknown(serde_json::Value),
}

impl From<CaptureRuleActionKnown> for CaptureRuleAction {
    fn from(value: CaptureRuleActionKnown) -> Self {
        match value {
            CaptureRuleActionKnown::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            } => Self::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            },
        }
    }
}

impl Serialize for CaptureRuleAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            } => CaptureRuleActionKnown::Append {
                note: note.clone(),
                heading: heading.clone(),
                position: position.clone(),
                create_heading_if_missing: *create_heading_if_missing,
                strip_trigger: *strip_trigger,
                show_timestamp: *show_timestamp,
            }
            .serialize(serializer),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for CaptureRuleAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match CaptureRuleActionInput::deserialize(deserializer)? {
            CaptureRuleActionInput::Known(value) => Ok(value.into()),
            CaptureRuleActionInput::Unknown(value) => Ok(Self::Unknown(value)),
        }
    }
}

fn deserialize_capture_rule_actions<'de, D>(
    deserializer: D,
) -> Result<Vec<CaptureRuleAction>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries = Vec::<CaptureRuleActionInput>::deserialize(deserializer)?;

    Ok(entries
        .into_iter()
        .map(|entry| match entry {
            CaptureRuleActionInput::Known(value) => value.into(),
            CaptureRuleActionInput::Unknown(value) => CaptureRuleAction::Unknown(value),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRule {
    // id/name brauchen serde(default): ein fehlendes Feld darf nie die
    // komplette Settings-Deserialisierung killen — persistence.rs schreibt
    // sonst bei einem Parse-Fehler Defaults ueber die gesamte config.json.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, deserialize_with = "deserialize_capture_rule_conditions")]
    pub conditions: Vec<CaptureRuleCondition>,
    #[serde(rename = "match", default = "default_rule_match")]
    pub match_mode: String,
    #[serde(default, deserialize_with = "deserialize_capture_rule_actions")]
    pub actions: Vec<CaptureRuleAction>,
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
    pub show_capture_mode_button: bool,
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
    #[serde(default = "default_capture_mode_shortcut")]
    pub capture_mode_shortcut: String,
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
    #[serde(default)]
    pub capture_rules: Vec<CaptureRule>,
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
    #[serde(default = "default_reader_next_tab_shortcut")]
    pub reader_next_tab_shortcut: String,
    #[serde(default = "default_reader_previous_tab_shortcut")]
    pub reader_previous_tab_shortcut: String,
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
    #[serde(default = "default_false")]
    pub round_all_corners: bool,
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
    #[serde(default)]
    pub remember_window_geometry: bool,
    #[serde(default)]
    pub remembered_window_x: Option<i32>,
    #[serde(default)]
    pub remembered_window_y: Option<i32>,
    #[serde(default)]
    pub remembered_window_width: Option<u32>,
    #[serde(default)]
    pub remembered_window_height: Option<u32>,
    #[serde(default)]
    pub remember_reader_geometry: bool,
    #[serde(default = "default_true")]
    pub reader_show_daily_note: bool,
    #[serde(default)]
    pub remembered_reader_x: Option<i32>,
    #[serde(default)]
    pub remembered_reader_y: Option<i32>,
    #[serde(default)]
    pub remembered_reader_width: Option<u32>,
    #[serde(default)]
    pub remembered_reader_height: Option<u32>,
    #[serde(default)]
    pub persist_capture_draft: bool,
    #[serde(default = "default_persist_capture_draft_shortcut")]
    pub persist_capture_draft_shortcut: String,
    #[serde(default = "default_clear_capture_shortcut")]
    pub clear_capture_shortcut: String,
    #[serde(default = "default_ocr_shortcut")]
    pub ocr_shortcut: String,
    #[serde(default = "default_ocr_drag_modifier")]
    pub ocr_drag_modifier: String,
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
            show_capture_mode_button: default_true(),
            show_note_paths: default_true(),
            autocomplete_results: default_autocomplete_results(),
            global_shortcut: "Cmd+Shift+N".to_string(),
            global_shortcut_closes_window: default_false(),
            global_close_shortcut: String::new(),
            capture_text_shortcut: default_capture_text_shortcut(),
            capture_mode_shortcut: default_capture_mode_shortcut(),
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
            capture_rules: Vec::new(),
            pinned_notes: Vec::new(),
            reader_shortcut: default_reader_shortcut(),
            reader_shortcut_closes_window: default_false(),
            reader_close_shortcut: String::new(),
            reader_open_in_obsidian_shortcut: default_reader_open_in_obsidian_shortcut(),
            reader_navigate_back_shortcut: String::new(),
            reader_command_palette_shortcut: default_reader_command_palette_shortcut(),
            reader_next_tab_shortcut: default_reader_next_tab_shortcut(),
            reader_previous_tab_shortcut: default_reader_previous_tab_shortcut(),
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
            round_all_corners: false,
            autostart_enabled: default_autostart_enabled(),
            text_color: default_text_color(),
            accent_color: default_accent_color(),
            internal_link_color: default_internal_link_color(),
            external_link_color: default_external_link_color(),
            remember_window_geometry: false,
            remembered_window_x: None,
            remembered_window_y: None,
            remembered_window_width: None,
            remembered_window_height: None,
            remember_reader_geometry: false,
            reader_show_daily_note: true,
            remembered_reader_x: None,
            remembered_reader_y: None,
            remembered_reader_width: None,
            remembered_reader_height: None,
            persist_capture_draft: false,
            persist_capture_draft_shortcut: default_persist_capture_draft_shortcut(),
            clear_capture_shortcut: default_clear_capture_shortcut(),
            ocr_shortcut: default_ocr_shortcut(),
            ocr_drag_modifier: default_ocr_drag_modifier(),
        }
    }
}

#[cfg(test)]
mod new_field_tests {
    use super::Settings;

    #[test]
    fn defaults_have_geometry_memory_and_draft_persistence_off() {
        let settings = Settings::default();
        assert!(!settings.remember_window_geometry);
        assert_eq!(settings.remembered_window_x, None);
        assert_eq!(settings.remembered_window_y, None);
        assert_eq!(settings.remembered_window_width, None);
        assert_eq!(settings.remembered_window_height, None);
        assert!(!settings.remember_reader_geometry);
        assert!(settings.reader_show_daily_note);
        assert_eq!(settings.remembered_reader_x, None);
        assert_eq!(settings.remembered_reader_y, None);
        assert_eq!(settings.remembered_reader_width, None);
        assert_eq!(settings.remembered_reader_height, None);
        assert!(!settings.persist_capture_draft);
        assert_eq!(settings.persist_capture_draft_shortcut, "Cmd+Shift+D");
        assert_eq!(settings.clear_capture_shortcut, "Cmd+Backspace");
    }

    #[test]
    fn old_config_without_new_fields_still_deserializes() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("remember_window_geometry");
        object.remove("remembered_window_x");
        object.remove("remembered_window_y");
        object.remove("remembered_window_width");
        object.remove("remembered_window_height");
        object.remove("remember_reader_geometry");
        object.remove("reader_show_daily_note");
        object.remove("remembered_reader_x");
        object.remove("remembered_reader_y");
        object.remove("remembered_reader_width");
        object.remove("remembered_reader_height");
        object.remove("persist_capture_draft");
        object.remove("persist_capture_draft_shortcut");
        object.remove("clear_capture_shortcut");

        let content = serde_json::to_string(&value).unwrap();
        let settings: Settings = serde_json::from_str(&content)
            .expect("old config without new fields must still deserialize");

        assert!(!settings.remember_window_geometry);
        assert!(!settings.remember_reader_geometry);
        assert!(settings.reader_show_daily_note);
        assert!(!settings.persist_capture_draft);
        assert_eq!(settings.persist_capture_draft_shortcut, "Cmd+Shift+D");
        assert_eq!(settings.clear_capture_shortcut, "Cmd+Backspace");
    }

    #[test]
    fn capture_mode_defaults_are_button_shown_and_cmd_shift_m() {
        let settings = Settings::default();
        assert!(settings.show_capture_mode_button);
        assert_eq!(settings.capture_mode_shortcut, "Cmd+Shift+M");
    }

    #[test]
    fn old_config_without_capture_mode_fields_still_deserializes() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("show_capture_mode_button");
        object.remove("capture_mode_shortcut");

        let content = serde_json::to_string(&value).unwrap();
        let settings: Settings = serde_json::from_str(&content)
            .expect("old config without capture mode fields must still deserialize");

        assert!(settings.show_capture_mode_button);
        assert_eq!(settings.capture_mode_shortcut, "Cmd+Shift+M");
    }

    #[test]
    fn remembered_geometry_roundtrips_through_json() {
        let mut settings = Settings::default();
        settings.remember_window_geometry = true;
        settings.remembered_window_x = Some(120);
        settings.remembered_window_y = Some(80);
        settings.remembered_window_width = Some(400);
        settings.remembered_window_height = Some(650);
        settings.remember_reader_geometry = true;
        settings.remembered_reader_x = Some(220);
        settings.remembered_reader_y = Some(90);
        settings.remembered_reader_width = Some(500);
        settings.remembered_reader_height = Some(750);

        let content = serde_json::to_string(&settings).unwrap();
        let round_tripped: Settings = serde_json::from_str(&content).unwrap();

        assert!(round_tripped.remember_window_geometry);
        assert_eq!(round_tripped.remembered_window_x, Some(120));
        assert_eq!(round_tripped.remembered_window_y, Some(80));
        assert_eq!(round_tripped.remembered_window_width, Some(400));
        assert_eq!(round_tripped.remembered_window_height, Some(650));
        assert!(round_tripped.remember_reader_geometry);
        assert_eq!(round_tripped.remembered_reader_x, Some(220));
        assert_eq!(round_tripped.remembered_reader_y, Some(90));
        assert_eq!(round_tripped.remembered_reader_width, Some(500));
        assert_eq!(round_tripped.remembered_reader_height, Some(750));
    }
}

#[cfg(test)]
mod capture_rules_tests {
    use super::*;

    #[test]
    fn old_config_without_capture_rules_still_deserializes() {
        let mut value = serde_json::to_value(Settings::default()).unwrap();
        let object = value.as_object_mut().unwrap();
        object.remove("capture_rules");

        let content = serde_json::to_string(&value).unwrap();
        let settings: Settings = serde_json::from_str(&content)
            .expect("old config without capture_rules must still deserialize");

        assert!(settings.capture_rules.is_empty());
    }

    #[test]
    fn strip_trigger_defaults_to_true() {
        let json = r###"{
            "id": "r1",
            "name": "Test",
            "conditions": [{ "type": "hashtag", "value": "#heute" }],
            "match": "any",
            "actions": [{ "type": "append", "note": "Liste.md" }]
        }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        let action = match &rule.actions[0] {
            CaptureRuleAction::Append { strip_trigger, .. } => *strip_trigger,
            _ => panic!("expected Append action"),
        };
        assert!(action, "stripTrigger must default to true");
    }

    #[test]
    fn camel_case_keys_roundtrip() {
        let json = r###"{
            "id": "r1",
            "name": "Test",
            "conditions": [{ "type": "contains", "value": "x", "caseSensitive": true }],
            "match": "all",
            "actions": [{
                "type": "append",
                "note": "Liste.md",
                "heading": "Section Heute",
                "position": "top",
                "createHeadingIfMissing": true,
                "stripTrigger": false
            }]
        }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&rule).unwrap();

        // camelCase Keys kommen beim Re-Serialize wieder heraus
        assert!(serialized.contains("\"caseSensitive\":true"));
        assert!(serialized.contains("\"createHeadingIfMissing\":true"));
        assert!(serialized.contains("\"stripTrigger\":false"));

        let round: CaptureRule = serde_json::from_str(&serialized).unwrap();
        match &round.actions[0] {
            CaptureRuleAction::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            } => {
                assert_eq!(note, "Liste.md");
                assert_eq!(heading, "Section Heute");
                assert_eq!(position, "top");
                assert!(*create_heading_if_missing);
                assert!(!*strip_trigger);
                assert!(*show_timestamp, "showTimestamp must default to true");
            }
            _ => panic!("expected Append action"),
        }
        assert_eq!(round.match_mode, "all");
        assert_eq!(
            round.conditions,
            vec![CaptureRuleCondition::Contains {
                value: "x".to_string(),
                case_sensitive: true,
            }]
        );
    }

    #[test]
    fn unknown_condition_and_action_survive_roundtrip_losslessly() {
        let json = r###"{
            "id": "r1",
            "name": "Future",
            "conditions": [
                { "type": "time_between", "value": "21:00", "extra": 42 },
                { "type": "hashtag", "value": "#heute" }
            ],
            "match": "any",
            "actions": [
                { "type": "frontmatter", "key": "tags", "value": "heute" },
                { "type": "append", "note": "Liste.md" }
            ]
        }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();

        // Unknown bleibt als vollstaendiges Objekt erhalten (nicht Value::Null)
        assert_eq!(rule.conditions.len(), 2);
        match &rule.conditions[0] {
            CaptureRuleCondition::Unknown(v) => {
                assert_eq!(v["type"], "time_between");
                assert_eq!(v["extra"], 42);
            }
            other => panic!("expected Unknown condition, got {:?}", other),
        }
        assert_eq!(rule.actions.len(), 2);
        match &rule.actions[0] {
            CaptureRuleAction::Unknown(v) => {
                assert_eq!(v["type"], "frontmatter");
                assert_eq!(v["value"], "heute");
            }
            other => panic!("expected Unknown action, got {:?}", other),
        }

        // Roundtrip: Unknown-Payload geht verlustfrei durch Re-Serialize
        let serialized = serde_json::to_string(&rule).unwrap();
        assert!(serialized.contains("time_between"));
        assert!(serialized.contains("\"extra\":42"));
        assert!(serialized.contains("frontmatter"));

        let round: CaptureRule = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(
            round.conditions[0],
            CaptureRuleCondition::Unknown(_)
        ));
        assert!(matches!(round.actions[0], CaptureRuleAction::Unknown(_)));
    }

    #[test]
    fn settings_with_unknown_rule_types_validate_and_roundtrip() {
        let mut settings = Settings::default();
        let json = r###"{
            "id": "r1",
            "name": "Future",
            "conditions": [{ "type": "time_between", "value": "21:00" }],
            "match": "any",
            "actions": [{ "type": "frontmatter", "key": "tags", "value": "heute" }]
        }"###;
        settings.capture_rules = vec![serde_json::from_str(json).unwrap()];

        // Rule-agnostische Validierung darf nicht fehlschlagen
        settings
            .validate()
            .expect("unknown rule types must not fail validation");

        let serialized = serde_json::to_string(&settings).unwrap();
        let round: Settings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(round.capture_rules.len(), 1);
    }

    #[test]
    fn rule_missing_id_and_name_still_deserializes() {
        // Hand-edited or foreign configs may omit id/name; the whole settings
        // file must never fail to deserialize because of it (persistence.rs
        // would otherwise overwrite config.json with defaults).
        let json = r###"{ "conditions": [], "actions": [] }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.id, "");
        assert_eq!(rule.name, "");
        assert!(rule.enabled);
    }

    #[test]
    fn match_mode_defaults_to_any_and_renames_to_match() {
        let json = r###"{ "id": "r1", "name": "T", "conditions": [], "actions": [] }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.match_mode, "any");

        let serialized = serde_json::to_string(&rule).unwrap();
        assert!(serialized.contains("\"match\":\"any\""));
    }
}
