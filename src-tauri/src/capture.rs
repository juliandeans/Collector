use chrono::Local;
use std::fs::{self, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::Path;
use std::time::Duration;

use crate::fs_util::atomic_write_text;
use crate::log_safety::{redact_path, summarize_text_len};
use crate::markdown_sections::{self, InsertPosition};
use crate::settings::Settings;

#[derive(Debug)]
pub struct CaptureResult {
    #[allow(dead_code)]
    pub success: bool,
    pub message: String,
}
mod template;
pub use template::{generate_header, render_datetime_template};
#[cfg(test)]
pub(crate) use template::generate_header_with_time;


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
    let notes_folder = render_datetime_template(&settings.notes_folder);
    let notes_folder = notes_folder.trim_end_matches('/');

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

    let notes_folder = render_datetime_template(&settings.notes_folder);
    let notes_folder = notes_folder.trim_end_matches('/');
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
    let mut filename = render_datetime_template(template);
    if !filename.ends_with(".md") {
        filename.push_str(".md");
    }

    filename
}

/// Build an Obsidian Advanced URI to open/create today's daily note.
pub fn build_daily_note_advanced_uri(vault_name: &str) -> String {
    format!(
        "obsidian://adv-uri?vault={}&daily=true",
        urlencoding::encode(vault_name)
    )
}

/// Open Advanced URI and wait for the daily note file to appear and stabilise.
async fn ensure_daily_note_created(file_path: &Path, settings: &Settings) -> Result<(), String> {
    let adv_uri = build_daily_note_advanced_uri(&settings.vault_name);

    log::info!(
        "Opening Advanced URI to create daily note (file={}, uri={})",
        redact_path(file_path),
        adv_uri
    );

    open::that(&adv_uri).map_err(|e| format!("Failed to open Advanced URI: {}", e))?;

    // Phase 1: wait for file to appear
    let timeout = Duration::from_millis(settings.daily_note_create_timeout_ms);
    let start = std::time::Instant::now();

    loop {
        if file_path.exists() {
            break;
        }
        if start.elapsed() > timeout {
            return Err(format!(
                concat!(
                    "Timed out waiting for daily note to be created: {:?}. ",
                    "Please check that Obsidian is running and the vault \"{}\" exists."
                ),
                file_path, settings.vault_name
            ));
        }
        tokio::time::sleep(Duration::from_millis(250)).await;
    }

    // Phase 2: wait for file to stabilise (Obsidian / Templater still writing)
    let stability_max = Duration::from_secs(5);
    let stab_start = std::time::Instant::now();
    let mut last_len = 0u64;
    let mut last_modified = std::time::SystemTime::UNIX_EPOCH;
    let mut stable_count = 0u8;

    loop {
        if stab_start.elapsed() > stability_max {
            log::warn!("Stability check timed out for {:?}, proceeding", file_path);
            return Ok(());
        }

        match fs::metadata(file_path) {
            Ok(meta) => {
                let cur_len = meta.len();
                let cur_mtime = meta.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
                if cur_len > 0 && cur_len == last_len && cur_mtime == last_modified {
                    stable_count += 1;
                    if stable_count >= 3 {
                        return Ok(());
                    }
                } else {
                    stable_count = 0;
                    last_len = cur_len;
                    last_modified = cur_mtime;
                }
            }
            Err(_) => {
                stable_count = 0;
            }
        }

        tokio::time::sleep(Duration::from_millis(250)).await;
    }
}

/// Append `captured_text` to the daily note using section-insert logic.
///
/// If the file does not exist and `daily_note_create_if_missing` is true,
/// it opens the Obsidian Advanced URI to create it, waits for it, then writes
/// unless `captured_text` is empty.
pub async fn append_to_daily_note(
    captured_text: &str,
    file_path: &Path,
    settings: &Settings,
) -> Result<(), String> {
    let is_creation_only = captured_text.trim().is_empty();
    if is_creation_only && !settings.daily_note_create_if_missing {
        return Err("Nothing to append".to_string());
    }
    if settings.daily_note_folder.trim().is_empty() {
        return Err(
            "Daily Note folder is not configured. Please set it in Settings.".to_string(),
        );
    }

    log::info!(
        "Appending to daily note (file={}, chars={})",
        redact_path(file_path),
        summarize_text_len(captured_text)
    );

    // Create daily note when missing (Advanced URI)
    if !file_path.exists() {
        if !settings.daily_note_create_if_missing {
            return Err(format!(
                "Daily note not found: {:?}. Please create the file first.",
                file_path
            ));
        }
        ensure_daily_note_created(file_path, settings).await?;
    }

    if is_creation_only {
        return Ok(());
    }

    // Build the entry with header
    let header = generate_header(&settings.entry_header);
    let entry = format!("{}\n{}", header, captured_text);

    // Read current content
    let content =
        fs::read_to_string(file_path).map_err(|e| format!("Cannot read daily note: {}", e))?;

    // Determine insert position from settings
    let position = match settings.daily_note_insert_position.as_str() {
        "top" => InsertPosition::Top,
        _ => InsertPosition::Bottom,
    };

    // Use section-insert logic
    let new_content = markdown_sections::insert_into_markdown_section(
        &content,
        &entry,
        &settings.daily_note_target_heading,
        position,
        settings.daily_note_create_heading_if_missing,
    );

    atomic_write_text(file_path, &new_content)
        .map_err(|e| format!("Cannot write to daily note: {}", e))?;

    log::info!(
        "Successfully appended to daily note (file={})",
        redact_path(file_path)
    );
    Ok(())
}

/// Append `captured_text` to an arbitrary note file (legacy behaviour).
/// Always appends to end — no section-insert logic.
pub fn append_to_note(
    captured_text: &str,
    file_path: &Path,
    settings: &Settings,
) -> Result<(), String> {
    if captured_text.trim().is_empty() {
        return Err("Nothing to append".to_string());
    }

    log::info!(
        "Appending to note (file={}, chars={})",
        redact_path(file_path),
        summarize_text_len(captured_text)
    );

    if !file_path.exists() {
        return Err(format!(
            "Note not found: {:?}. Please create the file first.",
            file_path
        ));
    }

    let header = generate_header(&settings.entry_header);
    let entry = format!("{}\n{}\n", header, captured_text);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(file_path)
        .map_err(|e| format!("Cannot open note: {}", e))?;

    let file_size = file
        .metadata()
        .map_err(|e| format!("Cannot read file metadata: {}", e))?
        .len();

    if file_size > 0 {
        let seek_pos = if file_size >= 2 { file_size - 2 } else { 0 };
        file.seek(SeekFrom::Start(seek_pos))
            .map_err(|e| format!("Cannot set file position: {}", e))?;

        let mut buf = [0u8; 2];
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("Cannot read file: {}", e))?;

        let ends_with_newline = match n {
            2 => buf == [0x0D, 0x0A] || buf[1] == 0x0A,
            1 => buf[0] == 0x0A,
            _ => false,
        };

        if !ends_with_newline {
            file.write_all(b"\n")
                .map_err(|e| format!("Cannot write to file: {}", e))?;
        }
    }

    file.write_all(entry.as_bytes())
        .map_err(|e| format!("Cannot write to file: {}", e))?;

    file.sync_all()
        .map_err(|e| format!("Cannot sync file: {}", e))?;

    log::info!(
        "Successfully appended to note (file={})",
        redact_path(file_path)
    );
    Ok(())
}

#[cfg(test)]
mod tests;

