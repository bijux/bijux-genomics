//! Pipeline profiles across FASTQ, BAM, and cross-domain workflows.

use std::collections::BTreeMap;

use anyhow::{anyhow, Result};
use serde::Serialize;

pub mod bam;
pub mod cross;
pub mod fastq;
pub mod registry;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Domain {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub struct PipelineId(&'static str);

impl PipelineId {
    #[must_use]
    pub const fn new(id: &'static str) -> Self {
        Self(id)
    }

    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for PipelineId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

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
pub struct DefaultsLedgerV1 {
    pub pipeline_id: PipelineId,
    pub tools: BTreeMap<String, String>,
    pub params: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub thresholds: BTreeMap<String, serde_json::Value>,
    pub rationales: BTreeMap<String, String>,
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

pub fn validate_pipeline_id(id: PipelineId) -> Result<()> {
    validate_pipeline_id_str(id.as_str())
}

pub fn validate_pipeline_id_str(id: &str) -> Result<()> {
    let parts: Vec<&str> = id.split("__").collect();
    if parts.len() != 3 {
        return Err(anyhow!("pipeline id must be <graph>__<flavor>__vN"));
    }
    let graph = parts[0];
    let flavor = parts[1];
    let version = parts[2];
    if !graph.contains("-to-") {
        return Err(anyhow!("pipeline id graph must contain '-to-'"));
    }
    if !version.starts_with('v') || version.len() < 2 || !version[1..].chars().all(char::is_numeric)
    {
        return Err(anyhow!("pipeline id version must be v<digits>"));
    }
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
    if !graph.chars().all(allowed) || !flavor.chars().all(allowed) {
        return Err(anyhow!("pipeline id contains invalid characters"));
    }
    Ok(())
}
