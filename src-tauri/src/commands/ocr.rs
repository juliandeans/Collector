use crate::ocr::{ModifierFlags, OcrResult};

#[tauri::command]
pub async fn ocr_clipboard_image() -> Result<OcrResult, String> {
    tokio::task::spawn_blocking(crate::ocr::ocr_clipboard)
        .await
        .map_err(|e| format!("OCR task failed: {e}"))?
}

#[tauri::command]
pub async fn ocr_image_file(path: String) -> Result<OcrResult, String> {
    tokio::task::spawn_blocking(move || crate::ocr::ocr_file(&path))
        .await
        .map_err(|e| format!("OCR task failed: {e}"))?
}

#[tauri::command]
pub async fn ocr_image_bytes(bytes_base64: String) -> Result<OcrResult, String> {
    tokio::task::spawn_blocking(move || crate::ocr::ocr_bytes(&bytes_base64))
        .await
        .map_err(|e| format!("OCR task failed: {e}"))?
}

#[tauri::command]
pub async fn get_modifier_flags() -> Result<ModifierFlags, String> {
    Ok(crate::ocr::get_current_modifier_flags())
}
