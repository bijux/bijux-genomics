#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[cfg(feature = "bam_downstream")]
fn run_cargo_cli(args: &[&str], features: Option<&str>) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let mut command = Command::new("cargo");
    command
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["run", "-q", "-p", "bijux-dna"]);
    if let Some(features) = features {
        command.args(["--features", features]);
    }
    command.args(["--"]).args(args);
    command.output().expect("run cargo cli")
}

#[cfg(feature = "bam_downstream")]
fn run_cargo_cli_json(args: &[&str], features: Option<&str>) -> serde_json::Value {
    let output = run_cargo_cli(args, features);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_local_validate_slurm_dependencies_refuses_duplicated_dependency_locations() {
    let repo_root = support::repo_root().expect("repo root");
    let runs_root = repo_root.join("runs/bench");
    fs::create_dir_all(&runs_root).expect("create runs root");
    let tempdir = tempfile::Builder::new()
        .prefix("slurm-dependency-check-")
        .tempdir_in(&runs_root)
        .expect("tempdir");
    let root = tempdir.path().join("slurm-dry-run");
    let scripts_dir = root.join("fastq");
    fs::create_dir_all(&scripts_dir).expect("create scripts dir");

    let script_path = scripts_dir.join("fastq.validate_reads.sbatch");
    fs::write(
        &script_path,
        "#!/usr/bin/env bash\n#SBATCH --job-name=fastq-validate_reads\n#SBATCH --dependency=afterok:fastq-index_reference\ncargo run -q -p bijux-dna -- bench local materialize-stage --stage-id fastq.validate_reads\n",
    )
    .expect("write script");

    let manifest_path = tempdir.path().join("submit-manifest.json");
    fs::write(
        &manifest_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.bench.local_slurm_submit_manifest.v1",
            "root_path": root.to_string_lossy(),
            "manifest_path": manifest_path.to_string_lossy(),
            "run_id": "local-benchmark-dry-run",
            "job_count": 1,
            "dependency_count": 1,
            "jobs": [{
                "job_name": "fastq-validate_reads",
                "domain": "fastq",
                "stage_id": "fastq.validate_reads",
                "pipeline_id": serde_json::Value::Null,
                "corpus_id": "corpus-01-mini",
                "sample_id": serde_json::Value::Null,
                "sample_ids": ["toy-pe", "toy-se"],
                "tool_id": "fastqvalidator",
                "readiness_kind": "smoke",
                "resources": {
                    "cpus_per_task": 4,
                    "memory_mb": 8192,
                    "time_limit": "04:00:00"
                },
                "script_path": script_path.to_string_lossy(),
                "logs": {
                    "stdout_path": "stdout.log",
                    "stderr_path": "stderr.log"
                },
                "result_root": "result-root",
                "outputs": [],
                "dependencies": ["fastq-index_reference"],
                "compatibility_kind": "fixture",
                "compatibility_note": "fixture-backed"
            }]
        }))
        .expect("serialize manifest"),
    )
    .expect("write manifest");

    let report_path = tempdir.path().join("dependency-check.json");
    let output = run_cli(&[
        "bench",
        "local",
        "validate-slurm-dependencies",
        "--root",
        root.to_str().expect("root str"),
        "--manifest",
        manifest_path.to_str().expect("manifest str"),
        "--output",
        report_path.to_str().expect("report str"),
        "--json",
    ]);
    assert!(
        !output.status.success(),
        "command should fail\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("slurm dependency validation failed"),
        "stderr must explain the dependency validation failure, got:\n{stderr}"
    );

    let payload: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_dependency_check.v1")
    );
    assert_eq!(payload.get("findings_count").and_then(serde_json::Value::as_u64), Some(2));
    let job = payload
        .get("jobs")
        .and_then(serde_json::Value::as_array)
        .and_then(|jobs| jobs.first())
        .expect("job entry");
    assert_eq!(job.get("dependency_source").and_then(serde_json::Value::as_str), Some("mixed"));
    let findings = job.get("findings").and_then(serde_json::Value::as_array).expect("findings");
    assert!(
        findings.iter().any(|finding| {
            finding.as_str().is_some_and(|finding| {
                finding.contains("split across submit manifest and script header")
            })
        }),
        "job findings must flag mixed dependency sources"
    );
    assert!(
        findings.iter().any(|finding| {
            finding.as_str().is_some_and(|finding| finding.contains("fastq-index_reference"))
        }),
        "job findings must name the duplicated dependency"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_slurm_dependencies_reports_governed_submit_manifest_source() {
    let _manifest = run_cargo_cli_json(
        &["bench", "local", "render-slurm-submit-manifest", "--json"],
        Some("bam_downstream"),
    );
    let payload = run_cargo_cli_json(
        &["bench", "local", "validate-slurm-dependencies", "--json"],
        Some("bam_downstream"),
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_dependency_check.v1")
    );
    assert_eq!(
        payload.get("root_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run")
    );
    assert_eq!(
        payload.get("manifest_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run/submit-manifest.json")
    );
    assert_eq!(
        payload.get("report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run/dependency-check.json")
    );
    assert_eq!(payload.get("job_count").and_then(serde_json::Value::as_u64), Some(51));
    assert_eq!(
        payload.get("manifest_dependency_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(
        payload.get("script_header_dependency_count").and_then(serde_json::Value::as_u64),
        Some(0)
    );
    assert_eq!(payload.get("findings_count").and_then(serde_json::Value::as_u64), Some(0));
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(true));

    let jobs = payload.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    assert_eq!(jobs.len(), 51);
    assert!(jobs.iter().all(|job| {
        job.get("dependency_source").and_then(serde_json::Value::as_str) == Some("none")
            && job
                .get("manifest_dependencies")
                .and_then(serde_json::Value::as_array)
                .is_some_and(std::vec::Vec::is_empty)
            && job
                .get("script_header_dependencies")
                .and_then(serde_json::Value::as_array)
                .is_some_and(std::vec::Vec::is_empty)
            && job.get("script_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                path.starts_with("runs/bench/slurm-dry-run/") && path.ends_with(".sbatch")
            })
    }));
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_validate_slurm_dependencies_writes_governed_report_path() {
    let _manifest = run_cargo_cli_json(
        &["bench", "local", "render-slurm-submit-manifest", "--json"],
        Some("bam_downstream"),
    );
    let _payload = run_cargo_cli_json(
        &["bench", "local", "validate-slurm-dependencies", "--json"],
        Some("bam_downstream"),
    );

    let repo_root = support::repo_root().expect("repo root");
    let report_path = repo_root.join("runs/bench/slurm-dry-run/dependency-check.json");
    assert!(report_path.is_file(), "dependency report must exist");

    let report =
        serde_json::from_slice::<serde_json::Value>(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("report_path").and_then(serde_json::Value::as_str),
        Some("runs/bench/slurm-dry-run/dependency-check.json")
    );
}
