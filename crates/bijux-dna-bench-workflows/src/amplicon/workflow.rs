//! Owner: bijux-dna-bench-workflows
//! Amplicon workflow.
use super::{
    anyhow, build_handoffs, load_amplicon_corpus_fixture_manifest_path, passes_behavior_test,
    path_relative_to_repo, run_amplicon_corpus_fixture_stage, run_amplicon_output_judgment_stage,
    run_amplicon_truth_fixture_stage, run_cluster_otus_stage, run_infer_asvs_stage,
    run_normalize_abundance_stage, run_normalize_primers_stage, run_remove_chimeras_stage,
    timestamp_marker, AmpliconMicroPipelineReport, Context, Instant, Path, PathBuf, Result,
    AMPLICON_MICRO_PIPELINE_COMMAND, AMPLICON_MICRO_PIPELINE_ID,
    AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION, DEFAULT_AMPLICON_MICRO_PIPELINE_PATH,
    DEFAULT_AMPLICON_TRUTH_MANIFEST_PATH, DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH,
    GOVERNED_MICRO_ELAPSED_SECONDS, GOVERNED_MICRO_FINISHED_AT, GOVERNED_MICRO_STARTED_AT,
};

/// Execute admitted local stages and publish the workflow's ordered evidence and handoffs.
///
/// # Errors
/// Rejects invalid inputs/output ownership and propagates stage, handoff and publication failures.
pub fn render_amplicon_micro_pipeline(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<AmpliconMicroPipelineReport> {
    let absolute_output_path =
        if output_path.is_absolute() { output_path } else { repo_root.join(output_path) };
    let governed_output = path_relative_to_repo(repo_root, &absolute_output_path)
        == DEFAULT_AMPLICON_MICRO_PIPELINE_PATH;
    let output_root = absolute_output_path
        .parent()
        .ok_or_else(|| anyhow!("amplicon micro pipeline output has no parent directory"))?;

    let corpus_manifest_path = repo_root.join(DEFAULT_CORPUS_03_AMPLICON_MANIFEST_PATH);
    let truth_manifest_path = repo_root.join(DEFAULT_AMPLICON_TRUTH_MANIFEST_PATH);
    super::validate_amplicon_corpus_fixture_manifest_path(repo_root, &corpus_manifest_path)?;
    super::validate_amplicon_truth_manifest_path(repo_root, &truth_manifest_path)?;
    crate::execution::output::prepare_output_root(
        repo_root,
        output_root,
        &absolute_output_path,
        AMPLICON_MICRO_PIPELINE_ID,
        AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION,
    )?;
    let started_at =
        if governed_output { GOVERNED_MICRO_STARTED_AT.to_string() } else { timestamp_marker() };
    let started = Instant::now();

    let corpus_stage =
        run_amplicon_corpus_fixture_stage(repo_root, output_root, &corpus_manifest_path)?;
    let truth_stage =
        run_amplicon_truth_fixture_stage(repo_root, output_root, &truth_manifest_path)?;
    let normalize_stage = run_normalize_primers_stage(repo_root, output_root)?;
    let infer_stage = run_infer_asvs_stage(repo_root, output_root)?;
    let chimera_stage = run_remove_chimeras_stage(repo_root, output_root)?;
    let otu_stage = run_cluster_otus_stage(repo_root, output_root)?;
    let abundance_stage =
        run_normalize_abundance_stage(repo_root, output_root, &corpus_manifest_path)?;
    let stages = super::execution::AmpliconStageArtifacts {
        corpus: corpus_stage,
        truth: truth_stage,
        normalize: normalize_stage,
        infer: infer_stage,
        chimera: chimera_stage,
        otu: otu_stage,
        abundance: abundance_stage,
    };
    let judgment_row = run_amplicon_output_judgment_stage(repo_root, output_root, &stages)?;
    let rows = vec![
        stages.corpus.row.clone(),
        stages.truth.row.clone(),
        stages.normalize.row.clone(),
        stages.infer.row.clone(),
        stages.chimera.row.clone(),
        stages.otu.row.clone(),
        stages.abundance.row.clone(),
        judgment_row.clone(),
    ];
    let handoffs = build_handoffs(repo_root, &stages, &judgment_row);
    let sample_count =
        load_amplicon_corpus_fixture_manifest_path(&corpus_manifest_path)?.samples.len();
    let passes_behavior_test = passes_behavior_test(&rows, &handoffs);

    let report = AmpliconMicroPipelineReport {
        schema_version: AMPLICON_MICRO_PIPELINE_SCHEMA_VERSION,
        command: AMPLICON_MICRO_PIPELINE_COMMAND,
        output_path: path_relative_to_repo(repo_root, &absolute_output_path),
        pipeline_id: AMPLICON_MICRO_PIPELINE_ID,
        corpus_manifest_path: path_relative_to_repo(repo_root, &corpus_manifest_path),
        truth_manifest_path: path_relative_to_repo(repo_root, &truth_manifest_path),
        sample_count,
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
        passes_behavior_test,
        rows,
        handoffs,
    };
    bijux_dna_infra::atomic_write_json(&absolute_output_path, &report)
        .with_context(|| format!("write {}", absolute_output_path.display()))?;
    Ok(report)
}
