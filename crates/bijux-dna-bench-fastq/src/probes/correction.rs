use super::parameters::u64_json;
use super::paths::path_relative_to_repo;
use super::publication::copy_file;
use super::streams::count_fastq_stats;
use super::MaterializedFastqProof;
use anyhow::{Context, Result};
use std::path::Path;
/// Publish the governed synthetic correct errors report for parser verification.
///
/// # Errors
/// Rejects foreign plan identity, unavailable inputs and report publication failures.
pub fn generate_correct_errors_probe(
    repo_root: &Path,
    probe_root: &Path,
    case: &super::CorrectErrorsProbeInput<'_>,
) -> Result<MaterializedFastqProof> {
    super::inputs::admit_stage(
        case.plan,
        bijux_dna_domain_fastq::stages::STAGE_CORRECT_ERRORS.as_str(),
    )?;

    let input_r1 = repo_root.join(case.r1);
    let input_r2 = case.r2.map(|path| repo_root.join(path));
    let stats_r1 = count_fastq_stats(&input_r1)?;
    let stats_r2 = input_r2.as_ref().map(|path| count_fastq_stats(path)).transpose()?;
    let engine = match case.plan.tool_id.as_str() {
        "lighter" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Lighter,
        "musket" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Musket,
        "bayeshammer" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Bayeshammer,
        "rcorrector" => bijux_dna_domain_fastq::params::correct::CorrectionEngine::Rcorrector,
        other => return Err(anyhow::anyhow!("unsupported FASTQ correction probe tool `{other}`")),
    };
    let probe_dir = probe_root.join("error_correction");
    bijux_dna_infra::ensure_dir(&probe_dir)
        .with_context(|| format!("create {}", probe_dir.display()))?;

    let output_r1 = probe_dir.join("corrected_R1.fastq.gz");
    copy_file(&input_r1, &output_r1)?;
    let output_r2 = if let Some(input) = input_r2.as_ref() {
        let path = probe_dir.join("corrected_R2.fastq.gz");
        copy_file(input, &path)?;
        Some(path)
    } else {
        None
    };

    let reads_in = stats_r1.reads + stats_r2.as_ref().map_or(0, |stats| stats.reads);
    let bases_in = stats_r1.bases + stats_r2.as_ref().map_or(0, |stats| stats.bases);
    let pairs_in = input_r2
        .as_ref()
        .map(|_| stats_r1.reads.min(stats_r2.as_ref().map_or(0, |stats| stats.reads)));
    let report_path = probe_dir.join("correct_report.json");

    let report = bijux_dna_domain_fastq::CorrectErrorsReportV1 {
        schema_version: bijux_dna_domain_fastq::CORRECT_ERRORS_REPORT_SCHEMA_VERSION.to_string(),
        stage: bijux_dna_domain_fastq::stages::STAGE_CORRECT_ERRORS.as_str().to_string(),
        stage_id: bijux_dna_domain_fastq::stages::STAGE_CORRECT_ERRORS.as_str().to_string(),
        tool_id: case.plan.tool_id.as_str().to_string(),
        paired_mode: if input_r2.is_some() {
            bijux_dna_domain_fastq::PairedMode::PairedEnd
        } else {
            bijux_dna_domain_fastq::PairedMode::SingleEnd
        },
        threads: u64_json(&case.plan.effective_params, "threads")
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(1),
        correction_engine: engine,
        quality_encoding: case.quality_encoding.clone(),
        kmer_size: u64_json(&case.plan.effective_params, "kmer_size")
            .and_then(|value| u32::try_from(value).ok()),
        musket_kmer_budget: u64_json(&case.plan.effective_params, "musket_kmer_budget"),
        genome_size: u64_json(&case.plan.effective_params, "genome_size"),
        max_memory_gb: u64_json(&case.plan.effective_params, "max_memory_gb")
            .and_then(|value| u32::try_from(value).ok()),
        trusted_kmer_artifact: None,
        conservative_mode: case.conservative_mode,
        input_r1: path_relative_to_repo(repo_root, &input_r1),
        input_r2: input_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        output_r1: path_relative_to_repo(repo_root, &output_r1),
        output_r2: output_r2.as_ref().map(|path| path_relative_to_repo(repo_root, path)),
        report_json: path_relative_to_repo(repo_root, &report_path),
        corrected_reads: Some(reads_in),
        changed_reads: Some(0),
        unchanged_reads: Some(reads_in),
        reads_in: Some(reads_in),
        reads_out: Some(reads_in),
        bases_in: Some(bases_in),
        bases_out: Some(bases_in),
        pairs_in,
        pairs_out: pairs_in,
        mean_q_before: None,
        mean_q_after: None,
        kmer_fix_rate: Some(0.0),
        correction_effect: Some(
            serde_json::json!({ "outputs_changed": false, "probe": "governed_tiny_copy" }),
        ),
        runtime_s: Some(0.0),
        memory_mb: Some(0.0),
        exit_code: Some(0),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: Some(serde_json::json!({ "changed_reads": 0_u64 })),
    };
    bijux_dna_infra::atomic_write_json(&report_path, &report)?;
    Ok(MaterializedFastqProof {
        proof_path: report_path,
        observed_tool_id: case.plan.tool_id.as_str().to_string(),
    })
}
