#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_paths(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf, PathBuf) {
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(repo_root.join("runs/bench/hpc-dry-run"))
        .expect("temporary HPC dry-run directory");
    let script_path = temp_dir.path().join("stage-benchmark-array.sbatch");
    let manifest_path = temp_dir.path().join("stage-benchmark-array-manifest.json");
    (temp_dir, script_path, manifest_path)
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
fn bench_local_render_hpc_stage_benchmark_array_reports_governed_index_manifest() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, script_path, manifest_path) =
        render_paths(&repo_root, "render-hpc-stage-benchmark-array-");
    let script_arg = script_path.to_string_lossy().into_owned();

    let output =
        run_cli(&["bench", "local", "render-hpc-stage-benchmark-array", "--output", &script_arg]);
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
        script_path
            .strip_prefix(&repo_root)
            .expect("script path relative to repo root")
            .to_string_lossy()
    );

    let manifest: serde_json::Value =
        serde_json::from_slice(&fs::read(&manifest_path).expect("read manifest"))
            .expect("parse manifest");
    assert_eq!(
        manifest.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_stage_benchmark_array.v1")
    );
    assert_eq!(
        manifest.get("script_path").and_then(serde_json::Value::as_str),
        Some(
            script_path
                .strip_prefix(&repo_root)
                .expect("script path relative to repo root")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert_eq!(
        manifest.get("manifest_path").and_then(serde_json::Value::as_str),
        Some(
            manifest_path
                .strip_prefix(&repo_root)
                .expect("manifest path relative to repo root")
                .to_string_lossy()
                .as_ref()
        )
    );
    assert!(
        manifest
            .get("benchmark_job_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 100),
        "rendered array manifest must cover the governed benchmark-result scope"
    );
    assert!(
        manifest
            .get("array_spec")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.starts_with("0-")),
        "array script must declare a zero-based SLURM array range"
    );

    let rows = manifest.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let vcf_stats = rows
        .iter()
        .find(|row| {
            row.get("result_id").and_then(serde_json::Value::as_str)
                == Some("vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
        })
        .expect("governed VCF benchmark row");
    assert_eq!(
        vcf_stats.get("benchmark_result_command").and_then(serde_json::Value::as_str),
        Some("bijux-dna bench local execute-all-domain-benchmark-result --result-id vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools")
    );
    assert_eq!(
        vcf_stats.get("stdout_path").and_then(serde_json::Value::as_str),
        Some(
            "runs/bench/slurm-dry-run/all-domains/runs/all-domain-benchmark-dry-run/vcf/vcf.stats/bcftools/vcf_production_regression/vcf_cohort/stdout.log"
        )
    );
    assert!(
        vcf_stats.get("output_paths").and_then(serde_json::Value::as_array).is_some_and(
            |outputs| outputs.iter().any(|value| {
                value.as_str().is_some_and(|path| {
                    path.contains("/declared-outputs/")
                        && path.starts_with("runs/bench/slurm-dry-run/")
                })
            })
        ),
        "array rows must preserve exact declared output paths from the benchmark submit manifest"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("resolution_kind").and_then(serde_json::Value::as_str)
                == Some("apptainer_image")
        }),
        "array manifest must preserve container-backed execution resolution rows"
    );

    let script_body = fs::read_to_string(&script_path).expect("read script");
    assert!(
        script_body.contains("#SBATCH --array=0-"),
        "rendered script must declare an explicit SLURM array range"
    );
    assert!(
        script_body.contains("SLURM_ARRAY_TASK_ID"),
        "rendered script must dispatch on SLURM_ARRAY_TASK_ID"
    );
    assert!(
        script_body.contains(
            "bijux-dna bench local execute-all-domain-benchmark-result --result-id vcf:vcf_production_regression:vcf.stats:vcf_cohort:bcftools"
        ),
        "rendered script must keep the governed benchmark-result execution command"
    );
}
