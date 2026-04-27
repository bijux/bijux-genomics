use std::path::Path;

use anyhow::Result;

use crate::cli::{ScienceCli, ScienceCommand};
use crate::compile::compile_workspace;
use crate::io::write_utf8;
use crate::release::cut_release;
use crate::render::{
    binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv, fastq_closure_gate_tsv,
    fastq_container_reference_tsv, fastq_default_binding_risk_tsv, fastq_download_backlog_tsv,
    fastq_environment_tsv, fastq_missing_closure_prerequisites_tsv, fastq_paper_archive_tsv,
    fastq_truth_delta_tsv, index_json, source_archive_gaps_tsv, source_inventory_tsv,
    to_pretty_json,
};

/// Run the science CLI command.
///
/// # Errors
///
/// Returns an error when validation, build, trace, closure, or release work cannot compile the
/// workspace science inputs or write the requested outputs.
pub fn run(cli: ScienceCli) -> Result<()> {
    match cli.command {
        ScienceCommand::Validate => {
            validate_workspace(&cli.workspace_root)?;
            println!("science specs validated");
        }
        ScienceCommand::Build => {
            let compiled = build_workspace(&cli.workspace_root)?;
            let summary = &compiled.index.fastq_closure_summary;
            let source_summary = &compiled.index.source_archive_summary;
            let present_archives =
                source_summary.archive_status_counts.get("present").copied().unwrap_or_default();
            let missing_archives =
                source_summary.archive_status_counts.get("missing").copied().unwrap_or_default();
            let manual_archives =
                source_summary.access_counts.get("manual_download").copied().unwrap_or_default()
                    + source_summary.access_counts.get("manual_clone").copied().unwrap_or_default();
            println!(
                "science outputs refreshed: {} fastq environment rows; defaults={} world_class_closed={} declared_closed_with_gaps={} not_closed={}; source_archives_present={} source_archives_missing={} source_archives_manual={}",
                compiled.fastq_environment_rows.len(),
                summary.default_rows,
                summary.world_class_closed_rows,
                summary.declared_closed_with_gaps_rows,
                summary.not_closed_rows,
                present_archives,
                missing_archives,
                manual_archives,
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
        ScienceCommand::Closure { stage, tool } => {
            let compiled = compile_workspace(&cli.workspace_root)?;
            for row in compiled
                .fastq_closure_gate_rows
                .iter()
                .filter(|row| stage.as_ref().is_none_or(|value| &row.stage_id == value))
                .filter(|row| tool.as_ref().is_none_or(|value| &row.tool_id == value))
            {
                println!(
                    "{} {} world_class_closed={} status={} blockers={}",
                    row.stage_id,
                    row.tool_id,
                    row.world_class_closed,
                    row.effective_closure_status,
                    if row.blocking_reasons.is_empty() {
                        "none"
                    } else {
                        row.blocking_reasons.as_str()
                    }
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

/// Validate authored science inputs for a workspace.
///
/// # Errors
///
/// Returns an error when science specs cannot be read, parsed, or resolved.
pub fn validate_workspace(root: &Path) -> Result<()> {
    compile_workspace(root).map(|_| ())
}

/// Compile science inputs and refresh governed generated outputs.
///
/// # Errors
///
/// Returns an error when compilation fails or a generated output cannot be written.
pub fn build_workspace(root: &Path) -> Result<crate::domain::CompiledScience> {
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
        &root.join("science/generated/current/evidence/fastq_closure_gate.tsv"),
        &fastq_closure_gate_tsv(&compiled.fastq_closure_gate_rows),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_truth_delta.tsv"),
        &fastq_truth_delta_tsv(&compiled.fastq_truth_delta_rows),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv"),
        &fastq_missing_closure_prerequisites_tsv(&compiled.fastq_missing_closure_prerequisite_rows),
    )?;
    write_utf8(
        &root.join("science/generated/current/evidence/fastq_default_binding_risk_ledger.tsv"),
        &fastq_default_binding_risk_tsv(&compiled.fastq_default_binding_risk_rows),
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

/// Return compiled FASTQ environment rows, optionally filtered by stage and tool.
///
/// # Errors
///
/// Returns an error when the workspace cannot be compiled.
pub fn trace_workspace(
    root: &Path,
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

/// Cut an immutable science release bundle for a known release manifest.
///
/// # Errors
///
/// Returns an error when the release manifest is missing, compilation fails, or release outputs
/// cannot be written.
pub fn release_workspace(root: &Path, release_id: &str) -> Result<()> {
    cut_release(root, release_id)
}
