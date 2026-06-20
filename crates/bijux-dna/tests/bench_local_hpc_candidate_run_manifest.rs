#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_path(repo_root: &Path, label: &str) -> (tempfile::TempDir, PathBuf) {
    let readiness_root = repo_root.join("benchmarks/readiness/hpc");
    fs::create_dir_all(&readiness_root).expect("create readiness hpc root");
    let temp_dir = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(&readiness_root)
        .expect("temporary readiness directory");
    let report_path = temp_dir.path().join("FIRST_HPC_CANDIDATE_RUN.json");
    (temp_dir, report_path)
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

fn row_by_group<'a>(
    rows: &'a [serde_json::Value],
    representative_group_id: &str,
) -> &'a serde_json::Value {
    rows.iter()
        .find(|row| {
            row.get("representative_group_id").and_then(serde_json::Value::as_str)
                == Some(representative_group_id)
        })
        .expect("governed representative group row")
}

#[test]
fn bench_local_render_hpc_candidate_run_manifest_selects_small_representatives() {
    let repo_root = support::repo_root().expect("repo root");
    let (_temp_dir, report_path) = render_path(&repo_root, "render-hpc-candidate-run-manifest-");
    let report_arg = report_path.to_string_lossy().into_owned();

    let output =
        run_cli(&["bench", "local", "render-hpc-candidate-run-manifest", "--output", &report_arg]);
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
        report_path
            .strip_prefix(&repo_root)
            .expect("report path relative to repo root")
            .to_string_lossy()
    );

    let report: serde_json::Value =
        serde_json::from_slice(&fs::read(&report_path).expect("read report"))
            .expect("parse report");
    assert_eq!(
        report.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.local_hpc_candidate_run_manifest.v1")
    );
    assert_eq!(
        report.get("selection_profile_id").and_then(serde_json::Value::as_str),
        Some("small_runtime_surface_representatives")
    );
    assert_eq!(report.get("selected_job_count").and_then(serde_json::Value::as_u64), Some(6));
    assert_eq!(
        report.get("representative_group_count").and_then(serde_json::Value::as_u64),
        Some(6)
    );
    assert_eq!(
        report
            .get("behavior")
            .and_then(|value| value.get("proven"))
            .and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        report
            .get("selected_domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert_eq!(
        report
            .get("selected_domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(3)
    );
    assert!(
        report
            .get("exclusion_reason_counts")
            .and_then(|value| value.get("unknown_execution_mode"))
            .and_then(serde_json::Value::as_u64)
            .is_some_and(|count| count > 0),
        "VCF rows should remain excluded until execution modes are governed"
    );

    let rows = report.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 6);
    assert!(rows
        .iter()
        .all(|row| { row.get("domain").and_then(serde_json::Value::as_str) != Some("vcf") }));
    assert_eq!(
        row_by_group(rows, "bam:containerized")
            .get("result_id")
            .and_then(serde_json::Value::as_str),
        Some("bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:samtools")
    );
    assert_eq!(
        row_by_group(rows, "bam:java").get("result_id").and_then(serde_json::Value::as_str),
        Some("bam:corpus-01-bam-mini:bam.mapping_summary:sample-set:picard")
    );
    assert_eq!(
        row_by_group(rows, "bam:python").get("result_id").and_then(serde_json::Value::as_str),
        Some("bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc")
    );
    assert_eq!(
        row_by_group(rows, "fastq:containerized")
            .get("result_id")
            .and_then(serde_json::Value::as_str),
        Some("fastq:corpus-01-mini:fastq.trim_terminal_damage:sample-set:seqkit")
    );
    assert_eq!(
        row_by_group(rows, "fastq:internal").get("result_id").and_then(serde_json::Value::as_str),
        Some("fastq:corpus-01-mini:fastq.detect_duplicates_premerge:sample-set:bijux_dna")
    );
    assert_eq!(
        row_by_group(rows, "fastq:python").get("result_id").and_then(serde_json::Value::as_str),
        Some("fastq:corpus-01-mini:fastq.trim_reads:sample-set:atropos")
    );
    assert!(
        row_by_group(rows, "fastq:internal")
            .get("stop_conditions")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|conditions| conditions.len() == 4),
        "selected rows must keep the governed stop conditions"
    );
}
