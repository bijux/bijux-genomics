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
    let implemented = sort_stages(bijux_dna_stages_fastq::implemented_stages());
    assert_eq!(
        domain, implemented,
        "stages-fastq registry must match domain list"
    );
}
