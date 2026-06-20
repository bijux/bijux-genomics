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
    let script_path = temp_dir.path().join("pipeline-node-array.sbatch");
    let manifest_path = temp_dir.path().join("pipeline-node-array-manifest.json");
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
fn bench_local_render_hpc_pipeline_node_array_reports_governed_dependency_manifest() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, script_path, manifest_path) =
        render_paths(&repo_root, "render-hpc-pipeline-node-array-");
    let script_arg = script_path.to_string_lossy().into_owned();

    let output =
        run_cli(&["bench", "local", "render-hpc-pipeline-node-array", "--output", &script_arg]);
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
        Some("bijux.bench.local_hpc_pipeline_node_array.v1")
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
            .get("pipeline_job_count")
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 30),
        "rendered array manifest must cover the governed essential-pipeline scope"
    );

    let rows = manifest.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let vcf_qc = rows
        .iter()
        .find(|row| {
            row.get("pipeline_id").and_then(serde_json::Value::as_str)
                == Some("core-germline-fastq-bam-vcf")
                && row.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.qc")
        })
        .expect("governed VCF QC pipeline node");
    assert_eq!(
        vcf_qc.get("pipeline_node_command").and_then(serde_json::Value::as_str),
        Some(
            "bijux-dna bench local execute-essential-pipeline-node --pipeline-id core-germline-fastq-bam-vcf --node-id vcf.qc"
        )
    );
    assert_eq!(
        vcf_qc.get("dependency_job_ids").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec![
            "pipeline:core-germline-fastq-bam-vcf:vcf.filter",
            "pipeline:core-germline-fastq-bam-vcf:vcf.stats",
        ])
    );
    assert_eq!(
        vcf_qc.get("upstream_result_ids").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec!["filtered_vcf", "filtered_vcf_tbi", "stats_json"])
    );

    let dependencies =
        vcf_qc.get("dependencies").and_then(serde_json::Value::as_array).expect("dependency array");
    let vcf_filter = dependencies
        .iter()
        .find(|dependency| {
            dependency.get("node_id").and_then(serde_json::Value::as_str) == Some("vcf.filter")
        })
        .expect("vcf.filter dependency");
    assert_eq!(
        vcf_filter.get("upstream_result_ids").and_then(serde_json::Value::as_array).map(|values| {
            values.iter().filter_map(serde_json::Value::as_str).collect::<Vec<_>>()
        }),
        Some(vec!["filtered_vcf", "filtered_vcf_tbi"])
    );
    assert!(
        vcf_filter.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
            |outputs| outputs.iter().any(|output| {
                output.get("output_id").and_then(serde_json::Value::as_str) == Some("filtered_vcf")
                    && output.get("output_path").and_then(serde_json::Value::as_str).is_some_and(
                        |path| {
                            path.contains("/core-germline-fastq-bam-vcf/vcf.filter/")
                                && path.ends_with("/outputs/filtered_vcf.json")
                        },
                    )
            })
        ),
        "dependency manifest must map vcf.filter to the exact governed filtered VCF output path"
    );
    assert!(
        vcf_qc.get("expected_outputs").and_then(serde_json::Value::as_array).is_some_and(
            |outputs| outputs.iter().any(|output| {
                output.get("output_id").and_then(serde_json::Value::as_str) == Some("qc_report")
                    && output.get("output_path").and_then(serde_json::Value::as_str).is_some_and(
                        |path| {
                            path.contains("/core-germline-fastq-bam-vcf/vcf.qc/")
                                && path.ends_with("/outputs/qc_report.json")
                        },
                    )
            })
        ),
        "pipeline node rows must preserve exact expected output paths for the governed node"
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
            "bijux-dna bench local execute-essential-pipeline-node --pipeline-id core-germline-fastq-bam-vcf --node-id vcf.qc"
        ),
        "rendered script must keep the governed essential-pipeline execution command"
    );
    assert!(
        script_body.contains(
            "DEPENDENCY_JOB_IDS='pipeline:core-germline-fastq-bam-vcf:vcf.filter,pipeline:core-germline-fastq-bam-vcf:vcf.stats'"
        ),
        "rendered script must keep the dependency manifest linkage for governed nodes"
    );
}
