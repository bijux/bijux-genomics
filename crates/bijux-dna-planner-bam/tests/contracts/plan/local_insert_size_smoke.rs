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
fn local_insert_size_smoke_plans_use_governed_paired_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM insert-size case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-paired-triplet")
        .unwrap_or_else(|| panic!("governed BAM insert-size case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.insert_size");
    assert_eq!(case.plan.tool_id.as_str(), "picard");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/insert_size_paired_triplet.sam")
    );
    assert_eq!(case.expected_read_pairs, 3);
    assert!((case.expected_median_insert_size - 20.0).abs() <= 1e-9);
    assert!((case.expected_mean_insert_size - 21.666666666666668).abs() <= 1e-9);
    assert_eq!(case.expected_min_insert_size, 15);
    assert_eq!(case.expected_max_insert_size, 30);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.insert_size/core-v1-paired-triplet/picard")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/insert_size_paired_triplet.sam")
    );

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["insert_size_report", "insert_size_histogram", "summary", "stage_metrics"]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("insert-size summary output missing from BAM insert-size plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.insert_size/core-v1-paired-triplet/picard/insert_size.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_insert_size_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    )
        -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalInsertSizeSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_insert_size_smoke_plans;
}
