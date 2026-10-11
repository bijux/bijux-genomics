//! Owner: bijux-dna-bench-workflows
//! Core germline workflow.
use super::{
    anyhow, bail, materialize_pipeline_input_fastqs, path_relative_to_repo, timestamp_marker,
    CoreGermlineMicroPipelineReport, CoreGermlineWorkflowInputs, Instant, Path, PathBuf, Result,
    CORE_GERMLINE_MICRO_PIPELINE_COMMAND, CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION,
    CORE_GERMLINE_PIPELINE_ID, DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH,
    GOVERNED_MICRO_ELAPSED_SECONDS, GOVERNED_MICRO_FINISHED_AT, GOVERNED_MICRO_STARTED_AT,
    PIPELINE_SAMPLE_ID, REFERENCE_FASTA_PATH,
};

/// Execute admitted local stages and publish the workflow's ordered evidence and handoffs.
///
/// # Errors
/// Rejects invalid inputs/output ownership and propagates stage, handoff and publication failures.
pub fn render_core_germline_micro_pipeline(
    inputs: &CoreGermlineWorkflowInputs,
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<CoreGermlineMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let governed_output = path_relative_to_repo(repo_root, &absolute_output_path)
        == DEFAULT_CORE_GERMLINE_MICRO_PIPELINE_PATH;
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("core germline micro pipeline output has no parent directory"))?;

    let reference_fasta = repo_root.join(REFERENCE_FASTA_PATH);
    if !reference_fasta.is_file() {
        bail!("core germline micro pipeline is missing required reference input");
    }
    bijux_dna_db_ref::build_reference_fasta_index(&std::fs::read(&reference_fasta)?)?;
    crate::execution::output::prepare_output_root(
        repo_root,
        output_root,
        &absolute_output_path,
        CORE_GERMLINE_PIPELINE_ID,
        CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION,
    )?;
    let input_fixtures = materialize_pipeline_input_fastqs(output_root, &reference_fasta)?;

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

    let passes_behavior_test = rows.len() == 12 && handoffs.iter().all(|handoff| handoff.accepted);
    let finished_at =
        if governed_output { GOVERNED_MICRO_FINISHED_AT.to_string() } else { timestamp_marker() };
    let report = CoreGermlineMicroPipelineReport {
        schema_version: CORE_GERMLINE_MICRO_PIPELINE_SCHEMA_VERSION,
        command: CORE_GERMLINE_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: CORE_GERMLINE_PIPELINE_ID,
        sample_id: PIPELINE_SAMPLE_ID.to_string(),
        reference_fasta_path: path_relative_to_repo(repo_root, &reference_fasta),
        started_at,
        finished_at,
        elapsed_seconds: if governed_output {
            GOVERNED_MICRO_ELAPSED_SECONDS
        } else {
            started.elapsed().as_secs_f64()
        },
        stage_count: rows.len(),
        handoff_count: handoffs.len(),
        passes_behavior_test,
        rows,
        handoffs,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)?;
    Ok(report)
}
