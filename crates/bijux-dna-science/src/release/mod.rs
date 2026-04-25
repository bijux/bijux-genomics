use std::path::Path;

use anyhow::{anyhow, Result};

use crate::compile::compile_workspace;
use crate::io::write_utf8;
use crate::render::{binding_resolution_tsv, claim_evidence_tsv, decision_reasoning_tsv, fastq_environment_tsv, index_json, to_pretty_json};

pub fn cut_release(root: &Path, release_id: &str) -> Result<()> {
    let compiled = compile_workspace(root)?;
    let release_root = root.join("artifacts/science-releases").join(release_id);
    if release_root.exists() {
        return Err(anyhow!("release already exists at {}", release_root.display()));
    }
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
    write_utf8(
        &release_root.join("indexes/science_index.json"),
        &index_json(&compiled.index)?,
    )?;
    write_utf8(
        &release_root.join("indexes/release.json"),
        &to_pretty_json(&serde_json::json!({
            "release_id": release_id,
            "fastq_environment_rows": compiled.fastq_environment_rows.len(),
        }))?,
    )?;
    Ok(())
}
