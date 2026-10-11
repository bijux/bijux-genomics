//! Typed observations admitted for one science compilation; no repository access.
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct CompilationEvidence {
    pub(crate) execution_support: ExecutionSupportDoc,
    pub(crate) planned_tools: BTreeMap<String, BTreeSet<String>>,
    pub(crate) registry: BTreeMap<String, ToolRegistryEntry>,
    pub(crate) tool_contracts: BTreeMap<String, FastqToolContractEntry>,
    pub(crate) tool_evidence_map: Vec<ToolEvidenceMapEntry>,
    pub(crate) paper_map: Vec<PaperMapEntry>,
    pub(crate) qa_coverage_blockers: Vec<QaCoverageBlockerEntry>,
}

#[derive(Deserialize)]
pub(crate) struct ExecutionSupportDoc {
    pub(crate) stages: Vec<ExecutionSupportStage>,
}

#[derive(Deserialize)]
pub(crate) struct ExecutionSupportStage {
    pub(crate) stage_id: String,
    pub(crate) execution_status: String,
    pub(crate) runtime_support: String,
    pub(crate) normalization_support: String,
    pub(crate) benchmark_support: String,
    pub(crate) default_tool: Option<String>,
    pub(crate) admitted_tools: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ToolRegistryEntry {
    pub(crate) status: String,
    pub(crate) version: String,
    pub(crate) default_version: String,
    pub(crate) version_rule: String,
    pub(crate) upstream: String,
    pub(crate) citation: String,
    pub(crate) license: String,
    pub(crate) pinned_commit: String,
    pub(crate) pin_strategy: String,
    pub(crate) runtimes: Vec<String>,
    pub(crate) container_ref: String,
    pub(crate) dockerfile: String,
    pub(crate) apptainer_def: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub(crate) struct FastqToolContractEntry {
    #[serde(default)]
    pub(crate) tool_id: String,
    #[serde(default)]
    pub(crate) status: String,
    #[serde(default)]
    pub(crate) default_version: String,
    #[serde(default)]
    pub(crate) versioning_strategy: String,
    #[serde(default)]
    pub(crate) pin_strategy: String,
    #[serde(default)]
    pub(crate) upstream: String,
    #[serde(default)]
    pub(crate) citation: String,
    #[serde(default)]
    pub(crate) license: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ToolEvidenceMapEntry {
    pub(crate) source_id: String,
    pub(crate) tool_id: String,
    pub(crate) archive_path: String,
    pub(crate) archive_status: String,
    pub(crate) paper_root: String,
    pub(crate) acquisition_mode: String,
    pub(crate) primary_locator: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PaperMapEntry {
    pub(crate) paper_id: String,
    pub(crate) tool_id: String,
    pub(crate) paper_root: String,
    pub(crate) archive_status: String,
    pub(crate) paper_status: String,
    pub(crate) open_access_status: String,
    pub(crate) primary_locator: String,
    pub(crate) supporting_locators: String,
    pub(crate) notes: String,
}

#[derive(Clone, Debug)]
pub(crate) struct QaCoverageBlockerEntry {
    pub(crate) stage_id: String,
    pub(crate) blocker: String,
}
