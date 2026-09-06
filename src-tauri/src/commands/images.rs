use std::path::Path;

use crate::app_state::AppState;
use crate::image_handler::{self, ProcessedImage};
use crate::vault_paths::MAX_IMAGE_PAYLOAD_BYTES;

#[tauri::command]
pub(crate) async fn save_image(
    file_path: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessedImage, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;

    let result = image_handler::process_dropped_file(&file_path, &settings)?;

    // Register new image in vault index so load_images_batch can resolve it.
    if let Some(ref mut idx) = *state.vault_index.write().await {
        if let Err(e) = idx.add_image(Path::new(&result.saved_path)) {
            log::warn!("Failed to add image to vault index: {}", e);
        }
    }

    Ok(result)
}

#[tauri::command]
pub(crate) async fn save_image_from_bytes(
    bytes_base64: String,
    filename: String,
    state: tauri::State<'_, AppState>,
) -> Result<ProcessedImage, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;

    if bytes_base64.len() > MAX_IMAGE_PAYLOAD_BYTES {
        return Err("Image payload too large".to_string());
    }

    use base64::{engine::general_purpose, Engine as _};
    let bytes = general_purpose::STANDARD
        .decode(&bytes_base64)
        .map_err(|e| format!("Failed to decode base64: {}", e))?;

    let result = image_handler::process_dropped_file_from_bytes(bytes, &filename, &settings)?;

    // Register new image in vault index so load_images_batch can resolve it.
    if let Some(ref mut idx) = *state.vault_index.write().await {
        if let Err(e) = idx.add_image(Path::new(&result.saved_path)) {
            log::warn!("Failed to add image to vault index: {}", e);
        }
    }

    Ok(result)
}
