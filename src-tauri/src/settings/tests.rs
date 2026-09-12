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
