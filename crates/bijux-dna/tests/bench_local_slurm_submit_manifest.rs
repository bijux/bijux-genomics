#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

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
fn run_cargo_cli_json(args: &[&str], features: &str) -> serde_json::Value {
    let output = run_cargo_cli(args, Some(features));
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[cfg(not(feature = "bam_downstream"))]
#[test]
fn bench_local_render_slurm_submit_manifest_requires_bam_downstream_feature() {
    let output = run_cargo_cli(&["bench", "local", "render-slurm-submit-manifest", "--json"], None);
    assert!(
        !output.status.success(),
        "command should fail without bam_downstream\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("requires the `bam_downstream` feature"),
        "stderr must explain the bam_downstream requirement, got:\n{stderr}"
    );
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_slurm_submit_manifest_reports_governed_51_job_slice() {
    let payload = run_cargo_cli_json(
        &["bench", "local", "render-slurm-submit-manifest", "--json"],
        "bam_downstream",
    );

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_slurm_submit_manifest.v1")
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
        payload.get("run_id").and_then(serde_json::Value::as_str),
        Some("local-benchmark-dry-run")
    );
    assert_eq!(payload.get("job_count").and_then(serde_json::Value::as_u64), Some(51));

    let jobs = payload.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    assert_eq!(jobs.len(), 51);
    assert!(jobs.iter().all(|job| {
        job.get("job_name").and_then(serde_json::Value::as_str).is_some()
            && job.get("domain").and_then(serde_json::Value::as_str).is_some()
            && job.get("tool_id").and_then(serde_json::Value::as_str).is_some()
            && job.get("script_path").and_then(serde_json::Value::as_str).is_some_and(|path| {
                path.starts_with("runs/bench/slurm-dry-run/") && path.ends_with(".sbatch")
            })
            && job
                .get("logs")
                .and_then(|logs| logs.get("stdout_path"))
                .and_then(serde_json::Value::as_str)
                .is_some_and(|path| {
                    path.starts_with("runs/bench/slurm-dry-run/runs/local-benchmark-dry-run/")
                })
            && job.get("outputs").and_then(serde_json::Value::as_array).is_some()
            && job.get("dependencies").and_then(serde_json::Value::as_array).is_some()
    }));
}

#[cfg(feature = "bam_downstream")]
#[test]
fn bench_local_render_slurm_submit_manifest_captures_governed_metadata_fields() {
    let payload = run_cargo_cli_json(
        &["bench", "local", "render-slurm-submit-manifest", "--json"],
        "bam_downstream",
    );
    let jobs = payload.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");

    let screen_taxonomy = jobs
        .iter()
        .find(|job| {
            job.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
        })
        .expect("screen taxonomy job");
    assert_eq!(
        screen_taxonomy.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-02-edna-mini")
    );
    assert_eq!(screen_taxonomy.get("sample_id").and_then(serde_json::Value::as_str), None);
    assert_eq!(
        screen_taxonomy
            .get("sample_ids")
            .and_then(serde_json::Value::as_array)
            .map(|rows| rows.len()),
        Some(2)
    );
    assert_eq!(
        screen_taxonomy.get("compatibility_kind").and_then(serde_json::Value::as_str),
        Some("fixture")
    );

    let bam_damage = jobs
        .iter()
        .find(|job| job.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage"))
        .expect("bam damage job");
    assert_eq!(
        bam_damage.get("corpus_id").and_then(serde_json::Value::as_str),
        Some("corpus-01-adna-damage-mini")
    );
    assert_eq!(
        bam_damage.get("sample_id").and_then(serde_json::Value::as_str),
        Some("adna_damage_non_udg")
    );
    assert_eq!(
        bam_damage.get("sample_ids").and_then(serde_json::Value::as_array).map(|rows| rows.len()),
        Some(1)
    );
    assert!(
        bam_damage
            .get("outputs")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|outputs| !outputs.is_empty()),
        "bam.damage must record declared outputs"
    );

    let index_reference = jobs
        .iter()
        .find(|job| {
            job.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.index_reference")
        })
        .expect("index reference job");
    assert_eq!(index_reference.get("corpus_id"), Some(&serde_json::Value::Null));
    assert_eq!(index_reference.get("sample_id"), Some(&serde_json::Value::Null));
    assert_eq!(
        index_reference.get("compatibility_kind").and_then(serde_json::Value::as_str),
        Some("planner_only")
    );
    assert!(
        index_reference
            .get("compatibility_note")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|note| note.contains("does not consume corpus reads")),
        "planner-only compatibility note must stay explicit"
    );
}
