use serde::{Deserialize, Deserializer, Serialize};

use super::defaults::{default_rule_match, default_rule_position, default_true};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CaptureRuleConditionKnown {
    Contains {
        value: String,
        #[serde(rename = "caseSensitive")]
        case_sensitive: bool,
    },
    Hashtag {
        value: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CaptureRuleConditionInput {
    Known(CaptureRuleConditionKnown),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureRuleCondition {
    Contains { value: String, case_sensitive: bool },
    Hashtag { value: String },
    Unknown(serde_json::Value),
}

impl From<CaptureRuleConditionKnown> for CaptureRuleCondition {
    fn from(value: CaptureRuleConditionKnown) -> Self {
        match value {
            CaptureRuleConditionKnown::Contains {
                value,
                case_sensitive,
            } => Self::Contains {
                value,
                case_sensitive,
            },
            CaptureRuleConditionKnown::Hashtag { value } => Self::Hashtag { value },
        }
    }
}

impl Serialize for CaptureRuleCondition {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Contains {
                value,
                case_sensitive,
            } => CaptureRuleConditionKnown::Contains {
                value: value.clone(),
                case_sensitive: *case_sensitive,
            }
            .serialize(serializer),
            Self::Hashtag { value } => CaptureRuleConditionKnown::Hashtag {
                value: value.clone(),
            }
            .serialize(serializer),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for CaptureRuleCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match CaptureRuleConditionInput::deserialize(deserializer)? {
            CaptureRuleConditionInput::Known(value) => Ok(value.into()),
            CaptureRuleConditionInput::Unknown(value) => Ok(Self::Unknown(value)),
        }
    }
}

fn deserialize_capture_rule_conditions<'de, D>(
    deserializer: D,
) -> Result<Vec<CaptureRuleCondition>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries = Vec::<CaptureRuleConditionInput>::deserialize(deserializer)?;

    Ok(entries
        .into_iter()
        .map(|entry| match entry {
            CaptureRuleConditionInput::Known(value) => value.into(),
            CaptureRuleConditionInput::Unknown(value) => CaptureRuleCondition::Unknown(value),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum CaptureRuleActionKnown {
    Append {
        note: String,
        #[serde(default)]
        heading: String,
        #[serde(default = "default_rule_position")]
        position: String,
        #[serde(default, rename = "createHeadingIfMissing")]
        create_heading_if_missing: bool,
        #[serde(default = "default_true", rename = "stripTrigger")]
        strip_trigger: bool,
        #[serde(default = "default_true", rename = "showTimestamp")]
        show_timestamp: bool,
    },
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum CaptureRuleActionInput {
    Known(CaptureRuleActionKnown),
    Unknown(serde_json::Value),
}

#[derive(Debug, Clone, PartialEq)]
pub enum CaptureRuleAction {
    Append {
        note: String,
        heading: String,
        position: String,
        create_heading_if_missing: bool,
        strip_trigger: bool,
        show_timestamp: bool,
    },
    Unknown(serde_json::Value),
}

impl From<CaptureRuleActionKnown> for CaptureRuleAction {
    fn from(value: CaptureRuleActionKnown) -> Self {
        match value {
            CaptureRuleActionKnown::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            } => Self::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            },
        }
    }
}

impl Serialize for CaptureRuleAction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            } => CaptureRuleActionKnown::Append {
                note: note.clone(),
                heading: heading.clone(),
                position: position.clone(),
                create_heading_if_missing: *create_heading_if_missing,
                strip_trigger: *strip_trigger,
                show_timestamp: *show_timestamp,
            }
            .serialize(serializer),
            Self::Unknown(value) => value.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for CaptureRuleAction {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match CaptureRuleActionInput::deserialize(deserializer)? {
            CaptureRuleActionInput::Known(value) => Ok(value.into()),
            CaptureRuleActionInput::Unknown(value) => Ok(Self::Unknown(value)),
        }
    }
}

fn deserialize_capture_rule_actions<'de, D>(
    deserializer: D,
) -> Result<Vec<CaptureRuleAction>, D::Error>
where
    D: Deserializer<'de>,
{
    let entries = Vec::<CaptureRuleActionInput>::deserialize(deserializer)?;

    Ok(entries
        .into_iter()
        .map(|entry| match entry {
            CaptureRuleActionInput::Known(value) => value.into(),
            CaptureRuleActionInput::Unknown(value) => CaptureRuleAction::Unknown(value),
        })
        .collect())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureRule {
    // id/name brauchen serde(default): ein fehlendes Feld darf nie die
    // komplette Settings-Deserialisierung killen — persistence.rs schreibt
    // sonst bei einem Parse-Fehler Defaults ueber die gesamte config.json.
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub name: String,
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default, deserialize_with = "deserialize_capture_rule_conditions")]
    pub conditions: Vec<CaptureRuleCondition>,
    #[serde(rename = "match", default = "default_rule_match")]
    pub match_mode: String,
    #[serde(default, deserialize_with = "deserialize_capture_rule_actions")]
    pub actions: Vec<CaptureRuleAction>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_trigger_defaults_to_true() {
        let json = r###"{
            "id": "r1",
            "name": "Test",
            "conditions": [{ "type": "hashtag", "value": "#heute" }],
            "match": "any",
            "actions": [{ "type": "append", "note": "Liste.md" }]
        }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        let action = match &rule.actions[0] {
            CaptureRuleAction::Append { strip_trigger, .. } => *strip_trigger,
            _ => panic!("expected Append action"),
        };
        assert!(action, "stripTrigger must default to true");
    }

    #[test]
    fn camel_case_keys_roundtrip() {
        let json = r###"{
            "id": "r1",
            "name": "Test",
            "conditions": [{ "type": "contains", "value": "x", "caseSensitive": true }],
            "match": "all",
            "actions": [{
                "type": "append",
                "note": "Liste.md",
                "heading": "Section Heute",
                "position": "top",
                "createHeadingIfMissing": true,
                "stripTrigger": false
            }]
        }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        let serialized = serde_json::to_string(&rule).unwrap();

        // camelCase Keys kommen beim Re-Serialize wieder heraus
        assert!(serialized.contains("\"caseSensitive\":true"));
        assert!(serialized.contains("\"createHeadingIfMissing\":true"));
        assert!(serialized.contains("\"stripTrigger\":false"));

        let round: CaptureRule = serde_json::from_str(&serialized).unwrap();
        match &round.actions[0] {
            CaptureRuleAction::Append {
                note,
                heading,
                position,
                create_heading_if_missing,
                strip_trigger,
                show_timestamp,
            } => {
                assert_eq!(note, "Liste.md");
                assert_eq!(heading, "Section Heute");
                assert_eq!(position, "top");
                assert!(*create_heading_if_missing);
                assert!(!*strip_trigger);
                assert!(*show_timestamp, "showTimestamp must default to true");
            }
            _ => panic!("expected Append action"),
        }
        assert_eq!(round.match_mode, "all");
        assert_eq!(
            round.conditions,
            vec![CaptureRuleCondition::Contains {
                value: "x".to_string(),
                case_sensitive: true,
            }]
        );
    }

    #[test]
    fn unknown_condition_and_action_survive_roundtrip_losslessly() {
        let json = r###"{
            "id": "r1",
            "name": "Future",
            "conditions": [
                { "type": "time_between", "value": "21:00", "extra": 42 },
                { "type": "hashtag", "value": "#heute" }
            ],
            "match": "any",
            "actions": [
                { "type": "frontmatter", "key": "tags", "value": "heute" },
                { "type": "append", "note": "Liste.md" }
            ]
        }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();

        // Unknown bleibt als vollstaendiges Objekt erhalten (nicht Value::Null)
        assert_eq!(rule.conditions.len(), 2);
        match &rule.conditions[0] {
            CaptureRuleCondition::Unknown(v) => {
                assert_eq!(v["type"], "time_between");
                assert_eq!(v["extra"], 42);
            }
            other => panic!("expected Unknown condition, got {:?}", other),
        }
        assert_eq!(rule.actions.len(), 2);
        match &rule.actions[0] {
            CaptureRuleAction::Unknown(v) => {
                assert_eq!(v["type"], "frontmatter");
                assert_eq!(v["value"], "heute");
            }
            other => panic!("expected Unknown action, got {:?}", other),
        }

        // Roundtrip: Unknown-Payload geht verlustfrei durch Re-Serialize
        let serialized = serde_json::to_string(&rule).unwrap();
        assert!(serialized.contains("time_between"));
        assert!(serialized.contains("\"extra\":42"));
        assert!(serialized.contains("frontmatter"));

        let round: CaptureRule = serde_json::from_str(&serialized).unwrap();
        assert!(matches!(
            round.conditions[0],
            CaptureRuleCondition::Unknown(_)
        ));
        assert!(matches!(round.actions[0], CaptureRuleAction::Unknown(_)));
    }

    #[test]
    fn rule_missing_id_and_name_still_deserializes() {
        // Hand-edited or foreign configs may omit id/name; the whole settings
        // file must never fail to deserialize because of it (persistence.rs
        // would otherwise overwrite config.json with defaults).
        let json = r###"{ "conditions": [], "actions": [] }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.id, "");
        assert_eq!(rule.name, "");
        assert!(rule.enabled);
    }

    #[test]
    fn match_mode_defaults_to_any_and_renames_to_match() {
        let json = r###"{ "id": "r1", "name": "T", "conditions": [], "actions": [] }"###;
        let rule: CaptureRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.match_mode, "any");

        let serialized = serde_json::to_string(&rule).unwrap();
        assert!(serialized.contains("\"match\":\"any\""));
    }
}
