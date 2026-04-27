#[test]
fn root_exports_remain_compilable() {
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
