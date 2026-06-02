use anyhow::Result;
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
