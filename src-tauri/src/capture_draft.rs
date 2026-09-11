use std::fs;
use std::path::PathBuf;

use crate::fs_util::atomic_write_text;

fn draft_path() -> Result<PathBuf, String> {
    let config_dir =
        dirs::config_dir().ok_or_else(|| "Could not find config directory".to_string())?;
    Ok(config_dir.join("collector").join("capture_draft.txt"))
}

/// Whether `content` warrants writing a draft file, or clearing any existing one instead.
fn should_persist(content: &str) -> bool {
    !content.trim().is_empty()
}

pub fn save_draft(content: &str) -> Result<(), String> {
    if !should_persist(content) {
        return clear_draft();
    }

    let path = draft_path()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create draft directory: {}", e))?;
    }

    atomic_write_text(&path, content).map_err(|e| format!("Failed to write draft file: {}", e))
}

pub fn load_draft() -> Result<String, String> {
    let path = draft_path()?;
    if !path.exists() {
        return Ok(String::new());
    }

    fs::read_to_string(&path).map_err(|e| format!("Failed to read draft file: {}", e))
}

pub fn clear_draft() -> Result<(), String> {
    let path = draft_path()?;
    if path.exists() {
        fs::remove_file(&path).map_err(|e| format!("Failed to remove draft file: {}", e))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::should_persist;

    #[test]
    fn should_persist_true_for_non_empty_trimmed_content() {
        assert!(should_persist("hello"));
        assert!(should_persist("  hello  "));
    }

    #[test]
    fn should_persist_false_for_empty_or_whitespace_content() {
        assert!(!should_persist(""));
        assert!(!should_persist("   \n  "));
    }
}
