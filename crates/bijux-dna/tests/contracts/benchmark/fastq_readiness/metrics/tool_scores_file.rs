#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::support;

fn render_path(label: &str) -> PathBuf {
    tempfile::Builder::new()
        .prefix(label)
        .tempdir()
        .expect("temporary score directory")
        .keep()
        .join("FASTQ_TOOL_SCORES.tsv")
}

fn run_cli(repo_root: &Path, home: &Path, args: &[&str]) -> std::process::Output {
    Command::new(support::cli_binary_path())
        .current_dir(repo_root)
        .env("HOME", home)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_readiness_fastq_tool_scores_writes_governed_tsv_columns() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let sandbox = support::RepoSandbox::new("fastq-tool-scores-tsv-").expect("repo sandbox");
    super::score_evidence::materialize_fastq_score_evidence(&sandbox);
    let output = super::score_evidence::run_cli(
        &sandbox,
        &["bench", "readiness", "render-fastq-tool-scores"],
    );

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "artifacts/benchmark-smoke/micro/fastq/FASTQ_TOOL_SCORES.tsv");

    let tsv = fs::read_to_string(sandbox.path().join(rendered_path.trim()))
        .expect("read FASTQ score TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tretained_reads\tdropped_reads\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        71,
        "TSV must retain one FASTQ score row per governed benchmark-ready binding"
    );
    assert!(
        rows.iter().any(|row| {
            row.starts_with("fastq.remove_duplicates\tfastuniq\t")
                && row.contains("\tblocked\t")
                && row.contains("\tblocked_external_access\t")
        }),
        "TSV must retain the supported external FastUniq binding as access-blocked"
    );
    let fastp = rows
        .iter()
        .find(|row| row.starts_with("fastq.filter_reads\tfastp\t"))
        .expect("TSV must retain the governed fastq.filter_reads fastp row");
    let fastp_columns = fastp.split('\t').collect::<Vec<_>>();
    assert_eq!(fastp_columns.get(2), Some(&"multi_tool_ranking"));
    assert_eq!(fastp_columns.get(3), Some(&"output_contract"));
    assert_eq!(fastp_columns.get(9), Some(&"scored"));
    assert_eq!(fastp_columns.get(10), Some(&"0.333333"));
    assert_eq!(fastp_columns.get(11), Some(&"retained_fraction"));
    assert_eq!(fastp_columns.get(12), Some(&"1.000000"));
    assert_eq!(fastp_columns.get(14), Some(&"1"));
    assert_eq!(fastp_columns.get(15), Some(&"2"));
    assert_eq!(fastp_columns.get(17), Some(&"not_available"));
    assert_eq!(fastp_columns.get(20), Some(&"not_available"));
    assert_eq!(fastp_columns.get(21), Some(&"none"));
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "fastq.validate_reads\tfastqvalidator\tmulti_tool_ranking\tscientific_comparable_metrics"
            ) && row.contains("\tscored\t1.000000\tvalidation_pass_fraction\t1.000000\t")
                && row.contains("\t6\t0\t")
        }),
        "TSV must retain the governed scored fastq.validate_reads fastqvalidator row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains("fastq.trim_reads\talientrimmer\tmulti_tool_ranking\toutput_contract")
                && row.contains("\tinsufficient_evidence\t\t\t\t\t\t\t")
                && row.contains("\tinsufficient_data\t")
        }),
        "TSV must retain the governed insufficient-evidence trim row"
    );
}

#[test]
fn bench_readiness_fastq_tool_scores_render_and_validate_custom_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let output_path = render_path("fastq-tool-scores-file-");
    let output_arg = output_path.to_string_lossy().into_owned();
    let reported_path = support::path_relative_to_repo(&repo_root, &output_path);

    let render_output = run_cli(
        &repo_root,
        home.path(),
        &["bench", "readiness", "render-fastq-tool-scores", "--output", &output_arg],
    );
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );
    assert_eq!(String::from_utf8(render_output.stdout).expect("stdout utf8").trim(), reported_path);

    let validate_output = run_cli(
        &repo_root,
        home.path(),
        &["bench", "readiness", "validate-fastq-tool-scores", "--input", &output_arg],
    );
    assert!(
        validate_output.status.success(),
        "validate command failed: {}\nstdout:\n{}\nstderr:\n{}",
        validate_output.status,
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );
    assert_eq!(
        String::from_utf8(validate_output.stdout).expect("stdout utf8").trim(),
        reported_path
    );
}

#[test]
fn bench_readiness_fastq_tool_scores_validation_rejects_stale_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");
    let output_path = render_path("fastq-tool-scores-stale-");
    let output_arg = output_path.to_string_lossy().into_owned();

    let render_output = run_cli(
        &repo_root,
        home.path(),
        &["bench", "readiness", "render-fastq-tool-scores", "--output", &output_arg],
    );
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let rendered = fs::read_to_string(&output_path).expect("read rendered TSV");
    let stale = rendered.replacen("insufficient_data", "none", 1);
    fs::write(&output_path, stale).expect("write stale TSV");

    let validate_output = run_cli(
        &repo_root,
        home.path(),
        &["bench", "readiness", "validate-fastq-tool-scores", "--input", &output_arg],
    );
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale TSV\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );

    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("FASTQ tool score TSV drifted"),
        "stale TSV failure must report score drift, got:\n{stderr}"
    );
}
