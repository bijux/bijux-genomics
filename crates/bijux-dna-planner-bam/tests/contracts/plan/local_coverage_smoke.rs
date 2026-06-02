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
fn local_coverage_smoke_plans_use_governed_target_windows_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM coverage case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-target-windows")
        .unwrap_or_else(|| panic!("governed BAM coverage case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.coverage");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/coverage_target_windows.sam"));
    assert_eq!(case.regions, PathBuf::from("assets/toy/core-v1/bam/coverage_target_windows.bed"));
    assert_eq!(case.depth_thresholds, vec![1, 5]);
    assert_eq!(case.expected_coverage_regime, "low_pass");
    assert_eq!(case.expected_rows.len(), 2);
    assert_eq!(case.expected_rows[0].region_id, "chr1_window");
    assert_eq!(case.expected_rows[0].contig, "chr1");
    assert_eq!(case.expected_rows[0].start, 1);
    assert_eq!(case.expected_rows[0].end, 6);
    assert_eq!(case.expected_rows[0].length, 6);
    assert!((case.expected_rows[0].mean_depth - (4.0 / 3.0)).abs() <= 1e-9);
    assert!((case.expected_rows[0].breadth_1x - 1.0).abs() <= 1e-9);
    assert_eq!(case.expected_rows[0].covered_bases, 6);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.coverage/core-v1-target-windows/samtools")
    );
    assert_eq!(case.plan.params["depth_thresholds"], serde_json::json!([1, 5]));
    assert_eq!(
        case.plan.params["regions"],
        serde_json::json!("assets/toy/core-v1/bam/coverage_target_windows.bed")
    );

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["coverage_summary", "coverage_depth", "stage_metrics"]);

    let depth_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "coverage_depth")
        .unwrap_or_else(|| panic!("coverage depth output missing from BAM coverage plan"));
    assert_eq!(
        depth_output.path,
        PathBuf::from(
            "target/local-smoke/bam.coverage/core-v1-target-windows/samtools/coverage.depth.txt"
        )
    );

    Ok(())
}

#[test]
fn local_coverage_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalCoverageSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_coverage_smoke_plans;
}
