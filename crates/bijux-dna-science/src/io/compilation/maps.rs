//! Acquire governed UTF-8 tables and delegate their typed admission to the schema owner.
use super::paths::validate_source_path;
use crate::domain::compilation::{PaperMapEntry, QaCoverageBlockerEntry, ToolEvidenceMapEntry};
use crate::domain::LoadedSpecs;
use crate::io::read_utf8;
use crate::schema::evidence_maps::{parse_paper_map, parse_qa_map, parse_tool_map};
use anyhow::{Context, Result};
use std::path::Path;

pub(super) fn load_fastq_tool_evidence_map(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<ToolEvidenceMapEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.tool-evidence-map") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    parse_tool_map(&read_utf8(&path)?)
        .with_context(|| format!("admit evidence table {}", path.display()))
}

pub(super) fn load_fastq_paper_map(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<PaperMapEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.paper-map") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    parse_paper_map(&read_utf8(&path)?)
        .with_context(|| format!("admit evidence table {}", path.display()))
}

pub(super) fn load_fastq_qa_coverage_blockers(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<QaCoverageBlockerEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.qa-coverage-blockers") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    parse_qa_map(&read_utf8(&path)?)
        .with_context(|| format!("admit evidence table {}", path.display()))
}
