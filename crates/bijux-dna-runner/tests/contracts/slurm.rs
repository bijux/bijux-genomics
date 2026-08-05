#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;

use bijux_dna_runner::api::{
    cancel_slurm_job, collect_slurm_job, observe_slurm_job, submit_slurm_batch, submit_slurm_job,
    SlurmClientPaths, SlurmEvidenceSource, SlurmJobState, SlurmSubmitRequest,
};

fn write_executable(path: &Path, body: &str) -> anyhow::Result<()> {
    std::fs::write(path, body)?;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

fn shell_literal(path: &Path) -> String {
    format!("'{}'", path.display().to_string().replace('\'', "'\\''"))
}

fn client(temp: &tempfile::TempDir) -> anyhow::Result<SlurmClientPaths> {
    let paths = SlurmClientPaths {
        sbatch: temp.path().join("sbatch"),
        squeue: temp.path().join("squeue"),
        sacct: temp.path().join("sacct"),
        scancel: temp.path().join("scancel"),
    };
    for path in [&paths.sbatch, &paths.squeue, &paths.sacct, &paths.scancel] {
        write_executable(path, "#!/bin/sh\nexit 0\n")?;
    }
    Ok(paths)
}

fn request(temp: &tempfile::TempDir, run_id: &str) -> anyhow::Result<SlurmSubmitRequest> {
    let script_path = temp.path().join(format!("{run_id}.sbatch"));
    std::fs::write(&script_path, "#!/bin/sh\nexit 0\n")?;
    Ok(SlurmSubmitRequest {
        run_id: run_id.to_string(),
        script_path,
        working_directory: temp.path().to_path_buf(),
        stdout_path: temp.path().join(format!("{run_id}.stdout.log")),
        stderr_path: temp.path().join(format!("{run_id}.stderr.log")),
        submitted_at_utc: "2026-08-04T20:00:00Z".to_string(),
    })
}

fn submitted_handle(
    temp: &tempfile::TempDir,
    paths: &SlurmClientPaths,
) -> anyhow::Result<bijux_dna_runner::api::SlurmJobHandle> {
    write_executable(&paths.sbatch, "#!/bin/sh\nprintf '3459974;cluster\\n'\n")?;
    submit_slurm_job(paths, &request(temp, "run-2042")?)
}

#[test]
fn submission_returns_identity_without_scheduler_polling() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let argument_log = temp.path().join("sbatch.args");
    write_executable(
        &paths.sbatch,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > {}\nprintf '3459974;cluster\\n'\n",
            shell_literal(&argument_log)
        ),
    )?;

    let handle = submit_slurm_job(&paths, &request(&temp, "run-2042")?)?;

    assert_eq!(handle.job_id, "3459974");
    let arguments = std::fs::read_to_string(argument_log)?;
    assert!(arguments.contains("--parsable"));
    assert!(arguments.contains("--chdir"));
    assert!(arguments.contains("--output"));
    assert!(arguments.contains("--error"));
    Ok(())
}

#[test]
fn bounded_submission_refuses_an_oversized_batch() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let requests = vec![request(&temp, "run-2042")?, request(&temp, "run-2043")?];

    assert!(submit_slurm_batch(&paths, &requests, 1).is_err());
    assert!(submit_slurm_batch(&paths, &requests, 0).is_err());
    Ok(())
}

#[test]
fn bounded_submission_retains_handles_when_a_later_submission_fails() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    write_executable(
        &paths.sbatch,
        "#!/bin/sh\ncase \"$*\" in\n  *run-2043.sbatch*) echo rejected >&2; exit 1 ;;\n  *) printf '3459974\\n' ;;\nesac\n",
    )?;
    let requests = vec![request(&temp, "run-2042")?, request(&temp, "run-2043")?];

    let report = submit_slurm_batch(&paths, &requests, 2)?;

    assert_eq!(report.submitted.len(), 1);
    assert_eq!(report.submitted[0].run_id, "run-2042");
    assert_eq!(report.failures.len(), 1);
    assert_eq!(report.failures[0].run_id, "run-2043");
    assert!(report.failures[0].error.contains("rejected"));
    Ok(())
}

#[test]
fn queue_observation_does_not_consult_accounting() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    write_executable(&paths.squeue, "#!/bin/sh\nprintf 'RUNNING\\n'\n")?;
    write_executable(&paths.sacct, "#!/bin/sh\nexit 99\n")?;

    let observation = observe_slurm_job(&paths, &handle, "2026-08-04T20:10:00Z")?;

    assert_eq!(observation.state, SlurmJobState::Running);
    assert_eq!(observation.source, SlurmEvidenceSource::Queue);
    assert!(observation.accounting.is_none());
    Ok(())
}

#[test]
fn completed_job_queue_error_falls_through_to_accounting() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    write_executable(
        &paths.squeue,
        "#!/bin/sh\nprintf '%s\n' 'slurm_load_jobs error: Invalid job id specified' >&2\nexit 1\n",
    )?;
    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\n' '3459974|COMPLETED|0:0|2|00:00:01.750|32K|n001|None'\n",
    )?;

    let observed = observe_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")?;

    assert_eq!(observed.state, SlurmJobState::Completed);
    assert_eq!(observed.source, SlurmEvidenceSource::Accounting);
    assert!(observed.accounting.is_some());
    Ok(())
}

#[test]
fn unrelated_queue_failure_remains_fatal() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    write_executable(&paths.squeue, "#!/bin/sh\nprintf 'permission denied\n' >&2\nexit 1\n")?;

    let error = observe_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")
        .err()
        .ok_or_else(|| anyhow::anyhow!("unrelated queue failure was accepted"))?;

    assert!(format!("{error:#}").contains("permission denied"));
    Ok(())
}

#[test]
fn terminal_accounting_parses_exit_resources_and_failure_states() -> anyhow::Result<()> {
    for (raw_state, expected) in [
        ("COMPLETED", SlurmJobState::Completed),
        ("FAILED", SlurmJobState::Failed),
        ("CANCELLED by 123", SlurmJobState::Cancelled),
        ("TIMEOUT", SlurmJobState::Timeout),
        ("PREEMPTED", SlurmJobState::Preempted),
        ("NODE_FAIL", SlurmJobState::NodeFailure),
        ("OUT_OF_MEMORY", SlurmJobState::OutOfMemory),
    ] {
        let temp = tempfile::tempdir()?;
        let paths = client(&temp)?;
        let handle = submitted_handle(&temp, &paths)?;
        write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
        write_executable(
            &paths.sacct,
            &format!(
                "#!/bin/sh\nprintf '%s\\n' '3459974|{raw_state}|1:9|125|00:01:30|2048K|n001|scheduler reason'\n"
            ),
        )?;

        let observed = collect_slurm_job(&paths, &handle, "2026-08-04T20:20:00Z")?;
        let accounting =
            observed.accounting.ok_or_else(|| anyhow::anyhow!("terminal job lacks accounting"))?;

        assert_eq!(observed.state, expected);
        assert_eq!(accounting.exit_code, 1);
        assert_eq!(accounting.signal, 9);
        assert_eq!(accounting.elapsed_seconds, 125);
        assert_eq!(accounting.total_cpu_seconds, 90);
        assert_eq!(accounting.max_rss_bytes, Some(2 * 1024 * 1024));
        assert_eq!(accounting.node_list, "n001");
    }
    Ok(())
}

#[test]
fn accounting_uses_allocation_state_and_peak_step_memory() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\n' \
'3459974|COMPLETED|0:0|60|1-00:00:58||n001|None' \
'3459974.batch|COMPLETED|0:0|60|1-00:00:58|2048K|n001|None' \
'3459974.extern|COMPLETED|0:0|60|00:00:01|1024K|n001|None'\n",
    )?;

    let observed = collect_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")?;
    let accounting = observed.accounting.ok_or_else(|| anyhow::anyhow!("missing accounting"))?;

    assert_eq!(observed.state, SlurmJobState::Completed);
    assert_eq!(accounting.total_cpu_seconds, 86_458);
    assert_eq!(accounting.max_rss_bytes, Some(2 * 1024 * 1024));
    Ok(())
}

#[test]
fn accounting_accepts_fractional_total_cpu_reported_by_slurm() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\n' '3459974|COMPLETED|0:0|2|00:00:01.750|32K|n001|None'\n",
    )?;

    let observed = collect_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")?;
    let accounting = observed.accounting.ok_or_else(|| anyhow::anyhow!("missing accounting"))?;

    assert_eq!(accounting.total_cpu_seconds, 1);
    assert_eq!(accounting.max_rss_bytes, Some(32 * 1024));
    Ok(())
}

#[test]
fn missing_accounting_is_explicit_and_duplicate_collection_is_idempotent() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(&paths.sacct, "#!/bin/sh\nexit 0\n")?;

    let missing = collect_slurm_job(&paths, &handle, "2026-08-04T20:20:00Z")?;
    assert_eq!(missing.state, SlurmJobState::AccountingPending);
    assert!(missing.accounting.is_none());

    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\\n' '3459974|COMPLETED|0:0|60|00:00:58|1G|n001|None'\n",
    )?;
    let first = collect_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")?;
    let second = collect_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")?;
    assert_eq!(first, second);
    Ok(())
}

#[test]
fn cancellation_uses_exact_job_identity_and_propagates_failure() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let handle = submitted_handle(&temp, &paths)?;
    let cancellation_log = temp.path().join("cancelled-job");
    write_executable(
        &paths.scancel,
        &format!("#!/bin/sh\nprintf '%s' \"$1\" > {}\n", shell_literal(&cancellation_log)),
    )?;

    cancel_slurm_job(&paths, &handle)?;
    assert_eq!(std::fs::read_to_string(cancellation_log)?, "3459974");

    write_executable(&paths.scancel, "#!/bin/sh\necho denied >&2\nexit 1\n")?;
    assert!(cancel_slurm_job(&paths, &handle).is_err());
    Ok(())
}

#[test]
fn malformed_scheduler_evidence_fails_closed() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    write_executable(&paths.sbatch, "#!/bin/sh\nprintf 'not-a-job\\n'\n")?;
    assert!(submit_slurm_job(&paths, &request(&temp, "run-2042")?).is_err());

    let handle = submitted_handle(&temp, &paths)?;
    write_executable(&paths.squeue, "#!/bin/sh\nprintf 'MYSTERY\\n'\n")?;
    assert!(observe_slurm_job(&paths, &handle, "2026-08-04T20:10:00Z").is_err());

    write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\\n' '3459974|COMPLETED|0:0|60|18446744073709551615:00:00||n001|None'\n",
    )?;
    assert!(collect_slurm_job(&paths, &handle, "2026-08-04T20:20:00Z").is_err());
    Ok(())
}
