use anyhow::Result;
use bijux_dna_core::prelude::{StageId, ToolId};
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
    assert_eq!(output_names, vec!["flagstat", "idxstats", "stats", "summary", "stage_metrics"]);

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
    ) -> anyhow::Result<
        Vec<bijux_dna_planner_bam::stage_api::LocalMappingSummarySmokeCasePlan>,
    > = bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans;
}

#[test]
fn mapping_summary_plan_accepts_picard_governed_planning_contract() -> Result<()> {
    let repo_root = repo_root();
    let stage_id = StageId::new("bam.mapping_summary".to_string());
    let tool_id = ToolId::new("picard");
    let tool_spec = bijux_dna_planner_bam::stage_api::load_bam_domain_tool_planning_spec(
        &repo_root,
        &stage_id,
        &tool_id,
    )?;
    let bam = PathBuf::from("assets/toy/core-v1/bam/mapping_summary_partial_mapping.sam");
    let out_dir =
        PathBuf::from("target/local-smoke/bam.mapping_summary/core-v1-partial-mapping/picard");
    let plan =
        bijux_dna_planner_bam::tool_adapters::stages_pre::mapping_summary::plan(&tool_spec, &bam, &out_dir)?;

    assert_eq!(plan.stage_id.as_str(), "bam.mapping_summary");
    assert_eq!(plan.tool_id.as_str(), "picard");
    assert_eq!(plan.out_dir, out_dir);

    let stats_output = plan
        .io
        .outputs
        .iter()
        .find(|artifact| artifact.name.as_str() == "stats")
        .unwrap_or_else(|| panic!("stats output missing from picard bam.mapping_summary plan"));
    assert_eq!(
        stats_output.path,
        PathBuf::from(
            "target/local-smoke/bam.mapping_summary/core-v1-partial-mapping/picard/alignment_summary.metrics.txt"
        )
    );

    let command = plan.command.template.last().unwrap_or_else(|| {
        panic!("picard bam.mapping_summary command template must contain a shell body")
    });
    assert!(
        command.contains("CollectAlignmentSummaryMetrics")
            && command.contains("BamIndexStats")
            && command.contains("mapping.summary.json")
            && command.contains("alignment_summary.metrics.txt"),
        "picard bam.mapping_summary command must keep the governed alignment-summary and idxstats contract"
    );

    Ok(())
}
