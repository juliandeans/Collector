use crate::app_state::AppState;
use crate::settings::Settings;

#[tauri::command]
pub(crate) async fn load_settings(state: tauri::State<'_, AppState>) -> Result<Settings, String> {
    let settings = Settings::load()?;

    *state.settings.write().await = settings.clone();

    Ok(settings)
}

#[tauri::command]
pub(crate) async fn get_window_info(
    state: tauri::State<'_, AppState>,
) -> Result<serde_json::Value, String> {
    let settings = state.settings.read().await;
    Ok(serde_json::json!({
        "width": settings.window_width,
        "height": settings.window_height,
        "readerWidth": settings.reader_width,
        "readerHeight": settings.reader_height,
        "borderRadius": settings.border_radius,
        "backgroundColor": settings.overlay_color,
        "fontFamily": settings.font_family,
        "fontSize": settings.font_size,
        "edgeSide": settings.edge_side,
    }))
}

#[tauri::command]
pub(crate) async fn toggle_edge_detection(
    enabled: bool,
    state: tauri::State<'_, AppState>,
) -> Result<(), String> {
    state.edge_detector.set_enabled(enabled).await;

    let mut settings = state.settings.write().await;
    settings.edge_detection_enabled = enabled;
    settings.save()?;

    Ok(())
}
