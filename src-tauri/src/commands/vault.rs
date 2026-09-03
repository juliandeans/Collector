use tokio::time::{timeout, Duration};

use crate::app_state::AppState;
use crate::vault_index;
use crate::vault_paths::{build_image_data_url, resolve_vault_read_path, resolve_vault_write_path};

async fn get_or_build_index(state: &tauri::State<'_, AppState>) -> Result<(), String> {
    let vault_path = {
        let settings = state.settings.read().await;
        settings.vault_path.clone()
    };

    {
        let index = state.vault_index.read().await;
        if let Some(ref idx) = *index {
            // Compare against the original vault path used to build the index.
            // Avoiding fs::canonicalize here prevents potential blocking on iCloud paths.
            if idx.vault_path == vault_path {
                return Ok(());
            }
        }
    }

    log::info!(
        "Building vault index for: {}",
        crate::log_safety::redact_path_str(&vault_path)
    );
    let new_index = vault_index::VaultIndex::build(&vault_path)?;
    let build_duration_ms = new_index.built_at.elapsed().as_millis();
    let _ = new_index.resolve_note("");
    log::info!(
        "Vault index built: {} files in {} ms",
        new_index.file_count,
        build_duration_ms
    );

    *state.vault_index.write().await = Some(new_index);
    Ok(())
}

#[tauri::command]
pub(crate) async fn read_note_file(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await.clone();
    let resolved = resolve_vault_read_path(&settings, &path)?;
    let result = timeout(Duration::from_secs(5), tokio::fs::read_to_string(&resolved)).await;

    match result {
        Ok(Ok(content)) => Ok(content),
        Ok(Err(e)) => Err(format!("Failed to read file: {}", e)),
        Err(_) => Err("File read timed out (iCloud download may be pending)".to_string()),
    }
}

#[tauri::command]
pub(crate) async fn write_note_file(
    path: String,
    content: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    let resolved = resolve_vault_write_path(&settings, &path, true)?;
    crate::fs_util::atomic_write_text(&resolved, &content)
        .map_err(|e| format!("Failed to write file: {}", e))
}

#[tauri::command]
pub(crate) async fn load_image_data_url(
    path: String,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    get_or_build_index(&state).await?;

    let resolved = {
        let index = state.vault_index.read().await;
        let idx = index
            .as_ref()
            .ok_or_else(|| "Vault index not available".to_string())?;

        idx.resolve_image(&path)
            .cloned()
            .ok_or_else(|| format!("Image not found: {}", path))?
    };

    build_image_data_url(&resolved)
}

#[tauri::command]
pub(crate) async fn load_images_batch(
    paths: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<(String, String)>, String> {
    get_or_build_index(&state).await?;

    let index = state.vault_index.read().await;
    let idx = index
        .as_ref()
        .ok_or_else(|| "Vault index not available".to_string())?;

    let mut results = Vec::new();

    for path in paths {
        let clean = path.split('|').next().unwrap_or("").trim().to_string();
        if clean.is_empty() {
            continue;
        }

        match idx.resolve_image(&clean) {
            Some(resolved) => {
                if let Ok(data_url) = build_image_data_url(resolved) {
                    results.push((clean, data_url));
                }
            }
            None => {}
        }
    }

    Ok(results)
}

#[tauri::command]
pub(crate) async fn list_vault_notes(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<vault_index::NoteEntry>, String> {
    get_or_build_index(&state).await?;

    let index = state.vault_index.read().await;
    let idx = index
        .as_ref()
        .ok_or_else(|| "Vault index not available".to_string())?;

    Ok(idx.all_notes())
}

#[tauri::command]
pub(crate) async fn reindex_vault(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    let vault_path = {
        let settings = state.settings.read().await;
        settings.vault_path.clone()
    };

    log::info!("Manual vault reindex requested");
    let new_index = vault_index::VaultIndex::build(&vault_path)?;
    let count = new_index.file_count;
    let build_duration_ms = new_index.built_at.elapsed().as_millis();
    let _ = new_index.resolve_note("");
    *state.vault_index.write().await = Some(new_index);
    log::info!(
        "Vault reindex complete: {} files in {} ms",
        count,
        build_duration_ms
    );

    Ok(count)
}
