use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;

use crate::log_safety::{redact_path, summarize_text_len};
use crate::settings::Settings;

#[derive(Debug)]
pub struct CaptureResult {
    #[allow(dead_code)]
    pub success: bool,
    pub message: String,
}

fn generate_header(template: &str) -> String {
    generate_header_with_time(template, Local::now())
}

fn generate_header_with_time<Tz: chrono::TimeZone>(
    template: &str,
    dt: chrono::DateTime<Tz>,
) -> String
where
    Tz::Offset: std::fmt::Display,
{
    // 12h-Zeitwerte vorbereiten
    let hour_12 = dt.format("%I").to_string();
    let hour_12_nopad = if hour_12.starts_with('0') {
        hour_12[1..].to_string()
    } else {
        hour_12.clone()
    };
    let ampm_lower = dt.format("%P").to_string();
    let ampm_upper = dt.format("%p").to_string();

    // Token → Wert Zuordnung.
    // Längere Token zuerst, damit z.B. "hh" vor "h" matched.
    let tokens: &[(&str, String)] = &[
        ("YYYY", dt.format("%Y").to_string()),
        ("MM", dt.format("%m").to_string()),
        ("DD", dt.format("%d").to_string()),
        ("HH", dt.format("%H").to_string()),
        ("hh", hour_12),
        ("mm", dt.format("%M").to_string()),
        ("ss", dt.format("%S").to_string()),
        ("h", hour_12_nopad),
        ("a", ampm_lower),
        ("A", ampm_upper),
    ];

    // Single-Pass: jedes Token wird nur im Original-Template erkannt,
    // sodass eingesetzte Werte (z.B. "pm" enthält 'a') nie erneut ersetzt werden.
    let mut result = String::with_capacity(template.len() * 2);
    let mut remaining = template;
    while !remaining.is_empty() {
        if let Some((tok, val)) = tokens.iter().find(|(tok, _)| remaining.starts_with(tok)) {
            result.push_str(val);
            remaining = &remaining[tok.len()..];
        } else {
            // Safety: wir nehmen ein UTF-8-Zeichen, nicht nur ein Byte.
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }
    result
}

pub fn build_daily_note_path(settings: &Settings) -> String {
    let now = Local::now();

    let filename = settings
        .daily_note_format
        .replace("YYYY", &now.format("%Y").to_string())
        .replace("MM", &now.format("%m").to_string())
        .replace("DD", &now.format("%d").to_string());

    let mut path = parse_daily_note_path(&settings.daily_note_folder);

    if !path.is_empty() && !path.ends_with('/') {
        path.push('/');
    }

    path.push_str(&filename);

    if !path.ends_with(".md") {
        path.push_str(".md");
    }

    path
}

#[allow(dead_code)]
pub fn parse_daily_note_path(template: &str) -> String {
    let now = Local::now();

    template
        .replace("YYYY", &now.format("%Y").to_string())
        .replace("MM", &now.format("%m").to_string())
        .replace("DD", &now.format("%d").to_string())
}

pub fn build_note_relative_path(settings: &Settings) -> String {
    let filename = generate_filename_from_template(&settings.note_filename_template);
    let notes_folder = settings.notes_folder.trim_end_matches('/');

    if notes_folder.is_empty() {
        filename
    } else {
        format!("{notes_folder}/{filename}")
    }
}

pub fn build_note_relative_path_from_title(title: &str, settings: &Settings) -> String {
    let sanitized = sanitize_note_title(title);
    let filename = if sanitized.is_empty() {
        generate_filename_from_template(&settings.note_filename_template)
    } else if sanitized.ends_with(".md") {
        sanitized
    } else {
        format!("{}.md", sanitized)
    };

    let notes_folder = settings.notes_folder.trim_end_matches('/');
    if notes_folder.is_empty() {
        filename
    } else {
        format!("{}/{}", notes_folder, filename)
    }
}

fn sanitize_note_title(title: &str) -> String {
    title
        .chars()
        .filter(|c| {
            !matches!(
                c,
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0'
            )
        })
        .collect::<String>()
        .trim()
        .to_string()
}

pub fn save_note_at_path(
    content: &str,
    file_path: &Path,
    filename: &str,
    settings: &Settings,
) -> Result<CaptureResult, String> {
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create notes directory: {}", e))?;
    }

    let final_content = if !settings.note_template.is_empty() {
        format!("{}\n\n{}", settings.note_template, content)
    } else {
        content.to_string()
    };

    fs::write(&file_path, final_content)
        .map_err(|e| format!("Failed to write note file: {}", e))?;

    log::info!("Note saved (file={})", redact_path(&file_path));

    Ok(CaptureResult {
        success: true,
        message: format!("Note saved: {}", filename),
    })
}

fn generate_filename_from_template(template: &str) -> String {
    let now = Local::now();

    // 12h-Zeitwerte vorbereiten
    let hour_12 = now.format("%I").to_string();
    let hour_12_nopad = if hour_12.starts_with('0') {
        hour_12[1..].to_string()
    } else {
        hour_12.clone()
    };
    let ampm_lower = now.format("%P").to_string();
    let ampm_upper = now.format("%p").to_string();

    // Token → Wert Zuordnung.
    // Längere Token zuerst, damit z.B. "hh" vor "h" matched.
    // WICHTIG: Die Token-Namen selbst enthalten keine problematischen Zeichen,
    // aber Nutzer sollten Doppelpunkte in Filename-Templates vermeiden
    // (macOS erlaubt keine Doppelpunkte in Dateinamen).
    let tokens: &[(&str, String)] = &[
        ("YYYY", now.format("%Y").to_string()),
        ("MM", now.format("%m").to_string()),
        ("DD", now.format("%d").to_string()),
        ("HH", now.format("%H").to_string()),
        ("hh", hour_12),
        ("mm", now.format("%M").to_string()),
        ("ss", now.format("%S").to_string()),
        ("h", hour_12_nopad),
        ("a", ampm_lower),
        ("A", ampm_upper),
    ];

    // Single-Pass: jedes Token wird nur im Original-Template erkannt,
    // sodass eingesetzte Werte (z.B. "pm" enthält 'a') nie erneut ersetzt werden.
    let mut result = String::with_capacity(template.len() * 2);
    let mut remaining = template;
    while !remaining.is_empty() {
        if let Some((tok, val)) = tokens.iter().find(|(tok, _)| remaining.starts_with(tok)) {
            result.push_str(val);
            remaining = &remaining[tok.len()..];
        } else {
            // Safety: wir nehmen ein UTF-8-Zeichen, nicht nur ein Byte.
            let ch = remaining.chars().next().unwrap();
            result.push(ch);
            remaining = &remaining[ch.len_utf8()..];
        }
    }

    let mut filename = result;
    if !filename.ends_with(".md") {
        filename.push_str(".md");
    }

    filename
}

pub fn append_to_daily_note(
    captured_text: &str,
    file_path: &Path,
    settings: &Settings,
) -> Result<(), String> {
    if captured_text.trim().is_empty() {
        return Err("Nothing to append".to_string());
    }

    log::info!(
        "Appending to daily note (file={}, chars={})",
        redact_path(file_path),
        summarize_text_len(captured_text)
    );

    if !file_path.exists() {
        return Err(format!(
            "Daily note not found: {:?}. Please create the file first.",
            file_path
        ));
    }

    let header = generate_header(&settings.entry_header);

    let entry = format!(
        "{}
{}
",
        header, captured_text
    );

    let needs_leading_newline = {
        let mut check_file =
            File::open(&file_path).map_err(|e| format!("Cannot open daily note: {}", e))?;

        let file_size = check_file
            .metadata()
            .map_err(|e| format!("Cannot read file metadata: {}", e))?
            .len();

        if file_size == 0 {
            false
        } else {
            let seek_pos = if file_size >= 2 { file_size - 2 } else { 0 };
            check_file
                .seek(SeekFrom::Start(seek_pos))
                .map_err(|e| format!("Cannot set file position: {}", e))?;

            let mut check_buffer = [0u8; 2];
            let bytes_read = check_file
                .read(&mut check_buffer)
                .map_err(|e| format!("Cannot read file: {}", e))?;

            match bytes_read {
                2 => !(check_buffer == [0x0D, 0x0A] || check_buffer[1] == 0x0A),
                1 => check_buffer[0] != 0x0A,
                _ => true,
            }
        }
    };

    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path)
        .map_err(|e| format!("Cannot open daily note: {}", e))?;

    if needs_leading_newline {
        file.write_all(
            b"
",
        )
        .map_err(|e| format!("Cannot write to file: {}", e))?;
    }

    file.write_all(entry.as_bytes())
        .map_err(|e| format!("Cannot write to file: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Cannot sync file: {}", e))?;

    log::info!(
        "Successfully appended to daily note (file={})",
        redact_path(&file_path)
    );
    Ok(())
}

pub fn append_to_note(
    captured_text: &str,
    file_path: &Path,
    settings: &Settings,
) -> Result<(), String> {
    append_to_daily_note(captured_text, file_path, settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_header() {
        let header = generate_header("#### HH:mm");
        assert!(header.starts_with("#### "));
        assert!(header.contains(":"));
    }

    #[test]
    fn test_generate_header_12h_tokens() {
        // hh: 12h mit führender Null (z.B. "03")
        let h = generate_header("hh");
        assert!(h.len() == 2 && h.chars().all(|c| c.is_ascii_digit()));

        // h: 12h ohne führende Null (z.B. "3" oder "12")
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
        let dt = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 45).unwrap();

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
        let dt = chrono::Utc.with_ymd_and_hms(2024, 3, 15, 14, 30, 0).unwrap();

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
}
