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
fn local_mapping_summary_smoke_plans_use_governed_partial_mapping_fixture() -> Result<()> {
    let repo_root = repo_root();
    let plans = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans(&repo_root)?;
    assert_eq!(
        plans.len(),
        1,
        "governed local-smoke config must keep exactly one BAM mapping summary case"
    );

    let case = plans
        .iter()
        .find(|case| case.sample_id == "core-v1-partial-mapping")
        .unwrap_or_else(|| panic!("governed BAM mapping summary case missing"));
    assert_eq!(case.plan.stage_id.as_str(), "bam.mapping_summary");
    assert_eq!(case.plan.tool_id.as_str(), "samtools");
    assert_eq!(case.plan.resources.threads, 4);
    assert_eq!(
        case.bam,
        PathBuf::from("assets/toy/core-v1/bam/mapping_summary_partial_mapping.sam")
    );
    assert_eq!(case.expected_total_reads, 3);
    assert_eq!(case.expected_mapped_reads, 2);
    assert_eq!(case.expected_mapping_fraction, 2.0 / 3.0);
    assert_eq!(case.expected_reference_name, "chr1");
    assert_eq!(
        case.plan.out_dir,
        PathBuf::from("target/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools")
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
        vec!["flagstat", "idxstats", "stats", "summary", "stage_metrics"]
    );

    let summary_output = case
        .plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "summary")
        .unwrap_or_else(|| panic!("summary output missing from BAM mapping summary plan"));
    assert_eq!(
        summary_output.path,
        PathBuf::from(
            "target/local-smoke/bam.mapping_summary/core-v1-partial-mapping/samtools/mapping.summary.json"
        )
    );

    Ok(())
}

#[test]
fn local_mapping_summary_smoke_stage_api_surface_stays_callable() {
    let _: fn(
        &Path,
    ) -> anyhow::Result<Vec<bijux_dna_planner_bam::stage_api::LocalMappingSummarySmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans;
}
