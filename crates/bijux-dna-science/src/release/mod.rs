use std::path::Path;

use anyhow::{anyhow, Result};

use crate::compile::{compile_workspace, load_specs};
use crate::io::write_utf8;
use crate::render::{
    binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv,
    fastq_container_reference_tsv, fastq_environment_tsv, index_json, source_archive_gaps_tsv,
    source_inventory_tsv, to_pretty_json,
};

pub fn cut_release(root: &Path, release_id: &str) -> Result<()> {
    let loaded = load_specs(root)?;
    let manifest = loaded
        .releases
        .get(release_id)
        .ok_or_else(|| anyhow!("release manifest not found for {release_id}"))?;
    let compiled = compile_workspace(root)?;
    let release_root = root.join("artifacts/science-releases").join(release_id);
    if release_root.exists() {
        return Err(anyhow!("release already exists at {}", release_root.display()));
    }
    write_utf8(
        &release_root.join("evidence/source_inventory.tsv"),
        &source_inventory_tsv(&compiled.source_inventory),
    )?;
    write_utf8(
        &release_root.join("evidence/source_archive_gaps.tsv"),
        &source_archive_gaps_tsv(&compiled.source_archive_gaps),
    )?;
    write_utf8(
        &release_root.join("evidence/fastq_container_reference_matrix.tsv"),
        &fastq_container_reference_tsv(&compiled.fastq_container_reference_rows),
    )?;
    write_utf8(
        &release_root.join("evidence/claim_evidence_map.tsv"),
        &claim_evidence_tsv(&compiled.claim_evidence_map),
    )?;
    write_utf8(
        &release_root.join("evidence/decision_reasoning_map.tsv"),
        &decision_reasoning_tsv(&compiled.decision_reasoning_map),
    )?;
    write_utf8(
        &release_root.join("evidence/binding_resolution.tsv"),
        &binding_resolution_tsv(&compiled.binding_resolution),
    )?;
    write_utf8(
        &release_root.join("evidence/fastq_stage_tool_environment_matrix.tsv"),
        &fastq_environment_tsv(&compiled.fastq_environment_rows),
    )?;
    write_utf8(&release_root.join("indexes/science_index.json"), &index_json(&compiled.index)?)?;
    write_utf8(
        &release_root.join("indexes/release.json"),
        &to_pretty_json(&serde_json::json!({
            "release_id": release_id,
            "title": manifest.title,
            "status": manifest.status,
            "binding_ids": manifest.binding_ids,
            "claim_ids": manifest.claim_ids,
            "fastq_environment_rows": compiled.fastq_environment_rows.len(),
        }))?,
    )?;
    Ok(())
}
