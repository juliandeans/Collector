use tokio::time::{timeout, Duration};

use crate::app_state::AppState;
use crate::note_metadata_cache::NoteMetadataCache;
use crate::obsidian_config::{parse_obsidian_indent, ObsidianIndent};
use crate::vault_index;
use crate::vault_index_refresh::{self, NoteListRefresh};
use crate::vault_paths::{build_image_data_url, resolve_vault_read_path, resolve_vault_write_path};

async fn ensure_index(
    state: &tauri::State<'_, AppState>,
    max_age: Option<Duration>,
) -> Result<(), String> {
    vault_index_refresh::ensure_vault_index(
        &state.vault_index,
        &state.vault_index_rebuild,
        &state.settings,
        max_age,
        vault_index::VaultIndex::build,
    )
    .await
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
    ensure_index(&state, None).await?;

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
    ensure_index(&state, None).await?;

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

        if let Some(resolved) = idx.resolve_image(&clean) {
            if let Ok(data_url) = build_image_data_url(resolved) {
                results.push((clean, data_url));
            }
        }
    }

    Ok(results)
}

#[tauri::command]
pub(crate) async fn list_vault_notes(
    refresh: Option<NoteListRefresh>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<vault_index::NoteEntry>, String> {
    match refresh {
        Some(mode) => {
            vault_index_refresh::refresh_keeping_stale(
                &state.vault_index,
                &state.vault_index_rebuild,
                &state.settings,
                mode.max_age(),
                vault_index::VaultIndex::build,
            )
            .await?
        }
        None => ensure_index(&state, None).await?,
    }

    let index = state.vault_index.read().await;
    let idx = index
        .as_ref()
        .ok_or_else(|| "Vault index not available".to_string())?;

    Ok(idx.all_notes())
}

#[tauri::command]
pub(crate) async fn list_vault_tags(
    state: tauri::State<'_, AppState>,
) -> Result<Vec<String>, String> {
    ensure_index(&state, None).await?;

    let vault_path = {
        let settings = state.settings.read().await;
        settings.vault_path.clone()
    };

    {
        let cache = state.note_metadata.read().await;
        if let Some(cache) = cache.as_ref() {
            if cache.vault_path() == vault_path.as_str() {
                return Ok(cache.tags().to_vec());
            }
        }
    }

    let note_paths = {
        let index = state.vault_index.read().await;
        let idx = index
            .as_ref()
            .ok_or_else(|| "Vault index not available".to_string())?;

        idx.all_note_paths()
    };
    let note_count = note_paths.len();
    let build_vault_path = vault_path.clone();
    let cache = tokio::task::spawn_blocking(move || {
        NoteMetadataCache::build(&build_vault_path, &note_paths)
    })
    .await
    .map_err(|error| format!("Failed to build tag cache: {}", error))?;

    let build_duration_ms = cache.built_at().elapsed().as_millis();
    log::info!(
        "Tag cache built: {} tags from {} notes in {} ms",
        cache.tags().len(),
        note_count,
        build_duration_ms
    );

    let tags = cache.tags().to_vec();
    *state.note_metadata.write().await = Some(cache);

    Ok(tags)
}

#[tauri::command]
pub(crate) async fn reindex_vault(state: tauri::State<'_, AppState>) -> Result<usize, String> {
    log::info!("Manual vault reindex requested");
    ensure_index(&state, Some(Duration::ZERO)).await?;
    *state.note_metadata.write().await = None;

    let count = state
        .vault_index
        .read()
        .await
        .as_ref()
        .map(|index| index.file_count)
        .ok_or_else(|| "Vault index not available".to_string())?;
    log::info!("Vault reindex complete: {} files", count);

    Ok(count)
}

/// Obsidian's list indentation for the configured vault. Never fails: a missing,
/// unreadable or slow (iCloud) `.obsidian/app.json` yields Obsidian's defaults.
#[tauri::command]
pub(crate) async fn get_obsidian_indent(
    state: tauri::State<'_, AppState>,
) -> Result<ObsidianIndent, String> {
    let vault_path = state.settings.read().await.vault_path.clone();
    if vault_path.trim().is_empty() {
        return Ok(ObsidianIndent::default());
    }

    let app_json = std::path::Path::new(&vault_path)
        .join(".obsidian")
        .join("app.json");
    match timeout(Duration::from_secs(5), tokio::fs::read_to_string(&app_json)).await {
        Ok(Ok(content)) => Ok(parse_obsidian_indent(&content)),
        _ => Ok(ObsidianIndent::default()),
    }
}
