use std::path::Path;

pub fn summarize_text_len(text: &str) -> usize {
    text.chars().count()
}

pub fn summarize_bytes(bytes: usize) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else {
        format!("{}KB", (bytes + 1023) / 1024)
    }
}

pub fn redact_path(path: &Path) -> String {
    let parts: Vec<String> = path
        .components()
        .filter_map(|c| match c {
            std::path::Component::Normal(p) => Some(p.to_string_lossy().to_string()),
            _ => None,
        })
        .collect();

    redact_parts(&parts)
}

pub fn redact_path_str(path: &str) -> String {
    let parts: Vec<&str> = path
        .split(|c| c == '/' || c == '\\')
        .filter(|p| !p.is_empty())
        .collect();

    redact_parts_str(&parts)
}

fn redact_parts(parts: &[String]) -> String {
    if parts.is_empty() {
        return "<unknown>".to_string();
    }
    if parts.len() == 1 {
        return parts[0].clone();
    }
    format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
}

fn redact_parts_str(parts: &[&str]) -> String {
    if parts.is_empty() {
        return "<unknown>".to_string();
    }
    if parts.len() == 1 {
        return parts[0].to_string();
    }
    format!(".../{}/{}", parts[parts.len() - 2], parts[parts.len() - 1])
}
