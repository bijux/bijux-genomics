use anyhow::Result;

#[path = "../support/mod.rs"]
mod support;

#[test]
fn fastq_stage_contracts_use_shared_harness() -> Result<()> {
    let stages = bijux_dna_domain_fastq::STAGES;
    let id_catalog: Vec<&str> = stages
        .iter()
        .map(bijux_dna_core::ids::StageId::as_str)
        .collect();
    support::assert_stage_contracts("fastq", id_catalog.iter().copied(), |stage_id| {
        bijux_dna_domain_fastq::contract_for_stage(stage_id).is_some()
    })
}
