#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_manifest_path(repo_root: &Path, label: &str) -> PathBuf {
    tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
        .expect("temporary HPC dry-run directory")
        .keep()
        .join("asset-staging-manifest.json")
}

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

#[test]
fn bench_local_render_hpc_asset_staging_manifest_reports_governed_inputs() {
    let repo_root = support::repo_root().expect("repo root");
    let manifest_path = render_manifest_path(&repo_root, "render-hpc-asset-staging-");
    let manifest_arg = manifest_path.to_string_lossy().into_owned();

    let output = run_cli(&[
        "bench",
        "local",
        "render-hpc-asset-staging-manifest",
        "--output",
        &manifest_arg,
    ]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let printed_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(
        printed_path.trim(),
        manifest_path
            .strip_prefix(&repo_root)
            .expect("manifest path relative to repo root")
            .to_string_lossy()
    );
    let payload = serde_json::from_slice::<serde_json::Value>(
        &fs::read(&manifest_path).expect("read manifest"),
    )
    .expect("parse manifest");

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_asset_staging_manifest.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some(
            manifest_path
                .strip_prefix(&repo_root)
                .expect("manifest path relative to repo root")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert_eq!(
        payload.get("staging_root").and_then(serde_json::Value::as_str),
        Some("runs/bench/hpc-dry-run/staged")
    );
    assert!(
        payload
            .get("selected_job_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "rendered manifest must select governed future HPC jobs"
    );
    assert!(
        payload
            .get("staged_input_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "rendered manifest must include staged inputs"
    );

    let jobs = payload.get("jobs").and_then(serde_json::Value::as_array).expect("jobs array");
    let benchmark_job = jobs
        .iter()
        .find(|job| {
            job.get("result_id").and_then(serde_json::Value::as_str)
                == Some("bam:corpus-01-mini:bam.align:sample-set:bowtie2")
        })
        .expect("governed BAM benchmark job");
    let staged_inputs = benchmark_job
        .get("staged_inputs")
        .and_then(serde_json::Value::as_array)
        .expect("staged inputs array");
    assert!(
        staged_inputs.iter().any(|input| {
            input.get("source_path").and_then(serde_json::Value::as_str)
                == Some("assets/reference/host/references/toy_host_reference")
                && input.get("staged_path").and_then(serde_json::Value::as_str)
                    == Some("runs/bench/hpc-dry-run/staged/assets/reference/host/references/toy_host_reference")
                && input.get("source_kind").and_then(serde_json::Value::as_str)
                    == Some("prefix_bundle")
                && input.get("checksum_sha256").and_then(serde_json::Value::as_str).is_some()
                && input.get("size_bytes").and_then(serde_json::Value::as_u64).is_some()
                && input.get("consumer_result_id").and_then(serde_json::Value::as_str)
                    == Some("bam:corpus-01-mini:bam.align:sample-set:bowtie2")
        }),
        "governed BAM job must keep staged source, staged path, checksum, size, and consumer result id"
    );
}
