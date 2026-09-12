use tauri::AppHandle;

use crate::app_state::AppState;

#[tauri::command]
pub(crate) async fn get_capture_mode_state(
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    Ok(state.capture_mode_controller.is_active())
}

#[tauri::command]
pub(crate) async fn set_capture_mode_enabled(
    enabled: bool,
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    state
        .capture_mode_controller
        .set_enabled(&app, enabled)
        .await
}

#[tauri::command]
pub(crate) async fn toggle_capture_mode(
    app: AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<bool, String> {
    state.capture_mode_controller.toggle(&app).await
}
