#![cfg(unix)]

use std::os::unix::fs::PermissionsExt as _;
use std::path::Path;
use std::{fs, io::Write as _};

use bijux_dna_runner::api::{
    cancel_slurm_job, collect_slurm_job, observe_slurm_job, submit_slurm_batch, submit_slurm_job,
    SlurmArraySpec, SlurmClientPaths, SlurmDependencyCondition, SlurmEvidenceSource, SlurmJobState,
    SlurmSubmitRequest,
};

fn write_executable(path: &Path, body: &str) -> anyhow::Result<()> {
    let staging = path.with_extension("staging");
    let mut file = fs::OpenOptions::new().write(true).create_new(true).open(&staging)?;
    file.write_all(body.as_bytes())?;
    let mut permissions = file.metadata()?.permissions();
    permissions.set_mode(0o755);
    file.set_permissions(permissions)?;
    file.sync_all()?;
    drop(file);
    fs::rename(staging, path)?;
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
        dependencies: Vec::new(),
        dependency_condition: SlurmDependencyCondition::AfterOk,
        array: None,
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
fn submission_binds_afterok_dependencies_to_arguments_and_handle() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let argument_log = temp.path().join("sbatch.args");
    write_executable(
        &paths.sbatch,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > {}\nprintf '3459974\\n'\n",
            shell_literal(&argument_log)
        ),
    )?;
    let mut request = request(&temp, "run-2042")?;
    request.dependencies = vec!["3459901".to_string(), "3459902".to_string()];

    let handle = submit_slurm_job(&paths, &request)?;

    assert_eq!(handle.dependencies, request.dependencies);
    assert!(std::fs::read_to_string(argument_log)?.contains("--dependency afterok:3459901:3459902"));
    Ok(())
}

#[test]
fn submission_binds_afterany_array_to_arguments_and_handle() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let argument_log = temp.path().join("sbatch.args");
    write_executable(
        &paths.sbatch,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$*\" > {}\nprintf '3459974\\n'\n",
            shell_literal(&argument_log)
        ),
    )?;
    let mut request = request(&temp, "run-2042")?;
    request.dependencies = vec!["3459901".to_string()];
    request.dependency_condition = SlurmDependencyCondition::AfterAny;
    request.array = Some(SlurmArraySpec {
        first_task_id: 3,
        last_task_id: 11,
        maximum_parallel_tasks: Some(4),
    });
    request.stdout_path = temp.path().join("array-%A_%a.stdout.log");
    request.stderr_path = temp.path().join("array-%A_%a.stderr.log");

    let handle = submit_slurm_job(&paths, &request)?;

    assert_eq!(handle.dependency_condition, SlurmDependencyCondition::AfterAny);
    assert_eq!(handle.array, request.array);
    let arguments = std::fs::read_to_string(argument_log)?;
    assert!(arguments.contains("--dependency afterany:3459901"));
    assert!(arguments.contains("--array 3-11%4"));
    Ok(())
}

#[test]
fn submission_rejects_malformed_or_duplicate_dependencies() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    write_executable(&paths.sbatch, "#!/bin/sh\nprintf '3459974\\n'\n")?;
    for dependencies in [
        vec!["3459901".to_string(), "3459901".to_string()],
        vec!["job-3459901".to_string()],
        vec![String::new()],
    ] {
        let mut request = request(&temp, "run-2042")?;
        request.dependencies = dependencies;
        assert!(submit_slurm_job(&paths, &request).is_err());
    }
    let mut condition_without_dependency = request(&temp, "run-2042")?;
    condition_without_dependency.dependency_condition = SlurmDependencyCondition::AfterAny;
    assert!(submit_slurm_job(&paths, &condition_without_dependency).is_err());
    Ok(())
}

#[test]
fn submission_rejects_invalid_array_bounds() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    write_executable(&paths.sbatch, "#!/bin/sh\nprintf '3459974\\n'\n")?;
    for array in [
        SlurmArraySpec { first_task_id: 4, last_task_id: 3, maximum_parallel_tasks: None },
        SlurmArraySpec { first_task_id: 3, last_task_id: 4, maximum_parallel_tasks: Some(0) },
        SlurmArraySpec { first_task_id: 3, last_task_id: 4, maximum_parallel_tasks: Some(3) },
    ] {
        let mut request = request(&temp, "run-2042")?;
        request.array = Some(array);
        assert!(submit_slurm_job(&paths, &request).is_err());
    }
    let mut shared_logs = request(&temp, "run-2042")?;
    shared_logs.array =
        Some(SlurmArraySpec { first_task_id: 3, last_task_id: 4, maximum_parallel_tasks: None });
    assert!(submit_slurm_job(&paths, &shared_logs).is_err());
    Ok(())
}

#[test]
fn request_deserialization_defaults_additive_scheduler_fields() -> anyhow::Result<()> {
    let request = request(&tempfile::tempdir()?, "run-2042")?;
    let mut value = serde_json::to_value(&request)?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("request did not serialize as an object"))?;
    object.remove("dependency_condition");
    object.remove("array");

    let decoded: SlurmSubmitRequest = serde_json::from_value(value)?;

    assert_eq!(decoded.dependency_condition, SlurmDependencyCondition::AfterOk);
    assert!(decoded.array.is_none());
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
fn array_accounting_retains_every_exact_task_and_peak_step_memory() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let mut handle = submitted_handle(&temp, &paths)?;
    handle.array =
        Some(SlurmArraySpec { first_task_id: 3, last_task_id: 4, maximum_parallel_tasks: Some(2) });
    write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\n' \
'3459974|COMPLETED|0:0|60|00:01:58||n001|None' \
'3459974_3|COMPLETED|0:0|31|00:00:30||n001|None' \
'3459974_3.batch|COMPLETED|0:0|31|00:00:30|128M|n001|None' \
'3459974_4|FAILED|2:0|29|00:00:28|64M|n002|bad input'\n",
    )?;

    let observed = collect_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")?;
    let accounting = observed.accounting.ok_or_else(|| anyhow::anyhow!("missing accounting"))?;

    assert_eq!(accounting.array_tasks.len(), 2);
    assert_eq!(accounting.array_tasks[0].task_id, 3);
    assert_eq!(accounting.array_tasks[0].job_id, "3459974_3");
    assert_eq!(accounting.array_tasks[0].max_rss_bytes, Some(128 * 1024 * 1024));
    assert_eq!(accounting.array_tasks[1].task_id, 4);
    assert_eq!(accounting.array_tasks[1].state, SlurmJobState::Failed);
    assert_eq!(accounting.array_tasks[1].reason.as_deref(), Some("bad input"));
    Ok(())
}

#[test]
fn terminal_array_accounting_requires_exact_declared_task_coverage() -> anyhow::Result<()> {
    let temp = tempfile::tempdir()?;
    let paths = client(&temp)?;
    let mut handle = submitted_handle(&temp, &paths)?;
    handle.array =
        Some(SlurmArraySpec { first_task_id: 3, last_task_id: 4, maximum_parallel_tasks: None });
    write_executable(&paths.squeue, "#!/bin/sh\nexit 0\n")?;
    write_executable(
        &paths.sacct,
        "#!/bin/sh\nprintf '%s\n' \
'3459974|COMPLETED|0:0|60|00:00:58||n001|None' \
'3459974_3|COMPLETED|0:0|60|00:00:58|1G|n001|None'\n",
    )?;

    let error = collect_slurm_job(&paths, &handle, "2026-08-04T20:30:00Z")
        .err()
        .ok_or_else(|| anyhow::anyhow!("incomplete array accounting was accepted"))?;

    assert!(format!("{error:#}").contains("has 1 tasks instead of 2"));
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
fn accounting_deserialization_defaults_additive_array_tasks() -> anyhow::Result<()> {
    let accounting: bijux_dna_runner::api::SlurmAccounting =
        serde_json::from_value(serde_json::json!({
            "job_id": "3459974",
            "state": "completed",
            "raw_state": "COMPLETED",
            "exit_code": 0,
            "signal": 0,
            "elapsed_seconds": 60,
            "total_cpu_seconds": 58,
            "node_list": "n001"
        }))?;

    assert!(accounting.array_tasks.is_empty());
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
