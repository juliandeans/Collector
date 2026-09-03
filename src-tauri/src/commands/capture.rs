use tauri::{AppHandle, Manager};

use crate::app_state::AppState;
use crate::capture;
use crate::vault_paths::resolve_vault_write_path;

const SAVE_AS_NOTE_HIDE_DELAY_MS: u64 = 50;

#[tauri::command]
pub(crate) async fn save_as_note(
    content: String,
    title: Option<String>,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;

    let relative_path = match title.as_deref().filter(|t| !t.trim().is_empty()) {
        Some(t) => capture::build_note_relative_path_from_title(t, &settings),
        None => capture::build_note_relative_path(&settings),
    };

    let resolved = resolve_vault_write_path(&settings, &relative_path, true)?;
    let filename = resolved
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "Invalid note filename".to_string())?
        .to_string();

    let result = capture::save_note_at_path(&content.trim(), &resolved, &filename, &settings)?;

    state.edge_detector.set_capture_open(false).await;

    if let Some(window) = app.get_webview_window("capture") {
        crate::warn_if_failed(window.hide(), "Failed to hide capture window");
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(
        SAVE_AS_NOTE_HIDE_DELAY_MS,
    ))
    .await;

    Ok(result.message)
}

fn daily_note_folder_has_date_tokens(folder: &str) -> bool {
    folder.contains("YYYY") || folder.contains("MM") || folder.contains("DD")
}

#[tauri::command]
pub(crate) async fn append_to_daily_note(
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;
    if settings.daily_note_folder.trim().is_empty() {
        return Err(
            "Daily Note folder is not configured. Please set it in Settings.".to_string(),
        );
    }
    let daily_path = capture::build_daily_note_path(&settings);
    let resolved = resolve_vault_write_path(&settings, &daily_path, true)?;

    capture::append_to_daily_note(&text, &resolved, &settings).await?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn append_to_note(
    path: String,
    text: String,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;
    let resolved = resolve_vault_write_path(&settings, &path, false)?;

    capture::append_to_note(&text, &resolved, &settings)?;

    Ok(())
}

#[tauri::command]
pub(crate) async fn get_daily_note_path(
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let settings = state.settings.read().await.clone();
    settings.validate()?;
    if settings.daily_note_folder.trim().is_empty() {
        return Err(
            "Daily Note folder is not configured. Please set it in Settings.".to_string(),
        );
    }
    let create_parent = daily_note_folder_has_date_tokens(&settings.daily_note_folder);
    let daily_path = capture::build_daily_note_path(&settings);
    let file_path = resolve_vault_write_path(&settings, &daily_path, create_parent)?;
    Ok(file_path.to_string_lossy().to_string())
}
