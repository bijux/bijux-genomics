use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_env_runtime::api::{PlatformSpec, RunnerKind};
use serde::{Deserialize, Serialize};

mod logging;
pub use logging::{init_logging, StdoutLogger};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub platform: PlatformSpec,
    pub runner_override: Option<RunnerKind>,
    pub env: BTreeMap<String, String>,
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Capability {
    Fastq,
    Bam,
    Vcf,
    Umi,
    ReferenceGenome,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageRequirement {
    pub capabilities: Vec<Capability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInvocation {
    pub stage_id: String,
    pub tool_id: String,
    pub inputs: Vec<PathBuf>,
    pub params: serde_json::Value,
    pub requirements: Option<StageRequirement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageGraph {
    pub nodes: Vec<String>,
    pub edges: Vec<(String, String)>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageNode {
    pub stage_id: String,
    pub tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    pub from: String,
    pub to: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPlan {
    pub stages: Vec<StageNode>,
    pub edges: Vec<Dependency>,
    pub policy: Policy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineSpec {
    pub stages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunPlan {
    pub invocation: ToolInvocation,
    pub image_digest: String,
    pub runner: RunnerKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageResult {
    pub invocation: ToolInvocation,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub outputs: Vec<PathBuf>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Policy {
    PreferAccuracy,
    PreferSpeed,
    PreferMemory,
}

pub type MetricSet = bijux_core::metrics::MetricSet<serde_json::Value>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataArtifact {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadSet {
    pub reads: Vec<DataArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SequenceCollection {
    pub items: Vec<DataArtifact>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionManifest {
    pub run_id: String,
    pub stage: String,
    pub tool: String,
    pub tool_version: String,
    pub image_digest: String,
    pub command: String,
    pub input_hashes: Vec<String>,
    pub input_files: Vec<String>,
    pub output_dir: String,
    pub runner: String,
    pub platform: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainExclusion {
    pub tool: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExplainPlan {
    pub stage: String,
    pub selected_tools: Vec<String>,
    pub excluded_tools: Vec<ExplainExclusion>,
    pub policy: Option<Policy>,
    pub invariants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainDefinition {
    pub stages: Vec<String>,
    pub metrics: Vec<String>,
    pub validators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DomainRegistry {
    pub domains: std::collections::BTreeMap<String, DomainDefinition>,
}

impl DomainRegistry {
    pub fn register(&mut self, name: impl Into<String>, def: DomainDefinition) {
        self.domains.insert(name.into(), def);
    }
}

#[must_use]
pub fn default_domain_registry() -> DomainRegistry {
    let mut registry = DomainRegistry::default();
    registry.register("fastq", DomainDefinition::default());
    registry.register("bam", DomainDefinition::default());
    registry.register("vcf", DomainDefinition::default());
    registry
}

pub fn trace_enabled() -> bool {
    std::env::var("BIJUX_TRACE_ENGINE").is_ok()
}
