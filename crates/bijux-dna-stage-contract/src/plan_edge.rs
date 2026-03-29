use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanEdge {
    pub(crate) from: String,
    pub(crate) to: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) from_output_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) to_input_id: Option<String>,
}

impl PlanEdge {
    #[must_use]
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            from_output_id: None,
            to_input_id: None,
        }
    }

    #[must_use]
    pub fn with_artifact_binding(
        from: impl Into<String>,
        to: impl Into<String>,
        from_output_id: impl Into<String>,
        to_input_id: impl Into<String>,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            from_output_id: Some(from_output_id.into()),
            to_input_id: Some(to_input_id.into()),
        }
    }

    #[must_use]
    pub fn from(&self) -> &str {
        &self.from
    }

    #[must_use]
    pub fn to(&self) -> &str {
        &self.to
    }

    #[must_use]
    pub fn from_output_id(&self) -> Option<&str> {
        self.from_output_id.as_deref()
    }

    #[must_use]
    pub fn to_input_id(&self) -> Option<&str> {
        self.to_input_id.as_deref()
    }
}
