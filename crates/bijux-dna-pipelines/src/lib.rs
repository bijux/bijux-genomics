//! Pipeline profiles across FASTQ, BAM, and cross-domain workflows.

//! Canonical pipeline profiles and defaults ledger for all domains.

use std::collections::BTreeMap;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_bam::params::BamEffectiveParams;
use bijux_dna_domain_fastq::params::detect_adapters::DetectAdaptersEffectiveParams;
use bijux_dna_domain_fastq::params::filter::FilterEffectiveParams;
use bijux_dna_domain_fastq::params::merge::MergeEffectiveParams;
use bijux_dna_domain_fastq::params::preprocess::PreprocessEffectiveParams;
use bijux_dna_domain_fastq::params::qc_post::QcPostEffectiveParams;
use bijux_dna_domain_fastq::params::screen::ScreenEffectiveParams;
use bijux_dna_domain_fastq::params::trim::TrimEffectiveParams;
use bijux_dna_domain_fastq::params::validate::ValidateEffectiveParams;
use serde::ser::Serializer;
use serde::{Deserialize, Serialize};

pub mod bam;
pub mod cross;
pub mod defaults;
pub mod fastq;
pub mod registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Domain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

pub const STAGE_CORE_PREPARE_REFERENCE: &str = "core.prepare_reference";
pub const STAGE_CROSS_ALIGN_STUB: &str = "cross.align_stub";

pub use defaults::{DefaultProvenanceV1, DefaultsLedgerV1};
pub use registry::id::{validate_pipeline_id, validate_pipeline_id_str, PipelineId};

pub type PipelineProfileV1 = PipelineProfile;

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
    CrossHandoff,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum ReportSection {
    Fastq,
    Bam,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmptyParams {}

#[derive(Debug, Clone, Deserialize)]
pub enum DefaultParams {
    FastqValidate(ValidateEffectiveParams),
    FastqDetectAdapters(DetectAdaptersEffectiveParams),
    FastqTrim(TrimEffectiveParams),
    FastqFilter(FilterEffectiveParams),
    FastqQcPost(QcPostEffectiveParams),
    FastqPreprocess(PreprocessEffectiveParams),
    FastqMerge(MergeEffectiveParams),
    FastqScreen(ScreenEffectiveParams),
    Bam(BamEffectiveParams),
    Empty(EmptyParams),
}

impl DefaultParams {
    fn encode<T: Serialize>(value: &T, kind: &str) -> serde_json::Value {
        serde_json::to_value(value)
            .unwrap_or_else(|err| panic!("failed to serialize {kind} default params: {err}"))
    }

    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            DefaultParams::FastqValidate(value) => Self::encode(value, "fastq.validate"),
            DefaultParams::FastqDetectAdapters(value) => {
                Self::encode(value, "fastq.detect_adapters")
            }
            DefaultParams::FastqTrim(value) => Self::encode(value, "fastq.trim"),
            DefaultParams::FastqFilter(value) => Self::encode(value, "fastq.filter"),
            DefaultParams::FastqQcPost(value) => Self::encode(value, "fastq.qc_post"),
            DefaultParams::FastqPreprocess(value) => Self::encode(value, "fastq.preprocess"),
            DefaultParams::FastqMerge(value) => Self::encode(value, "fastq.merge"),
            DefaultParams::FastqScreen(value) => Self::encode(value, "fastq.screen"),
            DefaultParams::Bam(value) => match value {
                BamEffectiveParams::Align(inner) => Self::encode(inner, "bam.align"),
                BamEffectiveParams::Validate(inner) => Self::encode(inner, "bam.validate"),
                BamEffectiveParams::QcPre(inner) => Self::encode(inner, "bam.qc_pre"),
                BamEffectiveParams::MappingSummary(inner) => {
                    Self::encode(inner, "bam.mapping_summary")
                }
                BamEffectiveParams::Filter(inner) => Self::encode(inner, "bam.filter"),
                BamEffectiveParams::MapqFilter(inner) => Self::encode(inner, "bam.mapq_filter"),
                BamEffectiveParams::LengthFilter(inner) => Self::encode(inner, "bam.length_filter"),
                BamEffectiveParams::Markdup(inner) => Self::encode(inner, "bam.markdup"),
                BamEffectiveParams::DuplicationMetrics(inner) => {
                    Self::encode(inner, "bam.duplication_metrics")
                }
                BamEffectiveParams::Complexity(inner) => Self::encode(inner, "bam.complexity"),
                BamEffectiveParams::Coverage(inner) => Self::encode(inner, "bam.coverage"),
                BamEffectiveParams::InsertSize(inner) => Self::encode(inner, "bam.insert_size"),
                BamEffectiveParams::GcBias(inner) => Self::encode(inner, "bam.gc_bias"),
                BamEffectiveParams::EndogenousContent(inner) => {
                    Self::encode(inner, "bam.endogenous_content")
                }
                BamEffectiveParams::OverlapCorrection(inner) => {
                    Self::encode(inner, "bam.overlap_correction")
                }
                BamEffectiveParams::Damage(inner) => Self::encode(inner, "bam.damage"),
                BamEffectiveParams::Authenticity(inner) => Self::encode(inner, "bam.authenticity"),
                BamEffectiveParams::Contamination(inner) => {
                    Self::encode(inner, "bam.contamination")
                }
                BamEffectiveParams::Sex(inner) => Self::encode(inner, "bam.sex"),
                BamEffectiveParams::BiasMitigation(inner) => {
                    Self::encode(inner, "bam.bias_mitigation")
                }
                BamEffectiveParams::Recalibration(inner) => {
                    Self::encode(inner, "bam.recalibration")
                }
                BamEffectiveParams::Haplogroups(inner) => Self::encode(inner, "bam.haplogroups"),
                BamEffectiveParams::Genotyping(inner) => Self::encode(inner, "bam.genotyping"),
                BamEffectiveParams::Kinship(inner) => Self::encode(inner, "bam.kinship"),
            },
            DefaultParams::Empty(_) => serde_json::json!({}),
        }
    }
}

impl Serialize for DefaultParams {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.to_json().serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineProfile {
    pub id: PipelineId,
    pub description: &'static str,
    pub stability: StabilityTier,
    pub input_domains: Vec<Domain>,
    pub output_domains: Vec<Domain>,
    pub defaults: EffectiveDefaults,
    pub defaults_ledger_ref: &'static str,
    pub invariants_preset: Option<&'static str>,
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
    pub required_stages: Vec<&'static str>,
    pub required_metrics: Vec<&'static str>,
    pub required_artifacts: Vec<&'static str>,
    pub supports_benchmarks: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineContract {
    pub pipeline_id: PipelineId,
    pub required_stages: Vec<String>,
    pub required_artifacts: Vec<String>,
    pub required_metrics_bundles: Vec<MetricsBundle>,
    pub required_report_sections: Vec<ReportSection>,
}

impl PipelineProfile {
    #[must_use]
    pub fn defaults_ledger(&self) -> DefaultsLedgerV1 {
        let mut tool_provenance = BTreeMap::new();
        let mut param_provenance = BTreeMap::new();
        for (stage, rationale) in &self.defaults.rationales {
            let provenance = DefaultProvenanceV1 {
                rationale: rationale.clone(),
                assumptions: vec![
                    "defaults chosen for pre-HPC deterministic baseline comparisons".to_string(),
                ],
                comparability_implications: vec![
                    "changing this default can shift cross-run comparability baselines".to_string(),
                ],
                citations: vec!["docs/20-science/fastq/GOLD_PIPELINE_SPEC.md".to_string()],
            };
            if self.defaults.tools.contains_key(stage) {
                tool_provenance.insert(stage.clone(), provenance.clone());
            }
            if self.defaults.params.contains_key(stage) {
                param_provenance.insert(stage.clone(), provenance);
            }
        }
        for stage in self.defaults.tools.keys() {
            assert!(
                tool_provenance.contains_key(stage),
                "missing tool provenance rationale for stage {} in pipeline {}",
                stage.as_str(),
                self.id.as_str()
            );
        }
        for stage in self.defaults.params.keys() {
            assert!(
                param_provenance.contains_key(stage),
                "missing parameter provenance rationale for stage {} in pipeline {}",
                stage.as_str(),
                self.id.as_str()
            );
        }
        DefaultsLedgerV1 {
            pipeline_id: self.id.clone(),
            tools: self.defaults.tools.clone(),
            params: self.defaults.params.clone(),
            thresholds: BTreeMap::new(),
            tool_provenance,
            param_provenance,
            assumptions: Vec::new(),
            citations: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn contract(&self) -> PipelineContract {
        PipelineContract {
            pipeline_id: self.id.clone(),
            required_stages: self
                .capabilities
                .required_stages
                .iter()
                .map(|stage| (*stage).to_string())
                .collect(),
            required_artifacts: self
                .capabilities
                .required_artifacts
                .iter()
                .map(|artifact| (*artifact).to_string())
                .collect(),
            required_metrics_bundles: self.capabilities.required_metrics_bundles.clone(),
            required_report_sections: self.capabilities.required_report_sections.clone(),
        }
    }
}

pub fn merge_effective_defaults(
    profile: &EffectiveDefaults,
    config: Option<&EffectiveDefaults>,
    cli: Option<&EffectiveDefaults>,
    api: Option<&EffectiveDefaults>,
) -> anyhow::Result<EffectiveDefaults> {
    let mut merged = profile.clone();
    if let Some(config) = config {
        apply_overrides(&mut merged, profile, config, "config override")?;
    }
    if let Some(cli) = cli {
        apply_overrides(&mut merged, profile, cli, "cli override")?;
    }
    if let Some(api) = api {
        apply_overrides(&mut merged, profile, api, "api override")?;
    }
    Ok(merged)
}

fn apply_overrides(
    merged: &mut EffectiveDefaults,
    profile: &EffectiveDefaults,
    overrides: &EffectiveDefaults,
    rationale: &str,
) -> anyhow::Result<()> {
    for (stage, tool) in &overrides.tools {
        ensure_stage_known(profile, stage, "tool override")?;
        merged.tools.insert(stage.clone(), tool.clone());
        merged
            .rationales
            .insert(stage.clone(), rationale.to_string());
    }
    for (stage, params) in &overrides.params {
        ensure_stage_known(profile, stage, "params override")?;
        merged.params.insert(stage.clone(), params.clone());
        merged
            .rationales
            .insert(stage.clone(), rationale.to_string());
    }
    Ok(())
}

fn ensure_stage_known(
    profile: &EffectiveDefaults,
    stage: &StageId,
    context: &str,
) -> anyhow::Result<()> {
    if profile.tools.contains_key(stage) || profile.params.contains_key(stage) {
        return Ok(());
    }
    Err(anyhow::anyhow!(
        "{} references unknown stage {}",
        context,
        stage.as_str()
    ))
}
