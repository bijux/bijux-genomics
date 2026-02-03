//! Pipeline profiles across FASTQ, BAM, and cross-domain workflows.

use std::collections::BTreeMap;

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
    pub id: &'static str,
    pub description: &'static str,
    pub domains: Vec<Domain>,
    pub graph: Vec<StageNode>,
    pub defaults: EffectiveDefaults,
    pub invariants_preset: Option<&'static str>,
}
