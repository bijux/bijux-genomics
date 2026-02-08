use anyhow::Result;
use bijux_testkit::snapshot_name;

fn assert_snapshot(name: &str, payload: &serde_json::Value) -> Result<()> {
    let name = snapshot_name("contracts", name);
    insta::assert_snapshot!(name, serde_json::to_string_pretty(&payload)?);
    Ok(())
}

#[test]
fn pipeline_plan_snapshots_are_stable() -> Result<()> {
    let payload = bijux_planner_bam::pipeline_plan_bam_to_bam__adna_shotgun__v1()?;
    assert_snapshot("pipeline__bam__bam-to-bam__adna_shotgun__v1", &payload)?;

    let payload = bijux_planner_bam::pipeline_plan_bam_to_bam__adna_capture__v1()?;
    assert_snapshot("pipeline__bam__bam-to-bam__adna_capture__v1", &payload)?;
    Ok(())
}
