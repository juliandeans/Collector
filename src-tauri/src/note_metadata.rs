//! Pure extraction of note metadata from Markdown content.
//!
//! This iteration exposes only tag extraction. If another metadata field is
//! added later, the extraction API can be evolved to a metadata struct
//! together with that additional pure extractor.

use std::collections::HashSet;

/// Extract frontmatter and inline tags from a Markdown note.
pub(crate) fn extract_tags(content: &str) -> Vec<String> {
    let (body_start, frontmatter_tags) = parse_frontmatter_tags(content).unwrap_or((0, Vec::new()));

    let mut tags = Vec::new();
    let mut seen = HashSet::new();

    for tag in frontmatter_tags
        .into_iter()
        .chain(scan_inline_tags(&content[body_start..]))
    {
        let tag = normalize_tag(&tag);
        if tag.is_empty() {
            continue;
        }

        let key = tag.to_lowercase();
        if seen.insert(key) {
            tags.push(tag);
        }
    }

    tags.sort_by_key(|left| left.to_lowercase());
    tags
}

/// Parse a leading `---` frontmatter block and return its end offset and tags.
fn parse_frontmatter_tags(content: &str) -> Option<(usize, Vec<String>)> {
    let mut lines = content.split_inclusive('\n');
    let first_line = lines.next()?;

    if trim_line_ending(first_line).trim() != "---" {
        return None;
    }

    let mut frontmatter_lines = Vec::new();
    let mut offset = first_line.len();

    for line in lines {
        if trim_line_ending(line).trim() == "---" {
            let tags = parse_frontmatter_values(&frontmatter_lines);
            return Some((offset + line.len(), tags));
        }

        frontmatter_lines.push(trim_line_ending(line));
        offset += line.len();
    }

    None
}

fn parse_frontmatter_values(lines: &[&str]) -> Vec<String> {
    let mut tags = Vec::new();
    let mut line_index = 0;

    while line_index < lines.len() {
        let line = lines[line_index].trim();
        let Some((key, value)) = line.split_once(':') else {
            line_index += 1;
            continue;
        };

        if key.trim() != "tags" {
            line_index += 1;
            continue;
        }

        let value = value.trim();
        if value.is_empty() {
            line_index += 1;
            while line_index < lines.len() {
                let list_item = lines[line_index].trim();
                if list_item.is_empty() {
                    line_index += 1;
                    continue;
                }

                let Some(item) = list_item.strip_prefix('-') else {
                    break;
                };

                let item = item.trim();
                if !item.is_empty() {
                    tags.push(item.to_string());
                }
                line_index += 1;
            }
            continue;
        }

        let value = value
            .strip_prefix('[')
            .map(|value| value.strip_suffix(']').unwrap_or(value))
            .unwrap_or(value);

        tags.extend(
            value
                .split(',')
                .map(str::trim)
                .filter(|tag| !tag.is_empty())
                .map(str::to_string),
        );
        line_index += 1;
    }

    tags
}

/// Scan non-frontmatter Markdown content while skipping fenced code blocks.
fn scan_inline_tags(content: &str) -> Vec<String> {
    let mut tags = Vec::new();
    let mut in_fenced_code = false;

    for raw_line in content.split_inclusive('\n') {
        let line = trim_line_ending(raw_line);
        if line.trim_start().starts_with("```") {
            in_fenced_code = !in_fenced_code;
            continue;
        }

        if in_fenced_code {
            continue;
        }

        let characters: Vec<(usize, char)> = line.char_indices().collect();
        let mut index = 0;

        while index < characters.len() {
            let (start, character) = characters[index];
            if character != '#' {
                index += 1;
                continue;
            }

            let previous = index.checked_sub(1).map(|previous| characters[previous].1);
            let next_is_tag_character = characters
                .get(index + 1)
                .is_some_and(|(_, next)| is_tag_character(*next));

            if !is_tag_boundary(previous) || !next_is_tag_character {
                index += 1;
                continue;
            }

            let tag_start = start + character.len_utf8();
            let mut end_index = index + 1;
            while end_index < characters.len() && is_tag_character(characters[end_index].1) {
                end_index += 1;
            }

            let tag_end = characters
                .get(end_index)
                .map(|(offset, _)| *offset)
                .unwrap_or(line.len());
            tags.push(line[tag_start..tag_end].to_string());
            index = end_index;
        }
    }

    tags
}

fn is_tag_boundary(previous: Option<char>) -> bool {
    match previous {
        None => true,
        Some(character) => {
            character.is_whitespace()
                || (!character.is_alphanumeric() && character != '_' && character != '#')
        }
    }
}

fn is_tag_character(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '/' | '-')
}

fn normalize_tag(tag: &str) -> String {
    let tag = tag.trim();
    let tag = tag.strip_prefix('#').unwrap_or(tag);

    if tag.len() >= 2
        && ((tag.starts_with('"') && tag.ends_with('"'))
            || (tag.starts_with('\'') && tag.ends_with('\'')))
    {
        tag[1..tag.len() - 1].trim().to_string()
    } else {
        tag.to_string()
    }
}

fn trim_line_ending(line: &str) -> &str {
    let line = line.strip_suffix('\n').unwrap_or(line);
    line.strip_suffix('\r').unwrap_or(line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_inline_tags_at_valid_boundaries() {
        let content = "#alpha (#beta) [#gamma] {#delta}\nword#ignored\n##heading\n# heading";

        assert_eq!(
            extract_tags(content),
            vec!["alpha", "beta", "delta", "gamma"]
        );
    }

    #[test]
    fn extracts_all_frontmatter_list_forms() {
        assert_eq!(
            extract_tags("---\ntags: [alpha, parent/child]\n---\n"),
            vec!["alpha", "parent/child"]
        );
        assert_eq!(
            extract_tags("---\ntags: alpha, parent/child\n---\n"),
            vec!["alpha", "parent/child"]
        );
        assert_eq!(
            extract_tags("---\ntags:\n  - alpha\n  - parent/child\n---\n"),
            vec!["alpha", "parent/child"]
        );
    }

    #[test]
    fn ignores_missing_frontmatter_tags_and_content_without_frontmatter() {
        assert_eq!(
            extract_tags("---\ntitle: Note\n---\n#inline"),
            vec!["inline"]
        );
        assert_eq!(extract_tags("#inline"), vec!["inline"]);
    }

    #[test]
    fn skips_fenced_code_blocks_but_not_inline_code() {
        let content = "outside `#inline`\n```rust\n#fenced\n```\nafter #visible";

        assert_eq!(extract_tags(content), vec!["inline", "visible"]);
    }

    #[test]
    fn preserves_nested_tags_deduplicates_case_insensitively_and_sorts() {
        let content = "---\ntags: [Parent/Child, beta]\n---\n#alpha #parent/child #BETA #gamma";

        assert_eq!(
            extract_tags(content),
            vec!["alpha", "beta", "gamma", "Parent/Child"]
        );
    }

    #[test]
    fn excludes_hashes_after_word_characters_and_other_hashes() {
        let content = "word#no ##heading ###also-no #yes foo/#nested";

        assert_eq!(extract_tags(content), vec!["nested", "yes"]);
    }
}
