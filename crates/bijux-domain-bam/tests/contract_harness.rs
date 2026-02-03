use anyhow::Result;

#[test]
fn bam_stage_contracts_use_shared_harness() -> Result<()> {
    let stage_ids: Vec<&str> = bijux_domain_bam::BamStage::all()
        .iter()
        .map(|stage| stage.as_str())
        .collect();
    bijux_testkit::assert_stage_contracts("bam", stage_ids.iter().copied(), |stage_id| {
        bijux_domain_bam::contract_for_stage(stage_id).is_some()
    })
}
