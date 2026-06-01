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
fn local_filter_smoke_plans_use_governed_mixed_constraint_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_filter_smoke_plans(&repo_root)?;
    assert_eq!(plans.len(), 1, "governed local-smoke config must keep exactly one BAM filter case");

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-general-filter")
        .unwrap_or_else(|| panic!("governed BAM filter case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.filter");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/filter_mixed_constraints.sam")
    );
    assert_eq!(case.expected_input_reads, 5);
    assert_eq!(case.expected_kept_reads, 1);
    assert_eq!(case.expected_removed_reads, 4);
    assert_eq!(
        case.expected_active_filters,
        vec![
            "mapq_threshold".to_string(),
            "exclude_flags".to_string(),
            "min_length".to_string(),
            "remove_duplicates".to_string(),
        ]
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.filter/core-v1-general-filter/samtools")
    );
    assert_eq!(case.plan.params["mapq_threshold"], serde_json::json!(20));
    assert_eq!(case.plan.params["exclude_flags"], serde_json::json!([4]));
    assert_eq!(case.plan.params["min_length"], serde_json::json!(8));
    assert_eq!(case.plan.params["remove_duplicates"], serde_json::json!(true));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec![
            "filtered_bam",
            "filtered_bai",
            "flagstat_before",
            "flagstat_after",
            "idxstats_before",
            "idxstats_after",
            "summary",
            "stage_metrics",
        ]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.filter/core-v1-general-filter/samtools/filter.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_filter_smoke_stage_api_surface_stays_callable() {
    let _: fn(&Path) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalFilterSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_filter_smoke_plans;
}
