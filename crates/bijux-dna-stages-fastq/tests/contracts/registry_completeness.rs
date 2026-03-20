use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::STAGES;

fn sort_stages(mut stages: Vec<StageId>) -> Vec<String> {
    stages.sort_by_key(|stage| stage.as_str().to_string());
    stages
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .collect()
}

#[test]
fn fastq_stage_registry_is_complete() {
    let domain = sort_stages(STAGES.to_vec());
    let contract_stage_ids = sort_stages(bijux_dna_stages_fastq::contract_stage_ids());
    assert_eq!(
        domain, contract_stage_ids,
        "stages-fastq contract registry must match the domain list"
    );
}

#[test]
fn implemented_stage_set_matches_closed_execution_support() {
    let closed = sort_stages(bijux_dna_domain_fastq::execution_closed_stage_ids());
    let implemented = sort_stages(bijux_dna_stages_fastq::implemented_stages());
    assert_eq!(
        closed, implemented,
        "implemented_stages must expose only closed execution coverage"
    );
}
