use super::*;
use std::path::Path;

#[test]
fn test_generate_header() {
    let header = generate_header("#### HH:mm");
    assert!(header.starts_with("#### "));
    assert!(header.contains(":"));
}

#[test]
fn test_generate_header_12h_tokens() {
    // hh: 12h mit fuehrender Null (z.B. "03")
    let h = generate_header("hh");
    assert!(h.len() == 2 && h.chars().all(|c| c.is_ascii_digit()));

    // h: 12h ohne fuehrende Null (z.B. "3" oder "12")
    let h = generate_header("h");
    assert!(!h.is_empty() && h.chars().all(|c| c.is_ascii_digit()));

    // a: lowercase am/pm
    let a = generate_header("a");
    assert!(a == "am" || a == "pm");

    // A: uppercase AM/PM
    let a = generate_header("A");
    assert!(a == "AM" || a == "PM");

    // Wichtig: Single-Pass testet, dass 'a' in "am"/"pm" nicht
    // nochmal ersetzt wird und 'A' in "AM"/"PM" nicht nochmal
    let combo = generate_header("h:mm a");
    // Darf kein "pmpm", "amam", "pmPM", "amAM" enthalten
    assert!(!combo.contains("pmpm"));
    assert!(!combo.contains("amam"));
    assert!(!combo.contains("pmPM"));
    assert!(!combo.contains("amAM"));
    // Muss am Ende "am" oder "pm" enden (nicht mehr)
    assert!(combo.ends_with("am") || combo.ends_with("pm"));

    let combo_upper = generate_header("h:mm A");
    assert!(combo_upper.ends_with("AM") || combo_upper.ends_with("PM"));
    assert!(!combo_upper.contains("pmPM"));
    assert!(!combo_upper.contains("amAM"));
}

#[test]
fn test_generate_header_hh_before_h() {
    // "hh" muss als Einheit erkannt werden, nicht als zwei "h"
    let hh = generate_header("hh");
    assert_eq!(hh.len(), 2);
    // Wert muss zwischen "01" und "12" liegen
    let val: u32 = hh.parse().unwrap();
    assert!((1..=12).contains(&val));
}

#[test]
fn test_parse_daily_note_path() {
    let path = parse_daily_note_path("Tagebuch/YYYY/YYYY-MM-DD.md");
    assert!(path.contains("/"));
    assert!(path.ends_with(".md"));
    assert!(!path.contains("YYYY"));
}

#[test]
fn test_build_daily_note_path() {
    let settings = Settings {
        daily_note_folder: "Journal/Notes/".to_string(),
        daily_note_format: "YYYY-MM-DD".to_string(),
        ..Default::default()
    };

    let path = build_daily_note_path(&settings);
    assert!(path.starts_with("Journal/Notes/"));
    assert!(path.ends_with(".md"));
    assert!(!path.contains("YYYY"));
    assert!(!path.contains("MM"));
    assert!(!path.contains("DD"));
}

#[test]
fn test_build_daily_note_path_no_trailing_slash() {
    let settings = Settings {
        daily_note_folder: "Tagebuch".to_string(),
        daily_note_format: "YYYY-MM-DD".to_string(),
        ..Default::default()
    };

    let path = build_daily_note_path(&settings);
    assert!(path.starts_with("Tagebuch/"));
    assert!(path.ends_with(".md"));
}

#[test]
fn test_build_daily_note_path_empty_folder() {
    let settings = Settings {
        daily_note_folder: "".to_string(),
        daily_note_format: "YYYY-MM-DD".to_string(),
        ..Default::default()
    };

    let path = build_daily_note_path(&settings);
    assert!(path.ends_with(".md"));
    assert!(!path.starts_with("/"));
}

#[test]
fn test_build_note_relative_path_from_title() {
    let settings = Settings {
        notes_folder: "Notes".to_string(),
        ..Default::default()
    };

    let path = build_note_relative_path_from_title("My Title", &settings);
    assert_eq!(path, "Notes/My Title.md");
}

#[test]
fn test_build_note_relative_path_from_title_sanitizes_invalid_chars() {
    let settings = Settings {
        notes_folder: "Notes".to_string(),
        ..Default::default()
    };

    let path = build_note_relative_path_from_title(r#"My:/\*?"<>| Title"#, &settings);
    assert_eq!(path, "Notes/My Title.md");
}

#[test]
fn test_build_note_relative_path_from_title_falls_back_when_empty() {
    let settings = Settings {
        notes_folder: "".to_string(),
        note_filename_template: "note-YYYY-MM-DD-HHmmss".to_string(),
        ..Default::default()
    };

    let path = build_note_relative_path_from_title("///", &settings);
    assert!(path.ends_with(".md"));
    assert_ne!(path, ".md");
}

#[test]
fn test_time_tokens_morning_0905() {
    use chrono::TimeZone;
    // 09:05:30 AM on 2024-03-15
    let dt = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 9, 5, 30).unwrap();

    // HH: 24-hour with leading zero
    assert_eq!(generate_header_with_time("HH", dt), "09");

    // hh: 12-hour with leading zero
    assert_eq!(generate_header_with_time("hh", dt), "09");

    // h: 12-hour without leading zero
    assert_eq!(generate_header_with_time("h", dt), "9");

    // mm: minutes with leading zero
    assert_eq!(generate_header_with_time("mm", dt), "05");

    // ss: seconds with leading zero
    assert_eq!(generate_header_with_time("ss", dt), "30");

    // A: uppercase AM/PM
    assert_eq!(generate_header_with_time("A", dt), "AM");

    // a: lowercase am/pm
    assert_eq!(generate_header_with_time("a", dt), "am");

    // Combined format: h:mm a
    assert_eq!(generate_header_with_time("h:mm a", dt), "9:05 am");

    // Combined format: HH:mm:ss
    assert_eq!(generate_header_with_time("HH:mm:ss", dt), "09:05:30");

    // Combined format: hh:mm A
    assert_eq!(generate_header_with_time("hh:mm A", dt), "09:05 AM");

    // Full date-time format
    assert_eq!(
        generate_header_with_time("YYYY-MM-DD HH:mm:ss", dt),
        "2024-03-15 09:05:30"
    );
}

#[test]
fn test_time_tokens_afternoon_1430() {
    use chrono::TimeZone;
    // 14:30:45 (2:30:45 PM) on 2024-03-15
    let dt = chrono::Utc
        .with_ymd_and_hms(2024, 3, 15, 14, 30, 45)
        .unwrap();

    // HH: 24-hour with leading zero
    assert_eq!(generate_header_with_time("HH", dt), "14");

    // hh: 12-hour with leading zero
    assert_eq!(generate_header_with_time("hh", dt), "02");

    // h: 12-hour without leading zero
    assert_eq!(generate_header_with_time("h", dt), "2");

    // mm: minutes with leading zero
    assert_eq!(generate_header_with_time("mm", dt), "30");

    // ss: seconds with leading zero
    assert_eq!(generate_header_with_time("ss", dt), "45");

    // A: uppercase AM/PM
    assert_eq!(generate_header_with_time("A", dt), "PM");

    // a: lowercase am/pm
    assert_eq!(generate_header_with_time("a", dt), "pm");

    // Combined format: h:mm a
    assert_eq!(generate_header_with_time("h:mm a", dt), "2:30 pm");

    // Combined format: HH:mm:ss
    assert_eq!(generate_header_with_time("HH:mm:ss", dt), "14:30:45");

    // Combined format: hh:mm A
    assert_eq!(generate_header_with_time("hh:mm A", dt), "02:30 PM");
}

#[test]
fn test_time_tokens_no_double_replacement() {
    use chrono::TimeZone;
    // Test that 'a' in "am"/"pm" is not replaced again
    // and 'A' in "AM"/"PM" is not replaced again
    let dt_am = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 9, 0, 0).unwrap();
    let dt_pm = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 14, 0, 0).unwrap();

    // "a" should produce "am" or "pm", not "amm" or "pmm"
    assert_eq!(generate_header_with_time("a", dt_am), "am");
    assert_eq!(generate_header_with_time("a", dt_pm), "pm");

    // "A" should produce "AM" or "PM", not "AMM" or "PMM"
    assert_eq!(generate_header_with_time("A", dt_am), "AM");
    assert_eq!(generate_header_with_time("A", dt_pm), "PM");

    // Combined: the 'a' in the output "am" should not be replaced by the 'A' token
    let result = generate_header_with_time("a A", dt_am);
    assert_eq!(result, "am AM");

    let result = generate_header_with_time("A a", dt_pm);
    assert_eq!(result, "PM pm");
}

#[test]
fn test_time_tokens_hh_before_h() {
    use chrono::TimeZone;
    // Verify "hh" is matched as a single token, not as "h" + "h"
    let dt = chrono::Utc
        .with_ymd_and_hms(2024, 3, 15, 14, 30, 0)
        .unwrap();

    // "hh" should be "02" (14:30 = 2:30 PM)
    assert_eq!(generate_header_with_time("hh", dt), "02");

    // "h" alone should be "2"
    assert_eq!(generate_header_with_time("h", dt), "2");

    // "hhh" should be "02" + "2" = "022"
    assert_eq!(generate_header_with_time("hhh", dt), "022");
}

#[test]
fn test_time_tokens_midnight_and_noon() {
    use chrono::TimeZone;

    // Midnight: 00:00 (12:00 AM)
    let midnight = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 0, 0, 0).unwrap();
    assert_eq!(generate_header_with_time("HH", midnight), "00");
    assert_eq!(generate_header_with_time("hh", midnight), "12");
    assert_eq!(generate_header_with_time("h", midnight), "12");
    assert_eq!(generate_header_with_time("A", midnight), "AM");

    // Noon: 12:00 (12:00 PM)
    let noon = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 12, 0, 0).unwrap();
    assert_eq!(generate_header_with_time("HH", noon), "12");
    assert_eq!(generate_header_with_time("hh", noon), "12");
    assert_eq!(generate_header_with_time("h", noon), "12");
    assert_eq!(generate_header_with_time("A", noon), "PM");
}

// -- tests for new functions ----------------------------------------

#[test]
fn test_build_daily_note_advanced_uri() {
    let uri = build_daily_note_advanced_uri("My Vault");
    assert!(uri.starts_with("obsidian://adv-uri?"));
    assert!(uri.contains("vault=My%20Vault"));
    assert!(uri.contains("daily=true"));
}

#[test]
fn test_build_daily_note_advanced_uri_special_chars() {
    let uri = build_daily_note_advanced_uri("Vault & Co");
    assert!(uri.starts_with("obsidian://adv-uri?"));
    assert!(uri.contains("vault=Vault%20%26%20Co"));
    assert!(uri.contains("daily=true"));
}

#[test]
fn test_append_to_note_works_independently() {
    // append_to_note must still be callable (sync, no section logic)
    // We test the error path only (missing file) — no real I/O
    let settings = Settings::default();
    let result = append_to_note(
        "hello",
        Path::new("/tmp/collector-test-nonexistent.md"),
        &settings,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"));
}

#[tokio::test]
async fn test_append_daily_note_missing_file_no_create() {
    // Daily note fehlt + create_if_missing=false => Fehler, bevor irgendwas passiert
    let tmp = std::env::temp_dir().join("collector-test-missing-daily.md");
    let _ = std::fs::remove_file(&tmp); // clean slate

    let settings = Settings {
        daily_note_folder: "Journal/".to_string(),
        daily_note_create_if_missing: false,
        ..Default::default()
    };

    let result = append_to_daily_note("test text", &tmp, &settings).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("not found"));

    // Es wurde KEINE Datei erstellt
    assert!(!tmp.exists());
}

#[tokio::test]
async fn test_append_daily_note_integration() {
    // Vollstaendiger Pfad: existierende Daily Note mit ### Note und ## Other
    let tmp = std::env::temp_dir().join("collector-test-integration");
    let file_path = tmp.join("daily.md");
    let _ = std::fs::create_dir_all(&tmp);

    let initial = concat!(
        "---\n",
        "date: 2026-06-02\n",
        "---\n",
        "\n",
        "### Note\n",
        "\n",
        "- old entry\n",
        "\n",
        "## Other\n",
        "\n",
        "- other content\n",
    );
    std::fs::write(&file_path, initial).unwrap();

    let settings = Settings {
        daily_note_folder: "Journal/".to_string(),
        daily_note_target_heading: "### Note".to_string(),
        daily_note_insert_position: "bottom".to_string(),
        daily_note_create_heading_if_missing: false,
        daily_note_create_if_missing: false,
        ..Default::default()
    };

    let result = append_to_daily_note("- new entry", &file_path, &settings).await;
    assert!(
        result.is_ok(),
        "append_to_daily_note should succeed: {:?}",
        result
    );

    let content = std::fs::read_to_string(&file_path).unwrap();

    // Alter Inhalt erhalten
    assert!(content.contains("- old entry"));
    assert!(content.contains("- other content"));
    assert!(content.contains("## Other"));

    // Neuer Eintrag vorhanden
    assert!(content.contains("- new entry"));

    // Reihenfolge: ### Note > old entry > new entry > ## Other
    let heading_pos = content.find("### Note").unwrap();
    let old_pos = content.find("- old entry").unwrap();
    let new_pos = content.find("- new entry").unwrap();
    let other_pos = content.find("## Other").unwrap();

    assert!(heading_pos < old_pos, "### Note before - old entry");
    assert!(old_pos < new_pos, "- old entry before - new entry");
    assert!(new_pos < other_pos, "- new entry before ## Other");

    // Aufraeumen
    let _ = std::fs::remove_dir_all(&tmp);
}

// ── append_to_note_section ──────────────────────────────────────────────

fn setup_section_test_file(unique: &str, initial: &str) -> std::path::PathBuf {
    // Eindeutiges Unterverzeichnis pro Test: die Tests laufen parallel und
    // duerfen sich nicht gegenseitig ihre Dateien wegloeschen.
    let tmp = std::env::temp_dir().join(format!("collector-test-section-{unique}"));
    let _ = std::fs::create_dir_all(&tmp);
    let file_path = tmp.join("note.md");
    std::fs::write(&file_path, initial).unwrap();
    file_path
}

fn cleanup_section_test_dir(unique: &str) {
    let _ = std::fs::remove_dir_all(
        std::env::temp_dir().join(format!("collector-test-section-{unique}")),
    );
}

#[test]
fn test_append_to_note_section_missing_file_is_hard_error() {
    let settings = Settings::default();
    let result = append_to_note_section(
        "hello",
        Path::new("/tmp/collector-test-section-does-not-exist.md"),
        "## Heute",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.contains("Note not found"));
}

#[test]
fn test_append_to_note_section_empty_text_is_rejected() {
    let settings = Settings::default();
    let result = append_to_note_section(
        "   ",
        Path::new("/tmp/collector-test-section-anywhere.md"),
        "## Heute",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Nothing to append"));
}

#[test]
fn test_append_to_note_section_bottom_of_existing_heading() {
    let file_path = setup_section_test_file(
        "bottom",
        concat!(
            "# Liste\n",
            "\n",
            "## Heute\n",
            "\n",
            "- old entry\n",
            "\n",
            "## Morgen\n",
            "\n",
            "- tomorrow\n",
        ),
    );
    let settings = Settings::default();

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    let heute = content.find("## Heute").unwrap();
    let old_pos = content.find("- old entry").unwrap();
    let new_pos = content.find("- new entry").unwrap();
    let morgen = content.find("## Morgen").unwrap();

    assert!(heute < old_pos && old_pos < new_pos && new_pos < morgen);

    // Sektion "Morgen" unveraendert
    assert!(content.contains("- tomorrow"));

    cleanup_section_test_dir("bottom");
}

#[test]
fn test_append_to_note_section_top_of_existing_heading() {
    let file_path = setup_section_test_file(
        "top",
        concat!("# Liste\n", "\n", "## Heute\n", "\n", "- old entry\n",),
    );
    let settings = Settings::default();

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "top",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    let new_pos = content.find("- new entry").unwrap();
    let old_pos = content.find("- old entry").unwrap();
    assert!(
        new_pos < old_pos,
        "top insert must land above existing entry"
    );

    cleanup_section_test_dir("top");
}

#[test]
fn test_append_to_note_section_missing_heading_append_to_file_end() {
    let file_path = setup_section_test_file("no-heading", "# Liste\n\n- existing\n");
    let settings = Settings::default();

    // create_heading_if_missing = false => Fallback: ans Dateiende
    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("- existing"));
    assert!(content.contains("- new entry"));
    // Keine Headline erzeugt
    assert!(!content.contains("## Heute"));
    // Neuer Eintrag am Ende
    assert!(content.find("- new entry").unwrap() > content.find("- existing").unwrap());

    cleanup_section_test_dir("no-heading");
}

#[test]
fn test_append_to_note_section_missing_heading_created_when_enabled() {
    let file_path = setup_section_test_file("create-heading", "# Liste\n\n- existing\n");
    let settings = Settings::default();

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "bottom",
        true,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("## Heute"));
    assert!(content.contains("- existing"));
    assert!(content.contains("- new entry"));

    cleanup_section_test_dir("create-heading");
}

#[test]
fn test_append_to_note_section_empty_heading_appends_to_file_end() {
    let file_path = setup_section_test_file("empty-heading", "# Liste\n\n- existing\n");
    let settings = Settings::default();

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("- existing"));
    let new_pos = content.find("- new entry").unwrap();
    let existing_pos = content.find("- existing").unwrap();
    assert!(new_pos > existing_pos);

    cleanup_section_test_dir("empty-heading");
}

#[test]
fn test_append_to_note_section_entry_has_generated_header() {
    let file_path = setup_section_test_file("with-header", "# Liste\n\n## Heute\n\n- old\n");
    let settings = Settings {
        entry_header: "#### HH:mm".to_string(),
        ..Default::default()
    };

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    // Entry-Header (#### HH:MM-Format) wurde dem Eintrag vorangestellt
    assert!(content.contains("#### "));
    let header_pos = content.find("#### ").unwrap();
    let new_pos = content.find("- new entry").unwrap();
    assert!(header_pos < new_pos);

    cleanup_section_test_dir("with-header");
}

#[test]
fn test_append_to_note_section_position_is_case_insensitive() {
    let file_path = setup_section_test_file("case-position", "# Liste\n\n## Heute\n\n- old\n");
    let settings = Settings::default();

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "TOP",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    let new_pos = content.find("- new entry").unwrap();
    let old_pos = content.find("- old").unwrap();
    assert!(
        new_pos < old_pos,
        "\"TOP\" must be normalized to top insert"
    );

    cleanup_section_test_dir("case-position");
}

#[test]
fn test_append_to_note_section_normalizes_crlf_to_lf() {
    let file_path =
        setup_section_test_file("crlf", "# Liste\r\n\r\n## Heute\r\n\r\n- old entry\r\n");
    let settings = Settings::default();

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "bottom",
        false,
        true,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    // insert_into_markdown_section baut die Datei aus lines() neu auf und
    // normalisiert dabei die GESAMTE Datei auf LF (erwartetes Verhalten)
    assert!(!content.contains('\r'), "CRLF must be normalized to LF");
    assert!(content.contains("## Heute"));
    assert!(content.contains("- old entry"));
    assert!(content.contains("- new entry"));

    cleanup_section_test_dir("crlf");
}

#[test]
fn test_append_to_note_section_without_timestamp_has_no_header() {
    let file_path = setup_section_test_file("no-timestamp", "# Liste\n\n## Heute\n\n- old\n");
    let settings = Settings {
        entry_header: "#### HH:mm".to_string(),
        ..Default::default()
    };

    let result = append_to_note_section(
        "- new entry",
        &file_path,
        "## Heute",
        "bottom",
        false,
        false,
        &settings,
    );
    assert!(result.is_ok(), "expected ok: {:?}", result);

    let content = std::fs::read_to_string(&file_path).unwrap();
    assert!(content.contains("- new entry"));
    // Kein Entry-Header ueber dem Inhalt
    assert!(
        !content.contains("#### "),
        "show_timestamp=false must not generate an entry header"
    );

    cleanup_section_test_dir("no-timestamp");
}
