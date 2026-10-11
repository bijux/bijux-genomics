//! Owner: bijux-dna-bench-workflows
//! Ancient DNA workflow.
use super::{
    anyhow, bail, materialize_adna_pipeline_input_fastqs, path_relative_to_repo, timestamp_marker,
    AdnaMicroPipelineReport, AdnaMicroPipelineRowStatus, AlignmentFixtureOperations, Instant, Path,
    PathBuf, Result, ADNA_MICRO_PIPELINE_COMMAND, ADNA_MICRO_PIPELINE_ID,
    ADNA_MICRO_PIPELINE_SCHEMA_VERSION, ADNA_PIPELINE_SAMPLE_ID, ADNA_REFERENCE_FASTA_PATH,
    DEFAULT_ADNA_MICRO_PIPELINE_PATH, GOVERNED_MICRO_ELAPSED_SECONDS, GOVERNED_MICRO_FINISHED_AT,
    GOVERNED_MICRO_STARTED_AT,
};
use bijux_dna_core::id_catalog as stage;

/// Execute admitted local stages and publish the workflow's ordered evidence and handoffs.
///
/// # Errors
/// Rejects invalid inputs/output ownership and propagates stage, handoff and publication failures.
pub fn render_adna_micro_pipeline(
    inputs: &AlignmentFixtureOperations,
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AdnaMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let governed_output =
        path_relative_to_repo(repo_root, &absolute_output_path) == DEFAULT_ADNA_MICRO_PIPELINE_PATH;
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("aDNA micro pipeline output has no parent directory"))?;

    let reference_fasta = repo_root.join(ADNA_REFERENCE_FASTA_PATH);
    if !reference_fasta.is_file() {
        bail!("aDNA micro pipeline is missing required reference input");
    }
    bijux_dna_db_ref::build_reference_fasta_index(&std::fs::read(&reference_fasta)?)?;
    crate::execution::output::prepare_output_root(
        repo_root,
        output_root,
        &absolute_output_path,
        ADNA_MICRO_PIPELINE_ID,
        ADNA_MICRO_PIPELINE_SCHEMA_VERSION,
    )?;
    let input_fixtures = materialize_adna_pipeline_input_fastqs(output_root, &reference_fasta)?;

    let started_at =
        if governed_output { GOVERNED_MICRO_STARTED_AT.to_string() } else { timestamp_marker() };
    let started = Instant::now();

    let rows = super::execution::stages::execute_stages(
        inputs,
        repo_root,
        output_root,
        &reference_fasta,
        &input_fixtures,
    )?;

    let handoffs = super::handoffs::build_handoffs(repo_root, &rows)?;

    let skipped_count =
        rows.iter().filter(|row| row.status == AdnaMicroPipelineRowStatus::Skipped).count();
    let succeeded_stage_ids = rows
        .iter()
        .filter(|row| row.status == AdnaMicroPipelineRowStatus::Succeeded)
        .map(|row| row.stage_id.as_str())
        .collect::<Vec<_>>();
    let passes_behavior_test = succeeded_stage_ids.contains(&stage::FASTQ_TRIM_TERMINAL_DAMAGE)
        && succeeded_stage_ids.contains(&stage::FASTQ_DEDUPLICATE)
        && succeeded_stage_ids.contains(&stage::BAM_ALIGN)
        && succeeded_stage_ids.contains(&stage::BAM_DAMAGE)
        && succeeded_stage_ids.contains(&stage::BAM_AUTHENTICITY)
        && succeeded_stage_ids.contains(&stage::VCF_CALL_PSEUDOHAPLOID)
        && handoffs.iter().all(|handoff| handoff.accepted)
        && rows.iter().all(|row| {
            row.status == AdnaMicroPipelineRowStatus::Succeeded
                || (row.status == AdnaMicroPipelineRowStatus::Skipped
                    && !row.reason.trim().is_empty())
        });

    let report = AdnaMicroPipelineReport {
        schema_version: ADNA_MICRO_PIPELINE_SCHEMA_VERSION,
        command: ADNA_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: ADNA_MICRO_PIPELINE_ID,
        sample_id: ADNA_PIPELINE_SAMPLE_ID.to_string(),
        reference_fasta_path: path_relative_to_repo(repo_root, &reference_fasta),
        started_at,
        finished_at: if governed_output {
            GOVERNED_MICRO_FINISHED_AT.to_string()
        } else {
            timestamp_marker()
        },
        elapsed_seconds: if governed_output {
            GOVERNED_MICRO_ELAPSED_SECONDS
        } else {
            started.elapsed().as_secs_f64()
        },
        stage_count: rows.len(),
        handoff_count: handoffs.len(),
        skipped_count,
        passes_behavior_test,
        rows,
        handoffs,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}
