use super::inputs::{optional_input_path, required_input_path};
use super::parameters::{bool_json, f64_json, optional_string_json, string_json, u64_json};
use super::paths::path_relative_to_repo;
use super::publication::{copy_file, write_empty_gzip};
use super::streams::count_fastq_stats;
use super::MaterializedFastqProof;
use anyhow::{Context, Result};
use serde_json::Value;
use std::path::Path;
/// Publish the governed synthetic deplete rrna report for parser verification.
///
/// # Errors
/// Rejects foreign plan identity, unavailable inputs and report publication failures.
pub fn generate_deplete_rrna_probe(
    repo_root: &Path,
    probe_root: &Path,
    plan: &bijux_dna_stage_contract::StagePlanV1,
) -> Result<MaterializedFastqProof> {
    super::inputs::admit_stage(plan, bijux_dna_domain_fastq::stages::STAGE_DEPLETE_RRNA.as_str())?;

    let input_r1 = required_input_path(plan, "reads_r1")?;
    let input_r2 = optional_input_path(plan, "reads_r2");
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let probe_dir = probe_root.join("deplete_rrna");
    bijux_dna_infra::ensure_dir(&probe_dir)
        .with_context(|| format!("create {}", probe_dir.display()))?;

    let output_r1 = probe_dir.join("rrna_filtered_R1.fastq.gz");
    copy_file(&input_r1, &output_r1)?;
    let output_r2 = if let Some(input) = input_r2.as_ref() {
        let path = probe_dir.join("rrna_filtered_R2.fastq.gz");
        copy_file(input, &path)?;
        Some(path)
    } else {
        None
    };
    let removed_r1 = probe_dir.join("removed_rrna_R1.fastq.gz");
    write_empty_gzip(&removed_r1)?;
    let removed_r2 = if input_r2.is_some() {
        let path = probe_dir.join("removed_rrna_R2.fastq.gz");
        write_empty_gzip(&path)?;
        Some(path)
    } else {
        None
    };
    let report_tsv = probe_dir.join("rrna_report.tsv");
    bijux_dna_infra::atomic_write_bytes(&report_tsv, b"sample_id\treads_removed\nlocal_smoke\t0\n")
        .with_context(|| format!("write {}", report_tsv.display()))?;

    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let report_path = probe_dir.join("rrna_report.json");
    let report = bijux_dna_domain_fastq::DepleteRrnaReportV1 {
        schema_version: bijux_dna_domain_fastq::DEPLETE_RRNA_REPORT_SCHEMA_VERSION.to_string(),
        stage: bijux_dna_domain_fastq::stages::STAGE_DEPLETE_RRNA.as_str().to_string(),
        stage_id: bijux_dna_domain_fastq::stages::STAGE_DEPLETE_RRNA.as_str().to_string(),
        tool_id: plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: u64_json(&plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        rrna_db: optional_string_json(&plan.effective_params, "contaminant_db"),
        database_artifact_id: string_json(&plan.effective_params, "database_artifact_id")
            .unwrap_or_else(|| "rrna_reference".to_string()),
        database_build_id: optional_string_json(&plan.effective_params, "database_build_id"),
        database_digest: optional_string_json(&plan.effective_params, "database_digest"),
        screening_engine: serde_json::from_value(
            plan.effective_params
                .get("screening_engine")
                .cloned()
                .unwrap_or(Value::String("sortmerna".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::RrnaScreeningEngine::Sortmerna),
        report_format: serde_json::from_value(
            plan.effective_params
                .get("report_format")
                .cloned()
                .unwrap_or(Value::String("summary_tsv_and_json".to_string())),
        )
        .unwrap_or(bijux_dna_domain_fastq::params::screen::RrnaReportFormat::SummaryTsvAndJson),
        emit_removed_reads: bool_json(&plan.effective_params, "emit_removed_reads").unwrap_or(true),
        min_identity: f64_json(&plan.params, "min_identity"),
        retained_read_role: "rrna_filtered_reads".to_string(),
        rejected_read_role: "removed_rrna_reads".to_string(),
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        removed_reads_r1: path_relative_to_repo(repo_root, &removed_r1),
        removed_reads_r2: removed_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        rrna_report_tsv: path_relative_to_repo(repo_root, &report_tsv),
        rrna_report_json: path_relative_to_repo(repo_root, &report_path),
        reads_in,
        reads_out: reads_in,
        reads_removed: 0,
        bases_in,
        bases_out: bases_in,
        bases_removed: 0,
        pairs_in,
        pairs_out: pairs_in,
        rrna_fraction_removed: 0.0,
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(serde_json::json!({ "reads_removed": 0_u64 })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedFastqProof {
        proof_path: report_path,
        observed_tool_id: plan.tool_id.as_str().to_string(),
    })
}
