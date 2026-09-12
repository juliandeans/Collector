use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PinnedNote {
    pub path: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub icon: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum PinnedNoteInput {
    LegacyPath(String),
    Structured(PinnedNote),
}

pub(super) fn deserialize_pinned_notes<'de, D>(deserializer: D) -> Result<Vec<PinnedNote>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries = Vec::<PinnedNoteInput>::deserialize(deserializer)?;

    Ok(entries
        .into_iter()
        .filter_map(|entry| match entry {
            PinnedNoteInput::LegacyPath(path) => {
                let trimmed = path.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(PinnedNote {
                        path: trimmed,
                        label: String::new(),
                        icon: String::new(),
                    })
                }
            }
            PinnedNoteInput::Structured(note) => {
                if note.path.trim().is_empty() {
                    None
                } else {
                    Some(PinnedNote {
                        path: note.path.trim().to_string(),
                        label: note.label.trim().to_string(),
                        icon: note.icon.trim().to_string(),
                    })
                }
            }
        })
        .collect())
}
