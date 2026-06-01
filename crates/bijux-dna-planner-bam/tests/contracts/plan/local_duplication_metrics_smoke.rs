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
fn local_duplication_metrics_smoke_plans_use_governed_duplicate_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans =
        bijux_dna_planner_bam::stage_api::local_duplication_metrics_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM duplication metrics case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-duplicate-observation")
        .unwrap_or_else(|| panic!("governed BAM duplication metrics case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.duplication_metrics");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/duplication_metrics_duplicate_cluster.sam")
    );
    assert_eq!(case.expected_examined_reads, 3);
    assert_eq!(case.expected_duplicate_reads, 1);
    assert_eq!(case.expected_duplicate_fraction, 1.0 / 3.0);
    assert_eq!(case.expected_estimated_library_size, None);
    assert_eq!(
        case.expected_insufficient_library_size_reason.as_deref(),
        Some("tiny_smoke_duplicate_observation_is_insufficient_for_library_size_estimate")
    );
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from(
            "target/local-smoke/bam.duplication_metrics/core-v1-duplicate-observation/samtools"
        )
    );
    assert_eq!(case.plan.params["optical_duplicates"], serde_json::json!("mark_only"));
    assert_eq!(case.plan.params["umi_policy"], serde_json::json!("ignore"));
    assert_eq!(case.plan.params["duplicate_action"], serde_json::json!("mark"));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        output_names,
        vec!["duplication_report", "duplication_histogram", "summary", "stage_metrics",]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM duplication metrics plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.duplication_metrics/core-v1-duplicate-observation/samtools/duplication.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_duplication_metrics_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalDuplicationMetricsSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_duplication_metrics_smoke_plans;
}
