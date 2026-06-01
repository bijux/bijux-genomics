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
fn local_mapq_filter_smoke_plans_use_governed_threshold_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM MAPQ filter case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-mapq-threshold")
        .unwrap_or_else(|| panic!("governed BAM MAPQ filter case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.mapq_filter");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(case.bam, PathBuf::from("assets/toy/core-v1/bam/mapq_threshold_ladder.sam"));
    assert_eq!(case.mapq_threshold, 30);
    assert_eq!(case.expected_input_reads, 4);
    assert_eq!(case.expected_kept_reads, 3);
    assert_eq!(case.expected_removed_reads, 1);
    assert_eq!(case.expected_mapped_reads_removed, 1);
    assert_eq!(case.expected_mapped_fraction_retained, 2.0 / 3.0);
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.mapq_filter/core-v1-mapq-threshold/samtools")
    );
    assert_eq!(case.plan.params["action"], serde_json::json!("mapq_filter"));
    assert_eq!(case.plan.params["mapq_threshold"], serde_json::json!(30));
    assert_eq!(case.plan.params["include_flags"], serde_json::json!([]));
    assert_eq!(case.plan.params["exclude_flags"], serde_json::json!([]));

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
        .unwrap_or_else(|| panic!("summary output missing from BAM MAPQ filter plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.mapq_filter/core-v1-mapq-threshold/samtools/mapq_filter.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_mapq_filter_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalMapqFilterSmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_mapq_filter_smoke_plans;
}
