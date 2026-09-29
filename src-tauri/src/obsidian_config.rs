//! Obsidian editor settings Collector mirrors (read from `<vault>/.obsidian/app.json`).

use serde::Serialize;

/// How Obsidian indents: a tab, or 4 spaces when `use_tab` is false
/// ("Indent using tabs" off). `tabSize` only sets the visual tab width and is
/// deliberately not read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct ObsidianIndent {
    pub use_tab: bool,
}

impl Default for ObsidianIndent {
    /// Obsidian's own default (`useTab: true`).
    fn default() -> Self {
        Self { use_tab: true }
    }
}

/// Parses Obsidian's `app.json`. A missing key, a wrong type or invalid JSON
/// fall back to Obsidian's default.
pub fn parse_obsidian_indent(json: &str) -> ObsidianIndent {
    let defaults = ObsidianIndent::default();
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else {
        return defaults;
    };

    let use_tab = value
        .get("useTab")
        .and_then(|v| v.as_bool())
        .unwrap_or(defaults.use_tab);

    ObsidianIndent { use_tab }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_object_uses_obsidian_default() {
        assert_eq!(parse_obsidian_indent("{}"), ObsidianIndent::default());
    }

    #[test]
    fn reads_spaces_setting() {
        assert_eq!(
            parse_obsidian_indent(r#"{"useTab": false}"#),
            ObsidianIndent { use_tab: false }
        );
    }

    #[test]
    fn invalid_json_uses_default() {
        assert_eq!(parse_obsidian_indent("not json"), ObsidianIndent::default());
        assert_eq!(parse_obsidian_indent(""), ObsidianIndent::default());
    }

    #[test]
    fn wrong_type_uses_default() {
        assert_eq!(
            parse_obsidian_indent(r#"{"useTab": "no"}"#),
            ObsidianIndent::default()
        );
    }

    #[test]
    fn other_keys_are_ignored() {
        assert_eq!(
            parse_obsidian_indent(r#"{"vimMode": true, "tabSize": 2, "useTab": false}"#),
            ObsidianIndent { use_tab: false }
        );
    }
}
