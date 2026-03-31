use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PlanReasonKind {
    Default,
    Profile,
    Override,
    Fallback,
    Compatibility,
    InputAssessed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PlanDecisionReason {
    pub kind: PlanReasonKind,
    pub summary: String,
    #[serde(default)]
    pub details: serde_json::Value,
}

impl PlanDecisionReason {
    #[must_use]
    pub fn new(kind: PlanReasonKind, summary: impl Into<String>) -> Self {
        Self {
            kind,
            summary: summary.into(),
            details: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}

impl Default for PlanDecisionReason {
    fn default() -> Self {
        Self {
            kind: PlanReasonKind::Default,
            summary: "planner default".to_string(),
            details: serde_json::Value::Object(serde_json::Map::new()),
        }
    }
}
