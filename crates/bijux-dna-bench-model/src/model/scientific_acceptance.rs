//! Persisted scientific acceptance threshold configuration and readiness reports.

use std::collections::BTreeMap;

use schemars::{json_schema, JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};

use crate::{
    SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1, SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificAcceptanceDirection {
    ExactMatchPreferred,
    HigherIsBetter,
    LowerIsBetter,
    Minimum,
    Maximum,
    Range,
    StructuredMatch,
}

impl ScientificAcceptanceDirection {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ExactMatchPreferred => "exact_match_preferred",
            Self::HigherIsBetter => "higher_is_better",
            Self::LowerIsBetter => "lower_is_better",
            Self::Minimum => "minimum",
            Self::Maximum => "maximum",
            Self::Range => "range",
            Self::StructuredMatch => "structured_match",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificAcceptanceToleranceKind {
    RelativeFraction,
    AbsoluteDelta,
    ExactMatch,
    NormalizedSetOverlap,
    NormalizedRecordOverlap,
}

impl ScientificAcceptanceToleranceKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RelativeFraction => "relative_fraction",
            Self::AbsoluteDelta => "absolute_delta",
            Self::ExactMatch => "exact_match",
            Self::NormalizedSetOverlap => "normalized_set_overlap",
            Self::NormalizedRecordOverlap => "normalized_record_overlap",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificAcceptancePassRule {
    MatchReference,
    MeetOrExceedReference,
    NotExceedReference,
    RemainWithinReferenceRange,
    MatchReferenceStructure,
}

impl ScientificAcceptancePassRule {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MatchReference => "match_reference",
            Self::MeetOrExceedReference => "meet_or_exceed_reference",
            Self::NotExceedReference => "not_exceed_reference",
            Self::RemainWithinReferenceRange => "remain_within_reference_range",
            Self::MatchReferenceStructure => "match_reference_structure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ScientificAcceptanceInsufficiencyBehavior {
    RefuseStageComparison,
    DropMetricFromStage,
    WarnAndExcludeStage,
}

impl ScientificAcceptanceInsufficiencyBehavior {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::RefuseStageComparison => "refuse_stage_comparison",
            Self::DropMetricFromStage => "drop_metric_from_stage",
            Self::WarnAndExcludeStage => "warn_and_exclude_stage",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScientificAcceptanceThresholdRow {
    pub domain: String,
    pub stage_id: String,
    pub metric_id: String,
    pub metric_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,
    pub direction: ScientificAcceptanceDirection,
    pub tolerance_kind: ScientificAcceptanceToleranceKind,
    pub tolerance_value: f64,
    pub pass_rule: ScientificAcceptancePassRule,
    pub insufficiency_behavior: ScientificAcceptanceInsufficiencyBehavior,
    pub required: bool,
    pub declaration_origin: String,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScientificAcceptanceThresholdsConfig {
    #[schemars(schema_with = "config_schema_version")]
    pub schema_version: String,
    pub rows: Vec<ScientificAcceptanceThresholdRow>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScientificAcceptanceThresholdsReport {
    #[schemars(schema_with = "report_schema_version")]
    pub schema_version: String,
    pub config_path: String,
    pub comparable_metric_count: usize,
    pub row_count: usize,
    pub domain_counts: BTreeMap<String, usize>,
    pub direction_counts: BTreeMap<String, usize>,
    pub insufficiency_behavior_counts: BTreeMap<String, usize>,
    pub rows: Vec<ScientificAcceptanceThresholdRow>,
}

#[must_use]
pub fn scientific_acceptance_thresholds_schema() -> Schema {
    schemars::schema_for!(ScientificAcceptanceThresholdsConfig)
}

#[must_use]
pub fn scientific_acceptance_thresholds_report_schema() -> Schema {
    schemars::schema_for!(ScientificAcceptanceThresholdsReport)
}

fn config_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1})
}

fn report_schema_version(_: &mut SchemaGenerator) -> Schema {
    json_schema!({"type": "string", "const": SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1})
}
