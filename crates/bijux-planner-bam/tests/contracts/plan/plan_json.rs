use anyhow::Result;
use bijux_stage_contract::StagePlanV1;
use bijux_testkit::snapshot_name;

fn assert_snapshot(name: &str, plan: &StagePlanV1) -> Result<()> {
    let payload = serde_json::to_value(plan)?;
    let name = snapshot_name("contracts", name);
    insta::assert_snapshot!(name, serde_json::to_string_pretty(&payload)?);
    Ok(())
}

#[test]
fn stage_plan_snapshots_are_stable() -> Result<()> {
    let plan = bijux_planner_bam::stage_api::stage_plan_bam_align()?;
    assert_snapshot("stage__bam__bam.align", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_validate()?;
    assert_snapshot("stage__bam__bam.validate", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_qc_pre()?;
    assert_snapshot("stage__bam__bam.qc_pre", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_filter()?;
    assert_snapshot("stage__bam__bam.filter", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_markdup()?;
    assert_snapshot("stage__bam__bam.markdup", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_complexity()?;
    assert_snapshot("stage__bam__bam.complexity", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_coverage()?;
    assert_snapshot("stage__bam__bam.coverage", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_damage()?;
    assert_snapshot("stage__bam__bam.damage", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_authenticity()?;
    assert_snapshot("stage__bam__bam.authenticity", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_contamination()?;
    assert_snapshot("stage__bam__bam.contamination", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_sex()?;
    assert_snapshot("stage__bam__bam.sex", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_bias_mitigation()?;
    assert_snapshot("stage__bam__bam.bias_mitigation", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_recalibration()?;
    assert_snapshot("stage__bam__bam.recalibration", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_haplogroups()?;
    assert_snapshot("stage__bam__bam.haplogroups", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_genotyping()?;
    assert_snapshot("stage__bam__bam.genotyping", &plan)?;

    let plan = bijux_planner_bam::stage_api::stage_plan_bam_kinship()?;
    assert_snapshot("stage__bam__bam.kinship", &plan)?;
    Ok(())
}
