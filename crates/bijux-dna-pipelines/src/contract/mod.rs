//! Shared pipeline profile models.

use std::collections::BTreeMap;

use bijux_dna_core::ids::{LibraryModel, StageId, ToolId};
use serde::{Deserialize, Serialize};

use crate::DefaultParams;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Domain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StabilityTier {
    Stable,
    Beta,
    Experimental,
}

impl StabilityTier {
    #[must_use]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Beta => "beta",
            Self::Experimental => "experimental",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ArtifactType {
    FastqReads,
    Bam,
    ReportJson,
    RunManifestJson,
    StageSummariesJson,
    MetricsBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum MetricsBundle {
    FastqCore,
    BamCore,
    BamAdna,
    VcfCore,
    CrossHandoff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReportSection {
    Fastq,
    Bam,
    Vcf,
    Cross,
    Handoff,
    PipelineDefaults,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EffectiveDefaults {
    pub tools: BTreeMap<StageId, ToolId>,
    pub params: BTreeMap<StageId, DefaultParams>,
    pub rationales: BTreeMap<StageId, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineProfile {
    pub id: crate::PipelineId,
    pub description: &'static str,
    pub stability: StabilityTier,
    pub input_domains: Vec<Domain>,
    pub output_domains: Vec<Domain>,
    pub defaults: EffectiveDefaults,
    pub defaults_ledger_ref: &'static str,
    pub invariants_preset: Option<InvariantsPreset>,
    pub library_model: LibraryModel,
    pub capabilities: PipelineCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineCapabilities {
    pub input_domains: Vec<Domain>,
    pub output_domains: Vec<Domain>,
    pub input_artifacts: Vec<ArtifactType>,
    pub output_artifacts: Vec<ArtifactType>,
    pub required_inputs: Vec<&'static str>,
    pub produces_outputs: Vec<&'static str>,
    pub report_sections: Vec<&'static str>,
    pub required_report_sections: Vec<ReportSection>,
    pub required_metrics_bundles: Vec<MetricsBundle>,
    pub required_stages: Vec<String>,
    pub required_metrics: Vec<&'static str>,
    pub required_artifacts: Vec<&'static str>,
    pub supports_benchmarks: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineContract {
    pub pipeline_id: crate::PipelineId,
    pub required_stages: Vec<String>,
    pub required_artifacts: Vec<String>,
    pub required_metrics_bundles: Vec<MetricsBundle>,
    pub required_report_sections: Vec<ReportSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProfileManifestV1 {
    pub schema_version: &'static str,
    pub pipeline_id: String,
    pub invariants_preset: Option<String>,
    pub library_model: LibraryModel,
    pub stage_list: Vec<String>,
    pub tool_ids: BTreeMap<String, String>,
    pub param_hashes: BTreeMap<String, String>,
    pub schema_versions: BTreeMap<String, String>,
}

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
