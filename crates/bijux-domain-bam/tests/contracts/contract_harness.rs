use anyhow::Result;

#[path = "../support/mod.rs"]
mod support;

#[test]
fn bam_stage_contracts_use_shared_harness() -> Result<()> {
    let stage_ids: Vec<&str> = bijux_domain_bam::BamStage::all()
        .iter()
        .map(|stage| stage.as_str())
        .collect();
    support::assert_stage_contracts("bam", stage_ids.iter().copied(), |stage_id| {
        bijux_domain_bam::contract_for_stage(stage_id).is_some()
    })
}
