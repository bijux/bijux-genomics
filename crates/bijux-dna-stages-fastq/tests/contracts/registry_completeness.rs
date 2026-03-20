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
fn implemented_stage_set_matches_observer_runtime_coverage() {
    let observer_covered = sort_stages(bijux_dna_stages_fastq::observer_stage_ids());
    let implemented = sort_stages(bijux_dna_stages_fastq::implemented_stages());
    assert_eq!(
        observer_covered, implemented,
        "implemented_stages must expose only observer-specialized runtime interpretation coverage"
    );
}

#[test]
fn runtime_interpretation_partitions_fastq_contract_coverage() {
    let specialized = sort_stages(bijux_dna_stages_fastq::runtime_interpretation_stage_ids(
        bijux_dna_stages_fastq::RuntimeInterpretationLevel::ObserverSpecialized,
    ));
    let generic = sort_stages(bijux_dna_stages_fastq::runtime_interpretation_stage_ids(
        bijux_dna_stages_fastq::RuntimeInterpretationLevel::GenericEnvelope,
    ));
    let all = sort_stages(bijux_dna_stages_fastq::contract_stage_ids());
    let mut combined = specialized.clone();
    combined.extend(generic.clone());
    combined.sort();
    combined.dedup();
    assert_eq!(
        combined, all,
        "runtime interpretation levels must cover the full FASTQ contract registry",
    );
}
