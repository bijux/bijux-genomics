use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantSeverity {
    Soft,
    Hard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvariantViolationV1 {
    pub code: String,
    pub stage_id: Option<String>,
    pub severity: InvariantSeverity,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InvariantsReportV1 {
    pub schema_version: String,
    pub profile_id: String,
    pub invariants_version: String,
    pub valid: bool,
    pub blocking: bool,
    pub violations: Vec<InvariantViolationV1>,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InvariantsPreset {
    Adna,
    ReferenceAdna,
    VcfMinimal,
}

impl InvariantsPreset {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Adna => "adna",
            Self::ReferenceAdna => "reference_adna",
            Self::VcfMinimal => "vcf_minimal",
        }
    }
}
