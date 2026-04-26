use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn public_api_docs_match_curated_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs =
        std::fs::read_to_string(root.join("docs/PUBLIC_API.md")).expect("read docs/PUBLIC_API.md");

    assert!(
        docs.contains("## Public Modules\nNone."),
        "PUBLIC_API.md must document that implementation modules stay private"
    );
    assert_eq!(
        markdown_list_after_heading(&docs, "## Root Exports"),
        entries([
            "ChunkPlanSettings",
            "VcfPanelLock",
            "VcfPipelineInputs",
            "RegionChunkPlan",
            "PlannerExplainStage",
            "PlannerExplainV1",
            "PLANNER_VERSION",
            "explain_vcf_plan",
            "plan_vcf_stage_plans",
            "plan_vcf_pipeline",
            "plan_vcf_minimal",
        ]),
        "root export docs must match the curated VCF planner surface"
    );
}

#[test]
fn documented_root_exports_remain_compilable() {
    use anyhow::Result;
    use bijux_dna_core::contract::ExecutionGraph;
    use bijux_dna_planner_vcf::{
        ChunkPlanSettings, PlannerExplainStage, PlannerExplainV1, RegionChunkPlan, VcfPanelLock,
        VcfPipelineInputs, PLANNER_VERSION,
    };
    use bijux_dna_stage_contract::StagePlanV1;

    let _: &str = PLANNER_VERSION;
    let _: fn(&VcfPipelineInputs) -> Result<Vec<StagePlanV1>> =
        bijux_dna_planner_vcf::plan_vcf_stage_plans;
    let _: fn(&VcfPipelineInputs) -> Result<ExecutionGraph> =
        bijux_dna_planner_vcf::plan_vcf_pipeline;
    let _: fn(&VcfPipelineInputs) -> Result<ExecutionGraph> =
        bijux_dna_planner_vcf::plan_vcf_minimal;
    let _: fn(&VcfPipelineInputs, &[StagePlanV1]) -> PlannerExplainV1 =
        bijux_dna_planner_vcf::explain_vcf_plan;

    let _: Option<ChunkPlanSettings> = None;
    let _: Option<VcfPanelLock> = None;
    let _: Option<VcfPipelineInputs> = None;
    let _: Option<RegionChunkPlan> = None;
    let _: Option<PlannerExplainStage> = None;
    let _: Option<PlannerExplainV1> = None;
}

fn markdown_list_after_heading(markdown: &str, heading: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut in_section = false;

    for line in markdown.lines() {
        if line == heading {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(item) = line.strip_prefix("- `").and_then(|line| line.strip_suffix('`')) {
            values.insert(item.to_string());
        }
    }

    values
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
