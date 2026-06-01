use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn public_api_docs_match_root_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs =
        std::fs::read_to_string(root.join("docs/PUBLIC_API.md")).expect("read docs/PUBLIC_API.md");

    assert_eq!(
        markdown_list_after_heading(&docs, "## Public Modules"),
        entries(["tool_adapters"]),
        "public module docs must match the root module surface"
    );
    assert_eq!(
        markdown_list_after_heading(&docs, "## Root Exports"),
        entries([
            "BamPlanner",
            "BamPipelineInputs",
            "BamPlanConfig",
            "StagePlanRequest",
            "bam_workflow_template_catalog",
            "plan_stage",
            "plan_bam_to_bam__default__v1",
            "plan_bam_to_bam__adna_shotgun__v1",
            "plan_bam_to_bam__adna_capture__v1",
            "plan_bam_workflow_template",
            "pipeline_id_catalog",
            "report_stage_step",
            "stage_api",
            "PLANNER_VERSION",
        ]),
        "root export docs must match the public planner surface"
    );
}

#[test]
fn documented_root_exports_remain_compilable() {
    use anyhow::Result;
    use bijux_dna_core::contract::{ArtifactRef, ExecutionGraph, ExecutionStep};
    use bijux_dna_domain_bam::BamWorkflowTemplateV1;
    use bijux_dna_planner_bam::{
        BamPipelineInputs, BamPlanConfig, StagePlanRequest, PLANNER_VERSION,
    };
    use bijux_dna_stage_contract::StagePlanV1;

    let _planner = bijux_dna_planner_bam::BamPlanner;
    let _: &str = PLANNER_VERSION;
    let _: fn(&BamPlanConfig) -> Result<ExecutionGraph> = bijux_dna_planner_bam::BamPlanner::plan;
    let _: for<'a> fn(StagePlanRequest<'a>) -> Result<StagePlanV1> =
        bijux_dna_planner_bam::plan_stage;
    let _: fn() -> Vec<BamWorkflowTemplateV1> =
        bijux_dna_planner_bam::bam_workflow_template_catalog;
    let _: fn(&BamPipelineInputs) -> Result<ExecutionGraph> =
        bijux_dna_planner_bam::plan_bam_to_bam__default__v1;
    let _: fn(&BamPipelineInputs) -> Result<ExecutionGraph> =
        bijux_dna_planner_bam::plan_bam_to_bam__adna_shotgun__v1;
    let _: fn(&BamPipelineInputs) -> Result<ExecutionGraph> =
        bijux_dna_planner_bam::plan_bam_to_bam__adna_capture__v1;
    let _: fn(&str, &BamPipelineInputs) -> Result<ExecutionGraph> =
        bijux_dna_planner_bam::plan_bam_workflow_template;
    let _: fn(&str) -> Vec<String> = bijux_dna_planner_bam::pipeline_id_catalog;
    let _: fn(&Path, Vec<ArtifactRef>, Vec<ArtifactRef>) -> ExecutionStep =
        bijux_dna_planner_bam::report_stage_step;
    let _: for<'a> fn(StagePlanRequest<'a>) -> Result<StagePlanV1> =
        bijux_dna_planner_bam::stage_api::plan_stage;
    let _: fn(&Path) -> Result<Vec<bijux_dna_planner_bam::stage_api::LocalValidateSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_validate_smoke_plans;
    let _: fn(&Path) -> Result<Vec<bijux_dna_planner_bam::stage_api::LocalQcPreSmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_qc_pre_smoke_plans;
    let _: fn(
        &Path,
    ) -> Result<Vec<bijux_dna_planner_bam::stage_api::LocalMappingSummarySmokeCasePlan>> =
        bijux_dna_planner_bam::stage_api::local_mapping_summary_smoke_plans;
    let _: &str = bijux_dna_planner_bam::tool_adapters::tools::catalog::TOOLS_NAMESPACE;
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
