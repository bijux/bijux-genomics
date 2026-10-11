use super::paths::validate_source_path;
use crate::domain::compilation::{PaperMapEntry, QaCoverageBlockerEntry, ToolEvidenceMapEntry};
use crate::domain::LoadedSpecs;
use crate::io::read_utf8;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::path::Path;

pub(super) fn load_fastq_tool_evidence_map(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<ToolEvidenceMapEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.tool-evidence-map") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    let rows = parse_tsv_rows(&read_utf8(&path)?);
    let mut out = Vec::new();
    for row in rows {
        let source_id = row_value(&row, "source_id");
        out.push(ToolEvidenceMapEntry {
            archive_status: validated_archive_status(
                &row_value(&row, "archive_status"),
                &format!("FASTQ tool evidence row {source_id}"),
            )?,
            source_id,
            tool_id: row_value(&row, "tool_id"),
            archive_path: row_value(&row, "archive_path"),
            paper_root: row_value(&row, "paper_root"),
            acquisition_mode: row_value(&row, "acquisition_mode"),
            primary_locator: row_value(&row, "primary_locator"),
        });
    }
    Ok(out)
}
pub(super) fn load_fastq_paper_map(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<PaperMapEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.paper-map") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    let rows = parse_tsv_rows(&read_utf8(&path)?);
    let mut out = Vec::new();
    for row in rows {
        let paper_id = row_value(&row, "paper_id");
        out.push(PaperMapEntry {
            archive_status: validated_archive_status(
                &row_value(&row, "archive_status"),
                &format!("FASTQ paper map row {paper_id}"),
            )?,
            paper_id,
            tool_id: row_value(&row, "tool_id"),
            paper_root: row_value(&row, "paper_root"),
            paper_status: row_value(&row, "paper_status"),
            open_access_status: row_value(&row, "open_access_status"),
            primary_locator: row_value(&row, "primary_locator"),
            supporting_locators: row_value(&row, "supporting_locators"),
            notes: row_value(&row, "notes"),
        });
    }
    Ok(out)
}
fn validated_archive_status(value: &str, context: &str) -> Result<String> {
    match value {
        "missing" | "present" => Ok(value.to_string()),
        _ => Err(anyhow!("{context} must declare archive_status as missing or present")),
    }
}
pub(super) fn load_fastq_qa_coverage_blockers(
    root: &Path,
    loaded: &LoadedSpecs,
) -> Result<Vec<QaCoverageBlockerEntry>> {
    let Some(source) = loaded.sources.get("source.fastq.qa-coverage-blockers") else {
        return Ok(Vec::new());
    };
    let path = validate_source_path(root, source)?;
    let rows = parse_tsv_rows(&read_utf8(&path)?);
    let mut out = Vec::new();
    for row in rows {
        if row_value(&row, "status") == "tracked" {
            out.push(QaCoverageBlockerEntry {
                stage_id: row_value(&row, "stage_id"),
                blocker: row_value(&row, "blocker"),
            });
        }
    }
    Ok(out)
}
fn parse_tsv_rows(text: &str) -> Vec<BTreeMap<String, String>> {
    let mut lines = text.lines().filter(|line| !line.trim().is_empty());
    let Some(header_line) = lines.next() else {
        return Vec::new();
    };
    let headers = header_line.split('\t').map(str::to_string).collect::<Vec<_>>();
    let mut rows = Vec::new();
    for line in lines {
        let values = line.split('\t').map(str::to_string).collect::<Vec<_>>();
        let mut parsed_row = BTreeMap::new();
        for (index, header) in headers.iter().enumerate() {
            parsed_row.insert(header.clone(), values.get(index).cloned().unwrap_or_default());
        }
        rows.push(parsed_row);
    }
    rows
}
fn row_value(row: &BTreeMap<String, String>, key: &str) -> String {
    row.get(key).cloned().unwrap_or_default()
}
