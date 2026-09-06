#[tauri::command]
pub(crate) async fn open_external_url(url: String) -> Result<(), String> {
    let allowed = ["http://", "https://", "obsidian://"];
    if !allowed
        .iter()
        .any(|prefix| url.to_ascii_lowercase().starts_with(prefix))
    {
        return Err("Only http, https and obsidian links are allowed".to_string());
    }

    open::that(&url).map_err(|e| format!("Failed to open URL: {}", e))?;
    Ok(())
}

fn collect_apps(dir: &std::path::Path, seen: &mut std::collections::HashSet<String>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) == Some("app") {
                if let Some(name) = path.file_stem() {
                    seen.insert(name.to_string_lossy().to_string());
                }
            }
        }
    }
}

#[tauri::command]
pub(crate) async fn list_applications() -> Result<Vec<String>, String> {
    let mut seen = std::collections::HashSet::new();

    collect_apps(std::path::Path::new("/Applications"), &mut seen);
    collect_apps(std::path::Path::new("/System/Applications"), &mut seen);

    if let Ok(home) = std::env::var("HOME") {
        collect_apps(&std::path::Path::new(&home).join("Applications"), &mut seen);
    }

    let mut apps: Vec<String> = seen.into_iter().collect();
    apps.sort();
    Ok(apps)
}

#[tauri::command]
pub(crate) async fn get_running_apps() -> Result<Vec<String>, String> {
    use std::process::Command;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(
            r#"tell application "System Events" to get name of every process where background only is false"#,
        )
        .output()
        .map_err(|e| format!("Failed to get running apps: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let apps = stdout
        .split(',')
        .map(|app| app.trim().to_string())
        .filter(|app| !app.is_empty())
        .collect();

    Ok(apps)
}
