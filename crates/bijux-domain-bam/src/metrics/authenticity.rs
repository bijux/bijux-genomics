use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::LibraryType;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityEvidenceV1 {
    pub damage_high: bool,
    pub fragments_short: bool,
    pub mapq_low_with_damage: bool,
}

impl AuthenticityEvidenceV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            damage_high: false,
            fragments_short: false,
            mapq_low_with_damage: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LibraryTypeInferenceV1 {
    pub inferred: LibraryType,
    pub confidence: f64,
    pub rationale: String,
    #[serde(default)]
    pub declared: Option<LibraryType>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TrimSuggestionV1 {
    pub trim_5p: u8,
    pub trim_3p: u8,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthenticityScoreV1 {
    pub score: f64,
    pub confidence: f64,
    pub evidence: AuthenticityEvidenceV1,
    #[serde(default)]
    pub library_type_inference: Option<LibraryTypeInferenceV1>,
    #[serde(default)]
    pub trim_suggestion: Option<TrimSuggestionV1>,
    #[serde(default)]
    pub rationale: Vec<String>,
}

impl AuthenticityScoreV1 {
    #[must_use]
    pub fn empty() -> Self {
        Self {
            score: 0.0,
            confidence: 0.0,
            evidence: AuthenticityEvidenceV1::empty(),
            library_type_inference: None,
            trim_suggestion: None,
            rationale: Vec::new(),
        }
    }
}

impl Default for AuthenticityScoreV1 {
    fn default() -> Self {
        Self::empty()
    }
}
