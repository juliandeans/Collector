use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::settings::Settings;

pub(crate) const MAX_IMAGE_PAYLOAD_BYTES: usize = 20 * 1024 * 1024;

pub(crate) fn normalize_path(path: &Path) -> Result<PathBuf, String> {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::Normal(part) => normalized.push(part),
            Component::ParentDir => {
                if !normalized.pop() {
                    return Err("Path traversal is not allowed".to_string());
                }
            }
        }
    }

    Ok(normalized)
}

fn canonical_vault_root(settings: &Settings) -> Result<PathBuf, String> {
    Ok(PathBuf::from(&settings.vault_path))
}

fn ensure_markdown_path(path: &Path) -> Result<(), String> {
    if path.extension().map(|ext| ext == "md").unwrap_or(false) {
        Ok(())
    } else {
        Err("Only Markdown files inside the vault are allowed".to_string())
    }
}

pub(crate) fn resolve_vault_read_path(settings: &Settings, requested_path: &str) -> Result<PathBuf, String> {
    let vault_root = canonical_vault_root(settings)?;
    let normalized = normalize_path(Path::new(requested_path))?;
    let candidate = if normalized.is_absolute() {
        normalized
    } else {
        vault_root.join(normalized)
    };

    ensure_markdown_path(&candidate)?;

    if candidate.starts_with(&vault_root) {
        Ok(candidate)
    } else {
        Err("Requested file is outside the vault".to_string())
    }
}

pub(crate) fn resolve_vault_write_path(
    settings: &Settings,
    requested_path: &str,
    create_parent: bool,
) -> Result<PathBuf, String> {
    let vault_root = canonical_vault_root(settings)?;
    let normalized = normalize_path(Path::new(requested_path))?;
    let candidate = if normalized.is_absolute() {
        normalized
    } else {
        vault_root.join(normalized)
    };

    ensure_markdown_path(&candidate)?;

    if !candidate.starts_with(&vault_root) {
        return Err("Requested file is outside the vault".to_string());
    }

    let parent = candidate
        .parent()
        .ok_or_else(|| "Invalid note path".to_string())?;

    if create_parent {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create note directory: {}", e))?;
    } else if !parent.exists() {
        return Err(
            "Daily Notes folder not found. Please check your folder path in Settings."
                .to_string(),
        );
    }

    let filename = candidate
        .file_name()
        .ok_or_else(|| "Invalid note filename".to_string())?;

    Ok(parent.join(filename))
}

pub(crate) fn build_image_data_url(path: &Path) -> Result<String, String> {
    let file_size = fs::metadata(path)
        .map_err(|e| format!("Failed to inspect image: {}", e))?
        .len() as usize;

    if file_size > MAX_IMAGE_PAYLOAD_BYTES {
        return Err(format!(
            "Image too large to load: {:.1} MB",
            file_size as f64 / (1024.0 * 1024.0)
        ));
    }

    let bytes = fs::read(path).map_err(|e| format!("Failed to read image: {}", e))?;
    let mime = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .as_deref()
    {
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        _ => "image/jpeg",
    };

    use base64::{engine::general_purpose, Engine as _};
    Ok(format!(
        "data:{};base64,{}",
        mime,
        general_purpose::STANDARD.encode(bytes)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_vault_dir() -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("collector-test-{}", suffix));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn normalize_path_rejects_escape() {
        assert!(normalize_path(Path::new("../../secret.md")).is_err());
    }

    #[test]
    fn resolve_vault_read_path_rejects_outside_file() {
        let vault_dir = temp_vault_dir();
        let note_path = vault_dir.join("safe.md");
        fs::write(&note_path, "# Safe").unwrap();

        let settings = Settings {
            vault_path: vault_dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let result = resolve_vault_read_path(&settings, "../safe.md");
        assert!(result.is_err());

        let _ = fs::remove_dir_all(vault_dir);
    }

    #[test]
    fn resolve_vault_write_path_rejects_outside_file() {
        let vault_dir = temp_vault_dir();
        let settings = Settings {
            vault_path: vault_dir.to_string_lossy().to_string(),
            ..Default::default()
        };

        let result = resolve_vault_write_path(&settings, "../outside.md", false);
        assert!(result.is_err());

        let _ = fs::remove_dir_all(vault_dir);
    }
}
