use super::*;

pub(super) fn lab_run_bench(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("run-bench", args)?;
    ensure_artifact_root_inside_artifacts(workspace)?;
    let config = lab_config(workspace)?;
    let corpus_root = required_config_string(&config, "corpus_root", "lab config")?;
    let runner_kind = required_config_string(&config, "runner_kind", "lab config")?;
    let output_dir = required_config_string(&config, "output_dir", "lab config")?;
    let fastq = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "fastq".to_string(),
            "--runner".to_string(),
            runner_kind.clone(),
            "--corpus-root".to_string(),
            corpus_root.clone(),
            "--out".to_string(),
            output_dir.clone(),
        ],
    )?;
    if !fastq.is_success() {
        return Ok(fastq);
    }
    let bam = run_program(
        workspace,
        "cargo",
        &[
            "run".to_string(),
            "--bin".to_string(),
            "bijux-dna".to_string(),
            "--".to_string(),
            "bench".to_string(),
            "bam".to_string(),
            "--runner".to_string(),
            runner_kind,
            "--corpus-root".to_string(),
            corpus_root,
            "--out".to_string(),
            output_dir,
        ],
    )?;
    Ok(merge_outcomes(
        OpsCommandOutcome::success(fastq.stdout),
        bam,
    ))
}

pub(super) fn lab_run_pipelines(
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    ensure_help_only("run-pipelines", args)?;
    ensure_artifact_root_inside_artifacts(workspace)?;
    let config = lab_config(workspace)?;
    let corpus_root = required_config_string(&config, "corpus_root", "lab config")?;
    let runner_kind = required_config_string(&config, "runner_kind", "lab config")?;
    let output_dir = required_config_string(&config, "output_dir", "lab config")?;
    let pipeline_ids = required_config_string(&config, "pipeline_ids", "lab config")?;
    let mut aggregate = OpsCommandOutcome::success(String::new());
    for pipeline in pipeline_ids
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let outcome = run_program(
            workspace,
            "cargo",
            &[
                "run".to_string(),
                "--bin".to_string(),
                "bijux-dna".to_string(),
                "--".to_string(),
                "run".to_string(),
                "--pipeline".to_string(),
                pipeline.to_string(),
                "--runner".to_string(),
                runner_kind.clone(),
                "--corpus-root".to_string(),
                corpus_root.clone(),
                "--out".to_string(),
                output_dir.clone(),
            ],
        )?;
        aggregate = merge_outcomes(aggregate, outcome);
        if !aggregate.is_success() {
            return Ok(aggregate);
        }
    }
    Ok(aggregate)
}
