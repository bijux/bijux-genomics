use super::publish_rendered_command_artifacts;
use anyhow::Result;
use bijux_dna_bench_model::{AllDomainRenderedCommandArgvRow, AllDomainRenderedCommandRow};
use std::path::Path;

fn command() -> Result<AllDomainRenderedCommandRow> {
    Ok(serde_json::from_value(serde_json::json!({
        "result_id":"fastq:local:fastq.trim_reads:reads:fastp", "domain":"fastq",
        "stage_id":"fastq.trim_reads", "tool_id":"fastp", "corpus_id":"local",
        "asset_profile_id":"reads", "readiness_kind":"smoke", "benchmark_status":"benchmark_ready",
        "command_source":"fastq_bam_command_adapter",
        "command_steps":[{"step_id":"invoke", "step_kind":"exec", "consumes_previous_stdout":false,
            "argv":["fastp", "--in1", "reads with spaces.fq", ""],
            "command":"fastp --in1 'reads with spaces.fq' ''"}],
        "script_commands":["fastp --in1 'reads with spaces.fq' ''"]
    }))?)
}

#[test]
fn command_artifacts_preserve_argv_and_prior_files_when_observation_admission_fails() -> Result<()>
{
    let root = bijux_dna_infra::temp_dir("command-artifacts-")?;
    let row = command()?;
    let requested = Path::new("reports/commands.sh");
    let report = publish_rendered_command_artifacts(root.path(), requested, vec![row.clone()])?;
    let script = std::fs::read(root.path().join(&report.output_path))?;
    let argv = std::fs::read(root.path().join(&report.argv_output_path))?;
    let decoded: AllDomainRenderedCommandArgvRow = serde_json::from_slice(&argv)?;
    assert_eq!(decoded.command_steps[0].argv, row.command_steps[0].argv);
    assert_eq!(report.row_count, 1);
    assert_eq!(report.result_id_count, 1);
    assert_eq!(report.domain_counts.get("fastq"), Some(&1));
    assert!(String::from_utf8(script.clone())?.contains("fastp --in1 'reads with spaces.fq' ''"));
    assert!(publish_rendered_command_artifacts(root.path(), requested, vec![]).is_err());
    assert!(publish_rendered_command_artifacts(
        root.path(),
        requested,
        vec![row.clone(), row.clone()]
    )
    .is_err());
    let mut malformed = row;
    malformed.command_steps[0].argv.clear();
    assert!(publish_rendered_command_artifacts(root.path(), requested, vec![malformed]).is_err());
    assert_eq!(std::fs::read(root.path().join(&report.output_path))?, script);
    assert_eq!(std::fs::read(root.path().join(&report.argv_output_path))?, argv);
    Ok(())
}

#[test]
fn command_artifact_destination_admission_has_no_directory_or_partial_file_effects() -> Result<()> {
    let root = bijux_dna_infra::temp_dir("command-destinations-")?;
    let repository = root.path().join("repo");
    bijux_dna_infra::ensure_dir(&repository)?;
    let escaped = root.path().join("outside/nested/commands.sh");
    assert!(publish_rendered_command_artifacts(&repository, &escaped, vec![command()?]).is_err());
    assert!(!root.path().join("outside").exists());
    assert!(publish_rendered_command_artifacts(
        &repository,
        Path::new("created/../../outside/commands.sh"),
        vec![command()?]
    )
    .is_err());
    assert!(!repository.join("created").exists());
    let script = repository.join("commands.sh");
    bijux_dna_infra::write_bytes(&script, b"prior script")?;
    bijux_dna_infra::ensure_dir(repository.join("commands.argv.jsonl"))?;
    assert!(publish_rendered_command_artifacts(
        &repository,
        Path::new("commands.sh"),
        vec![command()?]
    )
    .is_err());
    assert_eq!(std::fs::read(&script)?, b"prior script");
    Ok(())
}

#[cfg(unix)]
#[test]
fn command_artifacts_refuse_symlink_ancestors_without_writing_the_target() -> Result<()> {
    let root = bijux_dna_infra::temp_dir("command-symlinks-")?;
    let repository = root.path().join("repo");
    let external = root.path().join("external");
    bijux_dna_infra::ensure_dir(&repository)?;
    bijux_dna_infra::ensure_dir(&external)?;
    std::os::unix::fs::symlink(&external, repository.join("linked"))?;
    assert!(publish_rendered_command_artifacts(
        &repository,
        Path::new("linked/nested/commands.sh"),
        vec![command()?]
    )
    .is_err());
    assert!(!external.join("nested").exists());
    Ok(())
}
