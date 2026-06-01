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
fn local_endogenous_content_smoke_plans_use_governed_bam_and_host_scope() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM endogenous-content case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-endogenous-partial-mapping")
        .unwrap_or_else(|| panic!("governed BAM endogenous-content case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.endogenous_content");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 2);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/endogenous_content_partial_mapping.sam")
    );
    assert_eq!(case.host_reference_scope, "human_host");
    assert_eq!(case.expected_total_reads, 5);
    assert_eq!(case.expected_mapped_reads, 3);
    assert!((case.expected_endogenous_fraction - 0.6).abs() <= 1e-9);
    assert_eq!(case.expected_method, "mapped_fraction_from_flagstat");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.endogenous_content/core-v1-endogenous-partial-mapping/samtools")
    );
    assert_eq!(
        case.plan.params["bam"],
        serde_json::json!("assets/toy/core-v1/bam/endogenous_content_partial_mapping.sam")
    );
    assert_eq!(case.plan.params["host_reference_scope"], serde_json::json!("human_host"));

    let output_names = case
        .plan
        .io
        .outputs
        .iter()
        .map(|artifact| artifact.name.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(output_names, vec!["endogenous_report", "summary", "stage_metrics"]);

    let report_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "endogenous_report")
        .unwrap_or_else(|| panic!("endogenous-content report output missing from BAM plan"));
    assert_eq!(
        report_output.path,
        PathBuf::from(
            "target/local-smoke/bam.endogenous_content/core-v1-endogenous-partial-mapping/samtools/endogenous.content.json"
        )
    );

    Ok(())
}

#[test]
fn local_endogenous_content_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalEndogenousContentSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_endogenous_content_smoke_plans;
}
