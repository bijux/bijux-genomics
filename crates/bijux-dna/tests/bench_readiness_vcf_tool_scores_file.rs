#![allow(clippy::expect_used)]

use std::fs;
use std::path::{Path, PathBuf};

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn render_path(sandbox: &support::RepoSandbox, contract: &str) -> PathBuf {
    let output_dir = sandbox.path().join("artifacts/tests/vcf-tool-scores").join(contract);
    fs::create_dir_all(&output_dir).expect("create score output directory");
    output_dir.join("VCF_TOOL_SCORES.tsv")
}

fn run_cli(sandbox: &support::RepoSandbox, home: &Path, args: &[&str]) -> std::process::Output {
    sandbox
        .bijux_dna_command()
        .env("HOME", home)
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

#[test]
fn bench_readiness_vcf_tool_scores_writes_governed_tsv_columns() {
    let sandbox = support::RepoSandbox::new("vcf-tool-score-columns-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");
    sandbox.materialize_vcf_score_evidence(home.path()).expect("materialize VCF score evidence");

    let output = run_cli(&sandbox, home.path(), &["bench", "readiness", "render-vcf-tool-scores"]);

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "runs/bench/micro/vcf/VCF_TOOL_SCORES.tsv");

    let tsv =
        fs::read_to_string(sandbox.path().join(rendered_path.trim())).expect("read VCF score TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\tdecision_mode\tcorrectness_signal\tresult_ids\treport_row_ids\tcorpus_ids\treport_sections\trow_statuses\tscore_status\ttruth_correctness_score\ttruth_correctness_basis\tcontract_correctness_score\tcontract_correctness_basis\tgenotype_truth_metric_value\tgenotype_truth_metric_basis\tmissingness_metric_value\tmissingness_metric_basis\tphasing_imputation_metric_value\tphasing_imputation_metric_basis\tpopulation_metric_value\tpopulation_metric_basis\tscientific_metric_ids\tscientific_metric_summary\truntime_seconds\truntime_source\tobserved_memory_mb\tdeclared_memory_mb\tmemory_source\tfailure_class\tmicro_execution_status\tscore_weight_coverage\tscore_total\tevidence_paths\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(
        rows.len(),
        21,
        "TSV must retain one VCF score row per governed benchmark-ready binding"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains("vcf.qc\tplink2\tmulti_tool_ranking\tscientific_comparable_metrics")
                && row.contains("\tscored\t")
                && row.contains("\tone_minus_missingness_post\t")
                && row.contains("\trsq_mean\t")
                && row.contains("\tnone\t")
        }),
        "TSV must retain the governed scored qc row with missingness and imputation quality"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains("vcf.imputation_metrics\tbeagle\tsingle_tool_acceptance\tscientific_comparable_metrics")
                && row.contains("\tconcordance\t")
                && row.contains("\tdosage_r2\t")
                && row.contains("\tnone\t")
        }),
        "TSV must retain the governed imputation metrics quality row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains("vcf.population_structure\tplink2\tsingle_tool_acceptance\tscientific_comparable_metrics")
                && row.contains("\tpair_count\t")
                && row.contains("\tnone\t")
        }),
        "TSV must retain the governed population-structure score row"
    );
}

#[test]
fn bench_readiness_vcf_tool_scores_render_and_validate_custom_file() {
    let sandbox = support::RepoSandbox::new("vcf-tool-score-validation-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");
    sandbox.materialize_vcf_score_evidence(home.path()).expect("materialize VCF score evidence");
    let output_path = render_path(&sandbox, "render-validation");
    let output_arg = output_path.to_string_lossy().into_owned();
    let reported_path = support::path_relative_to_repo(sandbox.path(), &output_path);

    let render_output = run_cli(
        &sandbox,
        home.path(),
        &["bench", "readiness", "render-vcf-tool-scores", "--output", &output_arg],
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
        &sandbox,
        home.path(),
        &["bench", "readiness", "validate-vcf-tool-scores", "--input", &output_arg],
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
fn bench_readiness_vcf_tool_scores_validation_rejects_stale_file() {
    let sandbox =
        support::RepoSandbox::new("vcf-tool-score-stale-validation-").expect("repo sandbox");
    let home = tempfile::tempdir().expect("tempdir");
    sandbox.materialize_vcf_score_evidence(home.path()).expect("materialize VCF score evidence");
    let output_path = render_path(&sandbox, "stale-validation");
    let output_arg = output_path.to_string_lossy().into_owned();

    let render_output = run_cli(
        &sandbox,
        home.path(),
        &["bench", "readiness", "render-vcf-tool-scores", "--output", &output_arg],
    );
    assert!(
        render_output.status.success(),
        "render command failed: {}\nstdout:\n{}\nstderr:\n{}",
        render_output.status,
        String::from_utf8_lossy(&render_output.stdout),
        String::from_utf8_lossy(&render_output.stderr)
    );

    let rendered = fs::read_to_string(&output_path).expect("read rendered TSV");
    let stale = rendered.replacen("none", "insufficient_data", 1);
    fs::write(&output_path, stale).expect("write stale TSV");

    let validate_output = run_cli(
        &sandbox,
        home.path(),
        &["bench", "readiness", "validate-vcf-tool-scores", "--input", &output_arg],
    );
    assert!(
        !validate_output.status.success(),
        "validate command should reject stale TSV\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&validate_output.stdout),
        String::from_utf8_lossy(&validate_output.stderr)
    );

    let stderr = String::from_utf8_lossy(&validate_output.stderr);
    assert!(
        stderr.contains("VCF tool score TSV drifted"),
        "stale TSV failure must report score drift, got:\n{stderr}"
    );
}
