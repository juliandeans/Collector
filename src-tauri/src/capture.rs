use chrono::Local;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::PathBuf;

use crate::log_safety::{redact_path, summarize_text_len};
use crate::settings::Settings;

#[derive(Debug)]
pub struct CaptureResult {
    #[allow(dead_code)]
    pub success: bool,
    pub message: String,
}

fn generate_header(template: &str) -> String {
    let now = Local::now();

    template
        .replace("YYYY", &now.format("%Y").to_string())
        .replace("MM", &now.format("%m").to_string())
        .replace("DD", &now.format("%d").to_string())
        .replace("HH", &now.format("%H").to_string())
        .replace("mm", &now.format("%M").to_string())
        .replace("ss", &now.format("%S").to_string())
}

pub fn build_daily_note_path(settings: &Settings) -> String {
    let now = Local::now();

    let filename = settings.daily_note_format
        .replace("YYYY", &now.format("%Y").to_string())
        .replace("MM", &now.format("%m").to_string())
        .replace("DD", &now.format("%d").to_string());

    let mut path = settings.daily_note_folder.clone();

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

pub fn save_as_note(content: &str, settings: &Settings) -> Result<CaptureResult, String> {
    let notes_folder = &settings.notes_folder;

    let notes_path = PathBuf::from(&settings.vault_path).join(notes_folder);

    fs::create_dir_all(&notes_path)
        .map_err(|e| format!("Failed to create notes directory: {}", e))?;

    let filename = generate_filename_from_template(&settings.note_filename_template);
    let file_path = notes_path.join(&filename);

    let final_content = if !settings.note_template.is_empty() {
        format!("{}\n\n{}", settings.note_template, content)
    } else {
        content.to_string()
    };

    fs::write(&file_path, final_content).map_err(|e| format!("Failed to write note file: {}", e))?;

    log::info!("Note saved (file={})", redact_path(&file_path));

    Ok(CaptureResult {
        success: true,
        message: format!("Note saved: {}", filename),
    })
}

fn generate_filename_from_template(template: &str) -> String {
    let now = Local::now();

    let mut filename = template
        .replace("YYYY", &now.format("%Y").to_string())
        .replace("MM", &now.format("%m").to_string())
        .replace("DD", &now.format("%d").to_string())
        .replace("HH", &now.format("%H").to_string())
        .replace("mm", &now.format("%M").to_string())
        .replace("ss", &now.format("%S").to_string());

    if !filename.ends_with(".md") {
        filename.push_str(".md");
    }

    filename
}

pub fn append_to_daily_note(captured_text: &str, settings: &Settings) -> Result<(), String> {
    if captured_text.trim().is_empty() {
        return Err("Nichts zum Anhängen".to_string());
    }

    let daily_path_template = build_daily_note_path(settings);

    let vault_path = PathBuf::from(&settings.vault_path);
    let file_path = vault_path.join(&daily_path_template);

    log::info!(
        "Appending to daily note (file={}, chars={})",
        redact_path(&file_path),
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
        .open(&file_path)
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
}
