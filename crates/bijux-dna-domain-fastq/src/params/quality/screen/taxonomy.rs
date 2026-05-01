use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::PairedMode;

pub const SCREEN_TAXONOMY_SCHEMA_VERSION: &str = "bijux.fastq.params.screen_taxonomy.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ScreenEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: u32,
    #[serde(default)]
    pub contaminant_db: Option<String>,
    pub database_catalog_id: String,
    pub database_artifact_id: String,
    #[serde(default)]
    pub database_build_id: Option<String>,
    #[serde(default)]
    pub database_digest: Option<String>,
    #[serde(default)]
    pub database_namespace: Option<String>,
    pub database_scope: TaxonomyDatabaseScope,
    pub classifier: TaxonomyClassifier,
    pub report_format: TaxonomyReportFormat,
    pub assignment_format: TaxonomyAssignmentFormat,
    #[serde(default)]
    pub minimum_confidence: Option<f32>,
    pub emit_unclassified: bool,
    pub interpretation_boundary: TaxonomyInterpretationBoundary,
    #[serde(default)]
    pub truth_conditions: Vec<TaxonomyTruthCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxonomyDatabaseScope {
    ReadScreening,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxonomyClassifier {
    Kraken2,
    KrakenUniq,
    Centrifuge,
    Kaiju,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxonomyReportFormat {
    KrakenReport,
    KrakenUniqReport,
    CentrifugeReport,
    KaijuSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxonomyAssignmentFormat {
    KrakenAssignments,
    KrakenUniqAssignments,
    CentrifugeAssignments,
    KaijuAssignments,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxonomyInterpretationBoundary {
    ScreeningOnly,
    DefinitiveWithGovernedTruth,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaxonomyTruthCondition {
    LockedReferenceDatabase,
    OrthogonalValidationRequired,
    SampleScopeValidated,
    ConfirmatoryAssayRequired,
}

impl ScreenEffectiveParams {
    #[must_use]
    pub fn missing_required_fields(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.schema_version.trim().is_empty() {
            missing.push("schema_version");
        }
        if self.paired_mode == PairedMode::Unknown {
            missing.push("paired_mode");
        }
        if self.threads == 0 {
            missing.push("threads");
        }
        if self.database_artifact_id.trim().is_empty() {
            missing.push("database_artifact_id");
        }
        if self.database_catalog_id.trim().is_empty() {
            missing.push("database_catalog_id");
        }
        missing
    }

    #[must_use]
    pub fn retention_conditions(&self) -> serde_json::Value {
        serde_json::json!({
            "schema_version": self.schema_version,
            "contaminant_db": self.contaminant_db,
            "database_catalog_id": self.database_catalog_id,
            "database_artifact_id": self.database_artifact_id,
            "database_build_id": self.database_build_id,
            "database_digest": self.database_digest,
            "database_namespace": self.database_namespace,
            "database_scope": self.database_scope,
            "paired_mode": self.paired_mode,
            "threads": self.threads,
            "classifier": self.classifier,
            "report_format": self.report_format,
            "assignment_format": self.assignment_format,
            "minimum_confidence": self.minimum_confidence,
            "emit_unclassified": self.emit_unclassified,
            "interpretation_boundary": self.interpretation_boundary,
            "truth_conditions": self.truth_conditions,
        })
    }
}
