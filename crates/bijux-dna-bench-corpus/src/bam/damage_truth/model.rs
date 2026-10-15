//! Declared aDNA damage fixture records and read-only validation observations.
use std::path::PathBuf;

use bijux_dna_domain_bam::{params::UdgModel, BamAdnaDamageTruthSummaryV1};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaDamageTruthManifest {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) description: String,
    pub(super) expected_path: PathBuf,
    pub(super) source_paths: Vec<PathBuf>,
    pub(super) cases: Vec<AdnaDamageTruthCase>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaDamageTruthCase {
    pub(super) case_id: String,
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: PathBuf,
    pub(super) reference_path: PathBuf,
    pub(super) udg_model: UdgModel,
    pub(super) strict_profile: bool,
    pub(super) terminal_c_to_t_5p: f64,
    pub(super) terminal_g_to_a_3p: f64,
}

#[derive(Debug, Clone, Serialize)]
pub struct AdnaDamageTruthValidationReport {
    pub schema_version: &'static str,
    pub fixture_id: String,
    pub manifest_path: String,
    pub expected_path: String,
    pub validated_case_count: usize,
    pub validated_insufficient_case_count: usize,
    pub terminal_pattern_classes: Vec<String>,
    pub checked_cases: Vec<String>,
    pub valid: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaDamageTruthBundle {
    pub(super) schema_version: String,
    pub(super) fixture_id: String,
    pub(super) truths: Vec<AdnaDamageSampleTruth>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub(super) struct AdnaDamageSampleTruth {
    pub(super) case_id: String,
    pub(super) sample_id: String,
    pub(super) cohort: String,
    pub(super) alignment_path: String,
    pub(super) reference_path: String,
    pub(super) summary: BamAdnaDamageTruthSummaryV1,
}
