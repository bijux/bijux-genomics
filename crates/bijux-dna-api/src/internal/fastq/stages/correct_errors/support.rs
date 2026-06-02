use super::{
    anyhow, execute_observer_command, hash_file_sha256, input_fastq_stats, parse_seqkit_stats,
    Context, CorrectErrorsReportV1, FastqCorrectMetrics, FastqCorrectParams, Path, PathBuf, Result,
    RuntimeKind, SeqkitMetrics, StagePlanV1, StageResultV1, CORRECT_ERRORS_REPORT_SCHEMA_VERSION,
    STAGE_CORRECT_ERRORS,
};
use bijux_dna_core::prelude::ToolExecutionSpecV1;

pub(super) fn apply_thread_override(
    tool_spec: &ToolExecutionSpecV1,
    threads: Option<u32>,
) -> ToolExecutionSpecV1 {
    let mut spec = tool_spec.clone();
    if let Some(threads) = threads {
        spec.resources.threads = threads.max(1);
    }
    spec
}

pub(super) fn apply_memory_override(
    tool_spec: &ToolExecutionSpecV1,
    max_memory_gb: Option<u32>,
) -> ToolExecutionSpecV1 {
    let mut spec = tool_spec.clone();
    if let Some(max_memory_gb) = max_memory_gb {
        spec.resources.mem_gb = max_memory_gb.max(1);
    }
    spec
}

pub(super) fn parse_quality_encoding(
    value: Option<&str>,
) -> Result<bijux_dna_domain_fastq::params::correct::QualityEncoding> {
    match value.unwrap_or("phred33") {
        "phred33" => Ok(bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred33),
        "phred64" => Ok(bijux_dna_domain_fastq::params::correct::QualityEncoding::Phred64),
        other => Err(anyhow!(
            "unsupported fastq.correct_errors quality_encoding `{other}`; expected one of: phred33, phred64"
        )),
    }
}

pub(super) fn required_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Result<PathBuf> {
    optional_plan_output_path(plan, output_id).ok_or_else(|| {
        anyhow!(
            "correct_errors plan is missing governed output `{output_id}` for tool {}",
            plan.tool_id.as_str()
        )
    })
}

pub(super) fn optional_plan_output_path(plan: &StagePlanV1, output_id: &str) -> Option<PathBuf> {
    plan.io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == output_id)
        .map(|artifact| artifact.path.clone())
}

pub(super) fn correct_metrics_from_observed_stats(
    input_r1: &SeqkitMetrics,
    input_r2: Option<&SeqkitMetrics>,
    output_r1: &SeqkitMetrics,
    output_r2: Option<&SeqkitMetrics>,
    outputs_changed: bool,
) -> FastqCorrectMetrics {
    let reads_in = input_r1.reads + input_r2.map_or(0, |metrics| metrics.reads);
    let reads_out = output_r1.reads + output_r2.map_or(0, |metrics| metrics.reads);
    let bases_in = input_r1.bases + input_r2.map_or(0, |metrics| metrics.bases);
    let bases_out = output_r1.bases + output_r2.map_or(0, |metrics| metrics.bases);
    let mean_q_before = weighted_mean_q(input_r1, input_r2);
    let mean_q_after = weighted_mean_q(output_r1, output_r2);
    FastqCorrectMetrics {
        reads_in,
        reads_out,
        bases_in,
        bases_out,
        pairs_in: input_r2.map(|metrics| input_r1.reads.min(metrics.reads)),
        pairs_out: output_r2.map(|metrics| output_r1.reads.min(metrics.reads)),
        mean_q_before,
        mean_q_after,
        kmer_fix_rate: kmer_fix_rate_proxy(mean_q_before, mean_q_after, outputs_changed),
    }
}

fn weighted_mean_q(primary: &SeqkitMetrics, secondary: Option<&SeqkitMetrics>) -> f64 {
    let total_bases = primary.bases + secondary.map_or(0, |metrics| metrics.bases);
    if total_bases == 0 {
        return 0.0;
    }
    let secondary_weighted_mean =
        secondary.map_or(0.0, |metrics| metrics.mean_q * u64_to_f64(metrics.bases));
    ((primary.mean_q * u64_to_f64(primary.bases)) + secondary_weighted_mean)
        / u64_to_f64(total_bases)
}

fn u64_to_f64(value: u64) -> f64 {
    value.to_string().parse::<f64>().unwrap_or(0.0)
}

fn kmer_fix_rate_proxy(mean_q_before: f64, mean_q_after: f64, outputs_changed: bool) -> f64 {
    if !outputs_changed {
        return 0.0;
    }
    if mean_q_after <= mean_q_before {
        return f64::EPSILON;
    }
    ((mean_q_after - mean_q_before) / mean_q_after.max(1.0)).clamp(f64::EPSILON, 1.0)
}

fn decode_effective_correct_params(
    effective_params: &serde_json::Value,
) -> Result<FastqCorrectParams> {
    serde_json::from_value(effective_params.clone()).context("decode effective correction params")
}

pub(super) fn count_corrected_read_changes(
    input_r1: &Path,
    input_r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
) -> Result<(u64, u64)> {
    let (changed_r1, unchanged_r1) =
        bijux_dna_domain_fastq::stages::contract::count_changed_fastq_reads(input_r1, output_r1)?;
    let (changed_r2, unchanged_r2) = match (input_r2, output_r2) {
        (Some(input_r2), Some(output_r2)) => {
            bijux_dna_domain_fastq::stages::contract::count_changed_fastq_reads(
                input_r2, output_r2,
            )?
        }
        (None, None) => (0, 0),
        _ => {
            return Err(anyhow!(
                "correct_errors changed-read counting requires matched input/output mate presence"
            ));
        }
    };
    Ok((changed_r1 + changed_r2, unchanged_r1 + unchanged_r2))
}

#[allow(clippy::too_many_arguments)]
pub(super) fn build_correction_report(
    tool: &str,
    input_r1: &Path,
    input_r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
    report_json: &Path,
    effective_params: &serde_json::Value,
    metrics: &FastqCorrectMetrics,
    changed_reads: u64,
    unchanged_reads: u64,
    execution: &StageResultV1,
    outputs_changed: bool,
) -> Result<CorrectErrorsReportV1> {
    let effective_params = decode_effective_correct_params(effective_params)?;
    Ok(CorrectErrorsReportV1 {
        schema_version: CORRECT_ERRORS_REPORT_SCHEMA_VERSION.to_string(),
        stage: STAGE_CORRECT_ERRORS.as_str().to_string(),
        stage_id: STAGE_CORRECT_ERRORS.as_str().to_string(),
        tool_id: tool.to_string(),
        paired_mode: effective_params.paired_mode,
        threads: effective_params.threads,
        correction_engine: effective_params.correction_engine,
        quality_encoding: effective_params.quality_encoding,
        kmer_size: effective_params.kmer_size,
        musket_kmer_budget: effective_params.musket_kmer_budget,
        genome_size: effective_params.genome_size,
        max_memory_gb: effective_params.max_memory_gb,
        trusted_kmer_artifact: effective_params.trusted_kmer_artifact,
        conservative_mode: effective_params.conservative_mode,
        input_r1: input_r1.display().to_string(),
        input_r2: input_r2.map(|path| path.display().to_string()),
        output_r1: output_r1.display().to_string(),
        output_r2: output_r2.map(|path| path.display().to_string()),
        report_json: report_json.display().to_string(),
        corrected_reads: Some(metrics.reads_out),
        changed_reads: Some(changed_reads),
        unchanged_reads: Some(unchanged_reads),
        reads_in: Some(metrics.reads_in),
        reads_out: Some(metrics.reads_out),
        bases_in: Some(metrics.bases_in),
        bases_out: Some(metrics.bases_out),
        pairs_in: metrics.pairs_in,
        pairs_out: metrics.pairs_out,
        mean_q_before: Some(metrics.mean_q_before),
        mean_q_after: Some(metrics.mean_q_after),
        kmer_fix_rate: Some(metrics.kmer_fix_rate),
        correction_effect: Some(serde_json::json!({
            "outputs_changed": outputs_changed,
            "reads_delta": i128::from(metrics.reads_out) - i128::from(metrics.reads_in),
            "bases_delta": i128::from(metrics.bases_out) - i128::from(metrics.bases_in),
            "mean_q_delta": metrics.mean_q_after - metrics.mean_q_before,
        })),
        runtime_s: Some(execution.runtime_s),
        memory_mb: Some(execution.memory_mb),
        exit_code: Some(execution.exit_code),
        raw_backend_report: None,
        raw_backend_report_format: None,
        backend_metrics: None,
    })
}

pub(super) fn input_output_content_changed(
    input_r1: &Path,
    input_r2: Option<&Path>,
    output_r1: &Path,
    output_r2: Option<&Path>,
) -> Result<bool> {
    let primary_changed = hash_file_sha256(input_r1)? != hash_file_sha256(output_r1)?;
    let secondary_changed = match (input_r2, output_r2) {
        (Some(input_r2), Some(output_r2)) => {
            hash_file_sha256(input_r2)? != hash_file_sha256(output_r2)?
        }
        (None, None) => false,
        _ => true,
    };
    Ok(primary_changed || secondary_changed)
}

pub(super) fn observe_fastq_stats(
    seqkit_image: &str,
    runner: RuntimeKind,
    reads: &std::path::Path,
) -> Result<SeqkitMetrics> {
    let reads_dir = reads.parent().ok_or_else(|| anyhow!("reads path has no parent"))?;
    let stats_spec = input_fastq_stats(reads_dir, reads)?;
    let stats_output = execute_observer_command(
        seqkit_image,
        stats_spec.mount_dir.as_path(),
        &stats_spec.args,
        runner,
    )?;
    if stats_output.exit_code != 0 {
        return Err(anyhow!(
            "seqkit correction observer failed for {}: {}",
            reads.display(),
            stats_output.stderr
        ));
    }
    parse_seqkit_stats(&stats_output.stdout)
}

pub(super) fn benchmark_query_context() -> Result<bijux_dna_domain_fastq::BenchQueryContext> {
    bijux_dna_domain_fastq::governed_stage_bench_query_context(STAGE_CORRECT_ERRORS.as_str())
}
