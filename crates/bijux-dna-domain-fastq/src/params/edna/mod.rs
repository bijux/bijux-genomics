use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::params::PairedMode;

pub const EDNA_SCHEMA_VERSION: &str = "v1";
pub const DEFAULT_OTU_IDENTITY_THRESHOLD: f64 = 0.97;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct PrimerNormalizationEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub threads: Option<u32>,
    pub orientation_policy: String,
    pub primer_set_id: String,
    pub marker_id: Option<String>,
    pub primer_fasta: Option<String>,
    pub max_mismatch_rate: f64,
    pub min_overlap_bp: u32,
    pub strict_5p_anchor: bool,
    pub allow_iupac_codes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ChimeraDetectionEffectiveParams {
    pub method: String,
    pub detection_scope: String,
    pub input_layout: String,
    pub threads: u32,
    pub report_artifact: String,
    pub metrics_artifact: String,
    pub chimera_sequence_artifact: String,
    pub raw_backend_report_artifact: String,
    pub raw_backend_report_format: String,
    pub chimera_removed_definition: String,
    pub fallback_behavior: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AsvInferenceEffectiveParams {
    pub schema_version: String,
    pub paired_mode: PairedMode,
    pub denoising_method: String,
    pub pooling_mode: String,
    pub chimera_policy: String,
    pub threads: Option<u32>,
    pub requires_r_runtime: bool,
    pub output_table_kind: String,
    pub report_artifact: String,
    pub raw_backend_report_artifact: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OtuClusteringEffectiveParams {
    pub schema_version: String,
    pub identity_threshold: f64,
    pub threads: u32,
    pub output_table_kind: String,
    pub report_artifact: String,
    pub raw_backend_report_artifact: Option<String>,
    pub raw_backend_report_format: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AbundanceNormalizationEffectiveParams {
    pub schema_version: String,
    pub method: String,
    pub expected_columns: Vec<String>,
    pub input_value_column: String,
    pub normalized_value_column: String,
    pub compositional_rule: String,
    pub scale_factor: Option<f64>,
    pub report_artifact: String,
}
