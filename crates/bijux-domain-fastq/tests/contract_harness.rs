use anyhow::Result;

mod support;

#[test]
fn fastq_stage_contracts_use_shared_harness() -> Result<()> {
    let stage_ids: Vec<&str> = bijux_domain_fastq::STAGES
        .iter()
        .map(|stage| stage.stage_id)
        .collect();
    support::assert_stage_contracts("fastq", stage_ids.iter().copied(), |stage_id| {
        bijux_domain_fastq::contract_for_stage(stage_id).is_some()
    })
}
