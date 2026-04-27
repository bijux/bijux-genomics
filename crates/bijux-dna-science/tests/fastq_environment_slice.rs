use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_science::compile::compile_workspace;
use bijux_dna_science::domain::CompiledScience;
use bijux_dna_science::render::{
    binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv, fastq_closure_gate_tsv,
    fastq_container_reference_tsv, fastq_default_binding_risk_tsv, fastq_download_backlog_tsv,
    fastq_environment_tsv, fastq_missing_closure_prerequisites_tsv, fastq_paper_archive_tsv,
    fastq_truth_delta_tsv, index_json, source_archive_gaps_tsv, source_inventory_tsv,
    to_pretty_json,
};

fn repo_root() -> Result<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .context("resolve repository root from crate manifest")
}

#[test]
fn fastq_environment_slice_matches_committed_outputs() -> Result<()> {
    let root = repo_root()?;
    let compiled = compile_workspace(&root)?;

    assert_fastq_slice_rows(&compiled);
    assert_generated_output_inventory(&root)?;
    assert_committed_outputs_match(&root, &compiled)?;
    Ok(())
}

fn assert_fastq_slice_rows(compiled: &CompiledScience) {
    assert!(compiled.fastq_environment_rows.iter().any(|row| {
        row.stage_id == "fastq.trim_reads" && row.tool_id == "fastp" && row.is_default
    }));
    assert!(compiled.fastq_environment_rows.iter().any(|row| {
        row.stage_id == "fastq.trim_reads"
            && row.tool_id == "seqpurge"
            && row.tool_status == "disallowed"
    }));
    assert!(compiled
        .source_inventory
        .iter()
        .any(|row| row.source_id == "source.fastq.tool-registry"));
    assert!(compiled
        .fastq_container_reference_rows
        .iter()
        .any(|row| row.tool_id == "fastp" && row.version == "0.23.4"));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "fastp" && row.source_id == "source.fastq.tool.fastp.upstream"
    }));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "diamond"
            && row.backlog_status == "ready"
            && row.locator == "https://github.com/bbuchfink/diamond"
    }));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "dustmasker"
            && row.backlog_status == "ready"
            && row.locator
                == "https://www.ncbi.nlm.nih.gov/IEB/ToolBox/CPP_DOC/lxr/source/src/app/dustmasker/"
    }));
    assert!(compiled.fastq_download_backlog_rows.iter().any(|row| {
        row.tool_id == "fastp"
            && row.paper_root == "science/docs/upstream/papers/paper.fastq.fastp.chen-2018"
    }));
    assert!(compiled.fastq_paper_archive_rows.iter().any(|row| {
        row.tool_id == "atropos"
            && row.paper_id == "paper.fastq.atropos.didion-2017"
            && row.paper_status == "mapped"
    }));
    assert_eq!(
        compiled.index.source_inventory_rows,
        compiled.source_inventory.len()
    );
    assert_eq!(
        compiled.index.source_archive_gap_rows,
        compiled.source_archive_gaps.len()
    );
    assert_eq!(
        compiled
            .index
            .source_archive_summary
            .archive_status_counts
            .get("present")
            .copied()
            .unwrap_or_default(),
        compiled
            .source_inventory
            .iter()
            .filter(|row| row.archive_status == "present")
            .count()
    );
    assert_eq!(
        compiled
            .index
            .source_archive_summary
            .archive_status_counts
            .get("missing")
            .copied()
            .unwrap_or_default(),
        compiled
            .source_inventory
            .iter()
            .filter(|row| row.archive_status == "missing")
            .count()
    );
    assert_eq!(
        compiled
            .index
            .source_archive_summary
            .missing_tool_counts
            .len(),
        0
    );
    assert_eq!(
        compiled.index.fastq_closure_summary.total_rows,
        compiled.fastq_closure_gate_rows.len()
    );
    assert_eq!(
        compiled.index.fastq_closure_summary.default_rows,
        compiled.fastq_closure_gate_rows.iter().filter(|row| row.is_default).count()
    );
    assert!(
        compiled
            .index
            .fastq_closure_summary
            .blocking_reason_counts
            .contains_key("missing_environment_qa_stage")
    );
    assert!(
        compiled
            .index
            .fastq_evidence_summary
            .prerequisite_counts
            .contains_key("missing_environment_qa_stage")
    );
    assert!(
        compiled
            .index
            .fastq_evidence_summary
            .default_risk_counts
            .contains_key("closure_prerequisite_blocked")
    );
}

fn assert_committed_outputs_match(root: &Path, compiled: &CompiledScience) -> Result<()> {
    assert_rendered(
        root,
        "science/generated/current/evidence/source_inventory.tsv",
        &source_inventory_tsv(&compiled.source_inventory),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/source_archive_gaps.tsv",
        &source_archive_gaps_tsv(&compiled.source_archive_gaps),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_container_reference_matrix.tsv",
        &fastq_container_reference_tsv(&compiled.fastq_container_reference_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_download_backlog.tsv",
        &fastq_download_backlog_tsv(&compiled.fastq_download_backlog_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_paper_archive_matrix.tsv",
        &fastq_paper_archive_tsv(&compiled.fastq_paper_archive_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/claim_evidence_map.tsv",
        &claim_evidence_tsv(&compiled.claim_evidence_map),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/decision_reasoning_map.tsv",
        &decision_reasoning_tsv(&compiled.decision_reasoning_map),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/binding_resolution.tsv",
        &binding_resolution_tsv(&compiled.binding_resolution),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_stage_tool_environment_matrix.tsv",
        &fastq_environment_tsv(&compiled.fastq_environment_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_closure_gate.tsv",
        &fastq_closure_gate_tsv(&compiled.fastq_closure_gate_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_truth_delta.tsv",
        &fastq_truth_delta_tsv(&compiled.fastq_truth_delta_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv",
        &fastq_missing_closure_prerequisites_tsv(&compiled.fastq_missing_closure_prerequisite_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/fastq_default_binding_risk_ledger.tsv",
        &fastq_default_binding_risk_tsv(&compiled.fastq_default_binding_risk_rows),
    )?;
    assert_rendered(
        root,
        "science/generated/current/evidence/unresolved_refs.json",
        &to_pretty_json(&compiled.unresolved_refs)?,
    )?;
    assert_rendered(
        root,
        "science/generated/indexes/science_index.json",
        &index_json(&compiled.index)?,
    )?;
    Ok(())
}

fn assert_generated_output_inventory(root: &Path) -> Result<()> {
    let evidence_root = root.join("science/generated/current/evidence");
    let mut actual = BTreeSet::new();
    for entry in
        fs::read_dir(&evidence_root).with_context(|| format!("read {}", evidence_root.display()))?
    {
        let entry = entry?;
        if entry.file_type()?.is_file() && entry.file_name() != "README.md" {
            actual.insert(format!(
                "science/generated/current/evidence/{}",
                entry.file_name().to_string_lossy()
            ));
        }
    }
    actual.insert("science/generated/indexes/science_index.json".to_string());

    let expected = [
        "science/generated/current/evidence/binding_resolution.tsv",
        "science/generated/current/evidence/claim_evidence_map.tsv",
        "science/generated/current/evidence/decision_reasoning_map.tsv",
        "science/generated/current/evidence/fastq_closure_gate.tsv",
        "science/generated/current/evidence/fastq_container_reference_matrix.tsv",
        "science/generated/current/evidence/fastq_default_binding_risk_ledger.tsv",
        "science/generated/current/evidence/fastq_download_backlog.tsv",
        "science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv",
        "science/generated/current/evidence/fastq_paper_archive_matrix.tsv",
        "science/generated/current/evidence/fastq_stage_tool_environment_matrix.tsv",
        "science/generated/current/evidence/fastq_truth_delta.tsv",
        "science/generated/current/evidence/source_archive_gaps.tsv",
        "science/generated/current/evidence/source_inventory.tsv",
        "science/generated/current/evidence/unresolved_refs.json",
        "science/generated/indexes/science_index.json",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();

    assert_eq!(actual, expected, "generated science output inventory changed");
    Ok(())
}

fn assert_rendered(root: &Path, rel_path: &str, expected: &str) -> Result<()> {
    let path = root.join(rel_path);
    let actual = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
    assert_eq!(actual, expected, "generated output drifted at {rel_path}");
    Ok(())
}
