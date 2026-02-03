//! Pipeline profiles across FASTQ, BAM, and cross-domain workflows.

//! Canonical pipeline profiles and defaults ledger for all domains.

use std::collections::BTreeMap;

use serde::Serialize;

pub mod bam;
pub mod cross;
pub mod defaults_ledger;
pub mod fastq;
pub mod id;
pub mod registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Domain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

pub use defaults_ledger::DefaultsLedgerV1;
pub use id::{validate_pipeline_id, validate_pipeline_id_str, PipelineId};

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

#[derive(Debug, Clone, Serialize)]
pub struct StageNode {
    pub stage_id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EffectiveDefaults {
    pub tools: BTreeMap<String, String>,
    pub params: BTreeMap<String, serde_json::Value>,
    pub rationales: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineProfile {
    pub id: PipelineId,
    pub description: &'static str,
    pub stability: StabilityTier,
    pub input_domains: Vec<Domain>,
    pub output_domains: Vec<Domain>,
    #[serde(rename = "stage_graph")]
    pub graph: Vec<StageNode>,
    pub defaults: EffectiveDefaults,
    pub defaults_ledger_ref: &'static str,
    pub invariants_preset: Option<&'static str>,
    pub capabilities: PipelineCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineCapabilities {
    pub input_domains: Vec<Domain>,
    pub output_domains: Vec<Domain>,
    pub required_inputs: Vec<&'static str>,
    pub produces_outputs: Vec<&'static str>,
    pub report_sections: Vec<&'static str>,
    pub supports_benchmarking: bool,
}

impl PipelineProfile {
    #[must_use]
    pub fn defaults_ledger(&self) -> DefaultsLedgerV1 {
        DefaultsLedgerV1 {
            pipeline_id: self.id,
            tools: self.defaults.tools.clone(),
            params: self.defaults.params.clone(),
            thresholds: BTreeMap::new(),
            rationales: self.defaults.rationales.clone(),
        }
    }
}

#[must_use]
pub fn merge_effective_defaults(
    profile: &EffectiveDefaults,
    config: Option<&EffectiveDefaults>,
    cli: Option<&EffectiveDefaults>,
) -> EffectiveDefaults {
    let mut merged = profile.clone();
    if let Some(config) = config {
        for (stage, tool) in &config.tools {
            merged.tools.insert(stage.clone(), tool.clone());
            merged
                .rationales
                .insert(stage.clone(), "config override".to_string());
        }
        for (stage, params) in &config.params {
            merged.params.insert(stage.clone(), params.clone());
            merged
                .rationales
                .insert(stage.clone(), "config override".to_string());
        }
    }
    if let Some(cli) = cli {
        for (stage, tool) in &cli.tools {
            merged.tools.insert(stage.clone(), tool.clone());
            merged
                .rationales
                .insert(stage.clone(), "cli override".to_string());
        }
        for (stage, params) in &cli.params {
            merged.params.insert(stage.clone(), params.clone());
            merged
                .rationales
                .insert(stage.clone(), "cli override".to_string());
        }
    }
    merged
}
