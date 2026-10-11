use super::models::LocalDamageSmokeMetrics;
use super::models::ParsedDamageToolOutput;
use serde::Deserialize;
use serde::Serialize;
use std::path::PathBuf;
#[derive(Debug, Clone, Deserialize)]
pub(super) struct LocalDamageSmokeReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) sample_id: String,
    pub(super) expectation_matched: bool,
    pub(super) input_bam: String,
    pub(super) method: String,
    pub(super) tools_seen: Vec<String>,
    pub(super) terminal_c_to_t_5p: f64,
    pub(super) terminal_g_to_a_3p: f64,
    pub(super) short_fragment_fraction: f64,
    pub(super) damage_signal: String,
    pub(super) strict_profile_upgraded: bool,
    pub(super) damage_report: String,
    pub(super) terminal_position_metrics: String,
    pub(super) parser_output: String,
    pub(super) advisory_boundary: String,
    pub(super) udg_regime: String,
    pub(super) stage_metrics: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct DamageUnifiedMetricsReport {
    pub(super) canonical: bijux_dna_domain_bam::metrics::DamageMetricsV1,
    pub(super) tools_seen: Vec<String>,
    #[serde(default)]
    pub(super) comparison: Option<bijux_dna_domain_bam::metrics::DamageComparisonV1>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub(super) struct DamageParserOutputReport {
    pub(super) schema_version: String,
    pub(super) stage_id: String,
    pub(super) parsed_tools: Vec<ParsedDamageToolOutput>,
}

#[derive(Debug, Clone)]
pub(super) struct DamageStageProof {
    pub(super) local_smoke_report_path: PathBuf,
    pub(super) local_smoke_report: LocalDamageSmokeReport,
    pub(super) summary: bijux_dna_domain_bam::BamDamageEvidenceV1,
    pub(super) unified_metrics: DamageUnifiedMetricsReport,
    pub(super) parser_output: DamageParserOutputReport,
    pub(super) normalized_metrics: LocalDamageSmokeMetrics,
}
