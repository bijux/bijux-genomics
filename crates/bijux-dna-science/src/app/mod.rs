use std::path::PathBuf;

use anyhow::Result;

use crate::cli::{ScienceCli, ScienceCommand};
use crate::compile::compile_workspace;
use crate::io::write_utf8;
use crate::release::cut_release;
use crate::render::{
    binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv,
    fastq_container_reference_tsv, fastq_download_backlog_tsv, fastq_environment_tsv,
    fastq_paper_archive_tsv, index_json, source_archive_gaps_tsv, source_inventory_tsv,
    to_pretty_json,
};

pub fn run(cli: ScienceCli) -> Result<()> {
    match cli.command {
        ScienceCommand::Validate => {
            validate_workspace(&cli.workspace_root)?;
            println!("science specs validated");
        }
        ScienceCommand::Build => {
            let compiled = build_workspace(&cli.workspace_root)?;
            println!(
                "science outputs refreshed: {} fastq environment rows",
                compiled.fastq_environment_rows.len()
            );
        }
        ScienceCommand::Trace { stage, tool } => {
            let rows = trace_workspace(&cli.workspace_root, stage.as_deref(), tool.as_deref())?;
            for row in rows {
                println!(
                    "{} {} {} default={} runtimes={} decision={}",
                    row.stage_id,
                    row.tool_id,
                    row.tool_status,
                    row.is_default,
                    row.runtimes,
                    row.decision_id
                );
            }
        }
        ScienceCommand::Release { release_id } => {
            release_workspace(&cli.workspace_root, &release_id)?;
            println!("science release cut: {release_id}");
        }
    }
    Ok(())
}

pub fn validate_workspace(root: &PathBuf) -> Result<()> {
    compile_workspace(root).map(|_| ())
}

pub fn build_workspace(root: &PathBuf) -> Result<crate::domain::CompiledScience> {
    let compiled = compile_workspace(root)?;
    write_utf8(
        &root.join("science/generated/current/evidence/source_inventory.tsv"),
        &source_inventory_tsv(&compiled.source_inventory),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/source_archive_gaps.tsv"),
        &source_archive_gaps_tsv(&compiled.source_archive_gaps),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_container_reference_matrix.tsv"),
        &fastq_container_reference_tsv(&compiled.fastq_container_reference_rows),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_download_backlog.tsv"),
        &fastq_download_backlog_tsv(&compiled.fastq_download_backlog_rows),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_paper_archive_matrix.tsv"),
        &fastq_paper_archive_tsv(&compiled.fastq_paper_archive_rows),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/claim_evidence_map.tsv"),
        &claim_evidence_tsv(&compiled.claim_evidence_map),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/decision_reasoning_map.tsv"),
        &decision_reasoning_tsv(&compiled.decision_reasoning_map),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/binding_resolution.tsv"),
        &binding_resolution_tsv(&compiled.binding_resolution),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/unresolved_refs.json"),
        &to_pretty_json(&compiled.unresolved_refs)?,
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_stage_tool_environment_matrix.tsv"),
        &fastq_environment_tsv(&compiled.fastq_environment_rows),
    )?;
    write_utf8(
        &root.join("science/generated/indexes/science_index.json"),
        &index_json(&compiled.index)?,
    )?;
    Ok(compiled)
}

pub fn trace_workspace(
    root: &PathBuf,
    stage: Option<&str>,
    tool: Option<&str>,
) -> Result<Vec<crate::domain::FastqEnvironmentRow>> {
    let compiled = compile_workspace(root)?;
    let mut rows = compiled
        .fastq_environment_rows
        .into_iter()
        .filter(|row| stage.is_none_or(|value| row.stage_id == value))
        .filter(|row| tool.is_none_or(|value| row.tool_id == value))
        .collect::<Vec<_>>();
    rows.sort_by(|left, right| {
        (&left.stage_id, !left.is_default, &left.tool_id).cmp(&(
            &right.stage_id,
            !right.is_default,
            &right.tool_id,
        ))
    });
    Ok(rows)
}

pub fn release_workspace(root: &PathBuf, release_id: &str) -> Result<()> {
    cut_release(root, release_id)
}
