//! Pipeline profiles across FASTQ, BAM, and cross-domain workflows.

//! Canonical pipeline profiles and defaults ledger for all domains.

use std::collections::BTreeMap;

use serde::Serialize;
use bijux_core::ids::{StageId, ToolId};
use bijux_domain_fastq::params::{
    DetectAdaptersEffectiveParams, FilterEffectiveParams, MergeEffectiveParams,
    PreprocessEffectiveParams, QcPostEffectiveParams, ScreenEffectiveParams, TrimEffectiveParams,
    ValidateEffectiveParams,
};

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

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", content = "value")]
pub enum DefaultParams {
    FastqValidate(ValidateEffectiveParams),
    FastqDetectAdapters(DetectAdaptersEffectiveParams),
    FastqTrim(TrimEffectiveParams),
    FastqFilter(FilterEffectiveParams),
    FastqQcPost(QcPostEffectiveParams),
    FastqPreprocess(PreprocessEffectiveParams),
    FastqMerge(MergeEffectiveParams),
    FastqScreen(ScreenEffectiveParams),
    Json(serde_json::Value),
}

impl DefaultParams {
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            DefaultParams::FastqValidate(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::FastqDetectAdapters(value) => {
                serde_json::to_value(value).unwrap_or_default()
            }
            DefaultParams::FastqTrim(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::FastqFilter(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::FastqQcPost(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::FastqPreprocess(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::FastqMerge(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::FastqScreen(value) => serde_json::to_value(value).unwrap_or_default(),
            DefaultParams::Json(value) => value.clone(),
        }
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
                assumptions: Vec::new(),
                comparability_implications: Vec::new(),
                citations: Vec::new(),
            };
            if self.defaults.tools.contains_key(stage) {
                tool_provenance.insert(stage.clone(), provenance.clone());
            }
            if self.defaults.params.contains_key(stage) {
                param_provenance.insert(stage.clone(), provenance);
            }
        }
        for stage in self.defaults.tools.keys() {
            tool_provenance
                .entry(stage.clone())
                .or_insert_with(|| DefaultProvenanceV1 {
                    rationale: String::new(),
                    assumptions: Vec::new(),
                    comparability_implications: Vec::new(),
                    citations: Vec::new(),
                });
        }
        for stage in self.defaults.params.keys() {
            param_provenance
                .entry(stage.clone())
                .or_insert_with(|| DefaultProvenanceV1 {
                    rationale: String::new(),
                    assumptions: Vec::new(),
                    comparability_implications: Vec::new(),
                    citations: Vec::new(),
                });
        }
        DefaultsLedgerV1 {
            pipeline_id: self.id.clone(),
            tools: self.defaults.tools.clone(),
            params: self
                .defaults
                .params
                .iter()
                .map(|(stage, value)| (stage.clone(), value.to_json()))
                .collect(),
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
