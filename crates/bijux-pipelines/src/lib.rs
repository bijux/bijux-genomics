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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum StabilityTier {
    Stable,
    Beta,
    Experimental,
}

#[derive(Debug, Clone, Serialize)]
pub struct StageNode {
    pub stage_id: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct EffectiveDefaults {
    pub tools: BTreeMap<String, String>,
    pub params: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineProfile {
    pub id: PipelineId,
    pub description: &'static str,
    pub stability: StabilityTier,
    pub domains: Vec<Domain>,
    pub graph: Vec<StageNode>,
    pub defaults: EffectiveDefaults,
    pub invariants_preset: Option<&'static str>,
    pub capabilities: PipelineCapabilities,
}

#[derive(Debug, Clone, Serialize)]
pub struct PipelineCapabilities {
    pub required_inputs: Vec<&'static str>,
    pub produces_outputs: Vec<&'static str>,
    pub report_sections: Vec<&'static str>,
    pub supports_benchmarking: bool,
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
    if !version.starts_with('v') || version.len() < 2 || !version[1..].chars().all(char::is_numeric) {
        return Err(anyhow!("pipeline id version must be v<digits>"));
    }
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
    if !graph.chars().all(allowed) || !flavor.chars().all(allowed) {
        return Err(anyhow!("pipeline id contains invalid characters"));
    }
    Ok(())
}
