use super::*;

pub(super) fn smoke_run(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    if matches!(args, [single] if single == "--help" || single == "-h") {
        return success_line("Usage: cargo run -p bijux-dna-dev -- smoke run run -- <fastq|bam>");
    }
    match args.first().map(String::as_str) {
        Some("fastq") if args.len() == 1 => smoke_fastq(workspace, &[]),
        Some("bam") if args.len() == 1 => smoke_bam(workspace, &[]),
        Some(other) => Err(anyhow!("unsupported smoke target: {other}")),
        None => Err(anyhow!("smoke run requires <fastq|bam>")),
    }
}

pub(super) fn smoke_bam(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("smoke-bam", args)?;
    let sample = workspace.path("assets/golden/smoke-inputs-v1/bam/sample.bam");
    if !sample.is_file() {
        return Ok(OpsCommandOutcome::failure(
            "Missing assets/golden/smoke-inputs-v1/bam/sample.bam. Generate it with samtools (see assets/golden/README.md).\n",
        ));
    }
    let output_dir = artifact_root_path(workspace)?.join("smoke_bam");
    bijux_dna_infra::ensure_dir(&output_dir)?;
    let stage = run_program(
        workspace,
        "bijux",
        &[
            "bam".to_string(),
            "stage".to_string(),
            "--stage".to_string(),
            "validate".to_string(),
            "--bam".to_string(),
            sample.display().to_string(),
            "--out".to_string(),
            output_dir.display().to_string(),
            "--sample-id".to_string(),
            "smoke_bam".to_string(),
            "--dry-run".to_string(),
        ],
    )?;
    if !stage.is_success() {
        return Ok(stage);
    }
    let envs = artifact_env(workspace)?;
    run_program_with_env(
        workspace,
        "cargo",
        &[
            "test".to_string(),
            "-p".to_string(),
            "bijux-dna-api".to_string(),
            "bam_smoke_runner_minimal_pipeline_validates_report_section_presence".to_string(),
            "--".to_string(),
            "--exact".to_string(),
        ],
        &envs,
    )
}

pub(super) fn smoke_fastq(workspace: &Workspace, args: &[String]) -> Result<OpsCommandOutcome> {
    ensure_help_only("smoke-fastq", args)?;
    run_program(
        workspace,
        "bijux",
        &[
            "fastq".to_string(),
            "preprocess".to_string(),
            "--r1".to_string(),
            "assets/golden/smoke-inputs-v1/fastq/se/reads.fastq".to_string(),
            "--out".to_string(),
            artifact_root_path(workspace)?
                .join("smoke_fastq")
                .display()
                .to_string(),
            "--sample-id".to_string(),
            "smoke_fastq".to_string(),
            "--dry-run".to_string(),
        ],
    )
}
