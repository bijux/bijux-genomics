use anyhow::Result;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| panic!("workspace root"))
        .to_path_buf()
}

#[test]
fn local_complexity_smoke_plans_use_governed_sparse_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_complexity_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM complexity case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-complexity-insufficient")
        .unwrap_or_else(|| panic!("governed BAM complexity case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.complexity");
    assert_eq!(case.plan.tool_id.as_str(), "preseq");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/complexity_sparse_reads.sam"));
    assert_eq!(case.min_reads, 3);
    assert_eq!(case.projection_points, vec![6, 12]);
    assert_eq!(case.expected_observed_total_reads, 3);
    assert_eq!(case.expected_observed_unique_reads, 2);
    assert_eq!(case.expected_estimated_unique_reads, None);
    assert_eq!(
        case.expected_insufficient_data_reason.as_deref(),
        Some("insufficient_observed_unique_reads_for_complexity_extrapolation")
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.complexity/core-v1-complexity-insufficient/preseq")
    );
    assert_eq!(case.plan.params["min_reads"], serde_json::json!(3));
    assert_eq!(case.plan.params["projection_points"], serde_json::json!([6, 12]));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["complexity_report", "complexity_curve", "summary", "stage_metrics"]
    );

    let complexity_curve_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "complexity_curve")
        .unwrap_or_else(|| panic!("complexity_curve output missing from BAM complexity plan"));
    assert_eq!(
        complexity_curve_output.path,
        PathBuf::from(
            "target/local-smoke/bam.complexity/core-v1-complexity-insufficient/preseq/complexity_curve.tsv"
        )
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM complexity plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.complexity/core-v1-complexity-insufficient/preseq/complexity.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_complexity_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalComplexitySmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_complexity_smoke_plans;
}

fn write_local_complexity_config(root: &Path, body: &str) -> Result<()> {
    let config_dir = root.join("configs/bench/local");
    fs::create_dir_all(&config_dir)?;
    fs::write(config_dir.join("bam-complexity.toml"), body)?;
    Ok(())
}

#[test]
fn local_complexity_smoke_plans_reject_empty_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_complexity_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_complexity.v1"
tool_id = "preseq"

[[cases]]
sample_id = " "
bam = "assets/toy/core-v1/bam/complexity_sparse_reads.sam"
min_reads = 3
projection_points = [6, 12]
expected_observed_total_reads = 3
expected_observed_unique_reads = 2
expected_insufficient_data_reason = "insufficient_observed_unique_reads_for_complexity_extrapolation"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_complexity_smoke_plans(temp.path())
        .expect_err("empty sample_id must be rejected before plan construction");
    assert_eq!(error.to_string(), "local-smoke bam.complexity sample_id must not be empty");
    Ok(())
}

#[test]
fn local_complexity_smoke_plans_reject_duplicate_sample_ids() -> Result<()> {
    let temp = tempfile::tempdir()?;
    write_local_complexity_config(
        temp.path(),
        r#"
schema_version = "bijux.bench.bam.local_complexity.v1"
tool_id = "preseq"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/complexity_sparse_reads.sam"
min_reads = 3
projection_points = [6, 12]
expected_observed_total_reads = 3
expected_observed_unique_reads = 2
expected_insufficient_data_reason = "insufficient_observed_unique_reads_for_complexity_extrapolation"

[[cases]]
sample_id = "duplicate-case"
bam = "assets/toy/core-v1/bam/complexity_sparse_reads.sam"
min_reads = 3
projection_points = [6, 12]
expected_observed_total_reads = 3
expected_observed_unique_reads = 2
expected_insufficient_data_reason = "insufficient_observed_unique_reads_for_complexity_extrapolation"
"#,
    )?;

    let error = bijux_dna_planner_bam::stage_api::local_complexity_smoke_plans(temp.path())
        .expect_err("duplicate sample_id must be rejected before plan construction");
    assert_eq!(
        error.to_string(),
        "duplicate local-smoke bam.complexity sample_id `duplicate-case`"
    );
    Ok(())
}
