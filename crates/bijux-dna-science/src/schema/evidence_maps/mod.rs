//! Typed science evidence maps; declarations are not proof of archived payloads.
mod table;
use crate::domain::compilation::{PaperMapEntry, QaCoverageBlockerEntry, ToolEvidenceMapEntry};
use anyhow::{anyhow, ensure, Result};
use std::collections::BTreeSet;

pub(crate) fn parse_tool_map(text: &str) -> Result<Vec<ToolEvidenceMapEntry>> {
    let rows = table::parse(
        text,
        &[
            "source_id",
            "tool_id",
            "archive_path",
            "archive_status",
            "paper_root",
            "acquisition_mode",
            "primary_locator",
        ],
    )?;
    let mut out = Vec::new();
    let mut source_ids = BTreeSet::new();
    let mut tool_ids = BTreeSet::new();

    for row in rows {
        ensure!(
            source_ids.insert(table::identity(&row, "source_id")?.to_string()),
            "evidence table repeats source_id"
        );
        ensure!(
            tool_ids.insert(table::identity(&row, "tool_id")?.to_string()),
            "evidence table repeats tool_id"
        );

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
pub(crate) fn parse_paper_map(text: &str) -> Result<Vec<PaperMapEntry>> {
    let rows = table::parse(
        text,
        &[
            "paper_id",
            "tool_id",
            "paper_root",
            "archive_status",
            "paper_status",
            "open_access_status",
            "primary_locator",
            "supporting_locators",
            "notes",
        ],
    )?;
    let mut out = Vec::new();
    let mut paper_ids = BTreeSet::new();
    let mut paper_roots = BTreeSet::new();

    for row in rows {
        ensure!(
            paper_ids.insert(table::identity(&row, "paper_id")?.to_string()),
            "evidence table repeats paper_id"
        );
        ensure!(
            paper_roots.insert(table::identity(&row, "paper_root")?.to_string()),
            "evidence table repeats paper_root"
        );
        table::identity(&row, "tool_id")?;

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
pub(crate) fn parse_qa_map(text: &str) -> Result<Vec<QaCoverageBlockerEntry>> {
    let rows = table::parse(text, &["stage_id", "blocker", "status"])?;
    let mut out = Vec::new();

    for row in rows {
        table::identity(&row, "stage_id")?;
        table::identity(&row, "blocker")?;
        table::identity(&row, "status")?;

        if row_value(&row, "status") == "tracked" {
            out.push(QaCoverageBlockerEntry {
                stage_id: row_value(&row, "stage_id"),
                blocker: row_value(&row, "blocker"),
            });
        }
    }
    Ok(out)
}
fn row_value(row: &table::Row, key: &str) -> String {
    table::value(row, key).to_string()
}

#[cfg(test)]
#[path = "../../../tests/contracts/evidence_maps.rs"]
mod tests;
