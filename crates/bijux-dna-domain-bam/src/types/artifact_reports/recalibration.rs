//! Persisted BAM recalibration reports and schema identities.

use crate::params::BqsrMode;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const BAM_RECALIBRATION_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.recalibration.v1";

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamRecalibrationSummaryV1 {
    pub schema_version: String,
    pub stage_id: String,
    pub input_bam: PathBuf,
    #[serde(default)]
    pub reference_fasta: Option<PathBuf>,
    pub known_sites: Vec<PathBuf>,
    pub requested_mode: BqsrMode,
    pub effective_mode: BqsrMode,
    pub status: String,
    pub reason: String,
    pub coverage_gate: BamRecalibrationCoverageGateV1,
    pub observed_mean_coverage: f64,
    pub observed_breadth_1x: f64,
    pub output_bam_present: bool,
    pub recalibration_report_present: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BamRecalibrationCoverageGateV1 {
    pub min_mean_coverage: f64,
    pub min_breadth_1x: f64,
}
