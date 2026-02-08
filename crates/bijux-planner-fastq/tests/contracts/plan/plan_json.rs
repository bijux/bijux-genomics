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
    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_trim()?;
    assert_snapshot("stage__fastq__fastq.trim", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_filter()?;
    assert_snapshot("stage__fastq__fastq.filter", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_merge()?;
    assert_snapshot("stage__fastq__fastq.merge", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_validate_pre()?;
    assert_snapshot("stage__fastq__fastq.validate_pre", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_screen()?;
    assert_snapshot("stage__fastq__fastq.screen", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_umi()?;
    assert_snapshot("stage__fastq__fastq.umi", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_correct()?;
    assert_snapshot("stage__fastq__fastq.correct", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_preprocess()?;
    assert_snapshot("stage__fastq__fastq.preprocess", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_qc_post()?;
    assert_snapshot("stage__fastq__fastq.qc_post", &plan)?;

    let plan = bijux_planner_fastq::stage_api::stage_plan_fastq_stats_neutral()?;
    assert_snapshot("stage__fastq__fastq.stats_neutral", &plan)?;
    Ok(())
}
