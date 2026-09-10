//! Acquire every shared upstream input once before deriving science evidence.
mod maps;
mod paths;
mod registry;

use crate::domain::compilation::{CompilationEvidence, ExecutionSupportDoc};
use crate::domain::LoadedSpecs;
use crate::io::read_utf8;
use anyhow::{anyhow, Context, Result};
use maps::{load_fastq_paper_map, load_fastq_qa_coverage_blockers, load_fastq_tool_evidence_map};
use paths::validate_source_path;
use registry::{load_fastq_tool_contracts, load_planned_out_of_scope, load_tool_registry};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

pub(crate) fn load_compilation_evidence(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<CompilationEvidence> {
    let sources = resolve_binding_sources(root, loaded)?;
    let registry_path = sources
        .get("source.fastq.tool-registry")
        .context("missing source.fastq.tool-registry binding for fastq container matrix")?;
    let execution_path = sources
        .get("source.fastq.execution-support")
        .context("missing source.fastq.execution-support binding")?;
    let stage_path = sources
        .get("source.fastq.stage-contracts")
        .context("missing source.fastq.stage-contracts binding")?;
    let execution_support: ExecutionSupportDoc =
        bijux_dna_infra::formats::yaml::parse_yaml(&read_utf8(execution_path)?)
            .map_err(|err| anyhow!("parse {}: {err}", execution_path.display()))?;
    Ok(CompilationEvidence {
        execution_support,
        planned_tools: load_planned_out_of_scope(stage_path)?,
        registry: load_tool_registry(registry_path)?,
        tool_contracts: load_fastq_tool_contracts(&root.join("domain/fastq/tools"))?,
        tool_evidence_map: load_fastq_tool_evidence_map(root, loaded)?,
        paper_map: load_fastq_paper_map(root, loaded)?,
        qa_coverage_blockers: load_fastq_qa_coverage_blockers(root, loaded)?,
    })
}

fn resolve_binding_sources(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<BTreeMap<&'static str, PathBuf>> {
    let mut resolved = BTreeMap::new();
    for binding in loaded
        .bindings
        .values()
        .filter(|binding| binding.target_type == "fastq_stage_tool_environment_matrix")
    {
        for required in [
            "source.fastq.execution-support",
            "source.fastq.stage-contracts",
            "source.fastq.tool-registry",
        ] {
            let source = binding
                .source_ids
                .iter()
                .find(|source_id| source_id.as_str() == required)
                .and_then(|source_id| loaded.sources.get(source_id.as_str()))
                .ok_or_else(|| anyhow!("binding is missing required source {required}"))?;
            if let std::collections::btree_map::Entry::Vacant(entry) = resolved.entry(required) {
                entry.insert(validate_source_path(root, source)?);
            }
        }
    }
    Ok(resolved)
}
