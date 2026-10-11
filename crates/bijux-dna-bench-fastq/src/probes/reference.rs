use super::inputs::required_input_path;
use super::parameters::u64_json;
use super::paths::path_relative_to_repo;
use super::MaterializedFastqProof;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
/// Publish the governed synthetic index reference report for parser verification.
///
/// # Errors
/// Rejects foreign plan identity, unavailable inputs and report publication failures.
pub fn generate_index_reference_probe(
    repo_root: &Path,
    probe_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<MaterializedFastqProof> {
    super::inputs::admit_stage(
        plan,
        bijux_dna_domain_fastq::stages::STAGE_INDEX_REFERENCE.as_str(),
    )?;

    let reference_fasta = required_input_path(repo_root, plan, "reference_fasta")?;
    let reference_bytes = fs::metadata(&reference_fasta)
        .with_context(|| format!("stat {}", reference_fasta.display()))?
        .len();
    let probe_dir = probe_root.join("reference_indexing");
    bijux_dna_infra::ensure_dir(&probe_dir)
        .with_context(|| format!("create {}", probe_dir.display()))?;
    let tool_id = plan.tool_id.as_str().to_string();
    let index_root = probe_dir.join("reference_index");
    bijux_dna_infra::ensure_dir(&index_root)
        .with_context(|| format!("create {}", index_root.display()))?;
    let extension = if tool_id == "bowtie2_build" { "bt2" } else { "idx" };
    let emitted_path = index_root.join(format!("reference.1.{extension}"));
    bijux_dna_infra::atomic_write_bytes(&emitted_path, b"bijux-index\n")
        .with_context(|| format!("write {}", emitted_path.display()))?;
    let emitted_files = vec![bijux_dna_domain_fastq::IndexReferenceFileEntryV1 {
        relative_path: emitted_path
            .strip_prefix(&index_root)
            .unwrap_or(&emitted_path)
            .display()
            .to_string(),
        bytes: fs::metadata(&emitted_path)
            .with_context(|| format!("stat {}", emitted_path.display()))?
            .len(),
    }];
    let index_bytes = emitted_files.iter().map(|entry| entry.bytes).sum::<u64>();
    let report_path = probe_dir.join("index_reference_report.json");
    let report = bijux_dna_domain_fastq::IndexReferenceReportV1 {
        schema_version: bijux_dna_domain_fastq::INDEX_REFERENCE_REPORT_SCHEMA_VERSION.to_string(),
        stage: bijux_dna_domain_fastq::stages::STAGE_INDEX_REFERENCE.as_str().to_string(),
        stage_id: bijux_dna_domain_fastq::stages::STAGE_INDEX_REFERENCE.as_str().to_string(),
        tool_id: tool_id.clone(),
        threads: u64_json(&plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        index_format: tool_id.clone(),
        reference_fasta: path_relative_to_repo(repo_root, &reference_fasta),
        reference_bytes,
        reference_index: path_relative_to_repo(repo_root, &index_root),
        report_json: path_relative_to_repo(repo_root, &report_path),
        index_prefix: Some("reference".to_string()),
        index_file_count: emitted_files.len() as u64,
        index_bytes,
        emitted_files,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        backend_metrics: Some(serde_json::json!({ "probe": "governed_tiny_index_reference" })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedFastqProof { proof_path: report_path, observed_tool_id: tool_id })
}
