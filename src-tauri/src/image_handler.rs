use chrono::Local;
use image::{DynamicImage, ImageFormat};
use serde::Serialize;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::io::Write;

use crate::log_safety::{redact_path, summarize_bytes};
use crate::settings::Settings;

/// Result of saving an image
#[derive(Debug)]
pub struct SavedImage {
    /// Full path where the image was saved
    #[allow(dead_code)]
    pub full_path: PathBuf,
    /// Filename only (for Markdown link)
    pub filename: String,
    /// Final file size in bytes
    #[allow(dead_code)]
    pub size_bytes: usize,
}

#[derive(Debug, Serialize)]
pub struct ProcessedImage {
    pub markdown: String,
    pub saved_path: String,
    pub filename: String,
}

/// Save and compress an image
/// Returns the relative path for use in Markdown link
pub fn save_image(source_path: &Path, settings: &Settings) -> Result<SavedImage, String> {
    let img = image::open(source_path).map_err(|e| format!("Failed to open image: {}", e))?;

    let filename = generate_filename(&settings.image_filename);

    // NEU: Nutze screenshot_path direkt
    let output_dir = PathBuf::from(&settings.screenshot_path);

    fs::create_dir_all(&output_dir)
        .map_err(|e| format!("Failed to create screenshot directory: {}", e))?;

    let output_path = output_dir.join(&filename);

    let size_bytes = compress_and_save(&img, &output_path, settings.compression_max_kb)?;

    log::info!(
        "Image saved (file={}, size={})",
        redact_path(&output_path),
        summarize_bytes(size_bytes)
    );

    Ok(SavedImage {
        full_path: output_path,
        filename,
        size_bytes,
    })
}

/// Compress image to target size and save
fn compress_and_save(
    img: &DynamicImage,
    output_path: &Path,
    max_size_kb: u32,
) -> Result<usize, String> {
    let max_size_bytes = (max_size_kb * 1024) as usize;

    // Resize if too large (max 1920px width)
    let img = if img.width() > 1920 {
        log::info!(
            "Resizing image from {}x{} to max 1920px width",
            img.width(),
            img.height()
        );
        let ratio = 1920.0 / img.width() as f32;
        let new_height = (img.height() as f32 * ratio) as u32;
        img.resize(1920, new_height, image::imageops::FilterType::Lanczos3)
    } else {
        img.clone()
    };

    // Determine output format based on extension
    let extension = output_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("jpg")
        .to_lowercase();

    // For PNG files, try to save directly first
    if extension == "png" {
        let mut buffer = Vec::new();
        img.write_to(&mut Cursor::new(&mut buffer), ImageFormat::Png)
            .map_err(|e| format!("Failed to encode PNG: {}", e))?;

        // If PNG is small enough, save it
        if buffer.len() <= max_size_bytes {
            fs::write(output_path, &buffer).map_err(|e| format!("Failed to write image: {}", e))?;
            return Ok(buffer.len());
        }
        // Otherwise, fall through to JPEG compression
    }

    // JPEG compression with quality reduction loop
    let mut quality = 85u8;
    loop {
        let mut buffer = Vec::new();

        // Convert to RGB for JPEG (remove alpha channel)
        let rgb_img = img.to_rgb8();
        let dynamic_img = DynamicImage::ImageRgb8(rgb_img);

        // Create JPEG encoder with current quality
        let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, quality);

        encoder
            .encode_image(&dynamic_img)
            .map_err(|e| format!("Failed to encode JPEG: {}", e))?;

        log::debug!(
            "Compressed to {}KB at quality {}",
            buffer.len() / 1024,
            quality
        );

        // Check if we've reached target size or minimum quality
        if buffer.len() <= max_size_bytes || quality < 30 {
            // Change extension to .jpg since we're saving as JPEG
            let jpg_path = output_path.with_extension("jpg");

            fs::write(&jpg_path, &buffer).map_err(|e| format!("Failed to write image: {}", e))?;

            return Ok(buffer.len());
        }

        // Reduce quality for next iteration
        quality = quality.saturating_sub(5);
    }
}

/// Generate filename from template
/// Supports: YYYY, MM, DD, HH, mm, ss
fn generate_filename(template: &str) -> String {
    let now = Local::now();

    let filename = template
        .replace("YYYY", &now.format("%Y").to_string())
        .replace("MM", &now.format("%m").to_string())
        .replace("DD", &now.format("%d").to_string())
        .replace("HH", &now.format("%H").to_string())
        .replace("mm", &now.format("%M").to_string())
        .replace("ss", &now.format("%S").to_string());

    // Add extension if not present
    if !filename.contains('.') {
        format!("{}.jpg", filename)
    } else {
        filename
    }
}

/// Get supported image extensions
pub fn is_supported_image(path: &Path) -> bool {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());

    matches!(
        extension.as_deref(),
        Some("png" | "jpg" | "jpeg" | "webp" | "gif")
    )
}

/// Process a dropped file
/// Returns the markdown link to insert
/// Process a dropped file
/// Returns the markdown link to insert
pub fn process_dropped_file(file_path: &str, settings: &Settings) -> Result<ProcessedImage, String> {
    let source_path = Path::new(file_path);

    // Check if it's a supported image
    if !is_supported_image(source_path) {
        return Err(format!(
            "Unsupported file type. Supported: PNG, JPG, JPEG, WebP, GIF"
        ));
    }

    // Save and compress the image (KEIN vault_path mehr!)
    let saved = save_image(source_path, settings)?;

    // Generate Obsidian wikilink
    // Format: ![[filename.jpg]]
    let markdown_link = build_markdown_link(&saved.filename, settings);
    Ok(ProcessedImage {
        markdown: markdown_link,
        saved_path: saved.full_path.to_string_lossy().to_string(),
        filename: saved.filename,
    })
}

/// Process a dropped file from bytes (when file.path is not available)
/// Saves bytes to temp file first, then processes it
pub fn process_dropped_file_from_bytes(
    bytes: Vec<u8>,
    original_filename: &str,
    settings: &Settings,
) -> Result<ProcessedImage, String> {
    // Create temp directory if it doesn't exist
    let temp_dir = std::env::temp_dir();
    let temp_path = temp_dir.join(format!("tauri_drop_{}", original_filename));

    // Write bytes to temp file
    let mut file = fs::File::create(&temp_path)
        .map_err(|e| format!("Failed to create temp file: {}", e))?;
    file.write_all(&bytes)
        .map_err(|e| format!("Failed to write temp file: {}", e))?;
    drop(file); // Close file before processing

    // Check if it's a supported image based on extension
    let source_path = Path::new(&temp_path);
    if !is_supported_image(source_path) {
        // Try to clean up temp file
        let _ = fs::remove_file(&temp_path);
        return Err(format!(
            "Unsupported file type. Supported: PNG, JPG, JPEG, WebP, GIF"
        ));
    }

    // Save and compress the image
    let saved = save_image(source_path, settings)?;

    // Clean up temp file
    let _ = fs::remove_file(&temp_path);

    // Generate Obsidian wikilink
    // Format: ![[filename.jpg]]
    let markdown_link = build_markdown_link(&saved.filename, settings);
    Ok(ProcessedImage {
        markdown: markdown_link,
        saved_path: saved.full_path.to_string_lossy().to_string(),
        filename: saved.filename,
    })
}

fn build_markdown_link(filename: &str, settings: &Settings) -> String {
    let width = settings.default_image_width.trim();
    if width.is_empty() {
        format!("![[{}]]", filename)
    } else {
        format!("![[{}|{}]]", filename, width)
    }
}

/// Get the Obsidian vault path from settings
/// Tries common vault locations on macOS
#[allow(dead_code)]
pub fn find_vault_path(vault_name: &str) -> Result<PathBuf, String> {
    // Common Obsidian vault locations on macOS
    let home = dirs::home_dir().ok_or_else(|| "Could not find home directory".to_string())?;

    let possible_locations = vec![
        home.join("Documents").join(vault_name),
        home.join("Documents").join("Obsidian").join(vault_name),
        home.join(vault_name),
        home.join("Obsidian").join(vault_name),
        home.join("Library")
            .join("Mobile Documents")
            .join("iCloud~md~obsidian")
            .join("Documents")
            .join(vault_name),
    ];

    for path in possible_locations {
        if path.exists() && path.is_dir() {
            // Check if it's an Obsidian vault (has .obsidian folder)
            if path.join(".obsidian").exists() {
                log::info!("Found vault at: {}", redact_path(&path));
                return Ok(path);
            }
        }
    }

    // If not found, return the default location (user can configure)
    let default_path = home.join("Documents").join(vault_name);
    log::warn!(
        "Vault not found, using default path: {}",
        redact_path(&default_path)
    );
    Ok(default_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_filename() {
        let filename = generate_filename("screenshot-YYYY-MM-DD-HHmmss");
        assert!(filename.contains("-"));
        assert!(filename.ends_with(".jpg"));
    }

    #[test]
    fn test_is_supported_image() {
        assert!(is_supported_image(Path::new("test.png")));
        assert!(is_supported_image(Path::new("test.jpg")));
        assert!(is_supported_image(Path::new("test.JPEG")));
        assert!(!is_supported_image(Path::new("test.pdf")));
        assert!(!is_supported_image(Path::new("test.txt")));
    }
}
