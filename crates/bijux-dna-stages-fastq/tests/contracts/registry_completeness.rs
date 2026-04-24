use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::STAGES;

fn sort_stages(mut stages: Vec<StageId>) -> Vec<String> {
    stages.sort_by_key(|stage| stage.as_str().to_string());
    stages.into_iter().map(|stage| stage.as_str().to_string()).collect()
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
fn implemented_stage_set_matches_closed_execution_coverage() {
    let closed_execution = sort_stages(bijux_dna_stages_fastq::closed_execution_stage_ids());
    let implemented = sort_stages(bijux_dna_stages_fastq::implemented_stages());
    assert_eq!(
        closed_execution, implemented,
        "implemented_stages must expose the full closed execution FASTQ surface"
    );
}

#[test]
fn observer_specialized_stage_set_stays_within_closed_execution() {
    let observer_specialized =
        sort_stages(bijux_dna_stages_fastq::observer_specialized_stage_ids());
    let observer_alias = sort_stages(bijux_dna_stages_fastq::observer_stage_ids());
    let closed_execution = sort_stages(bijux_dna_stages_fastq::closed_execution_stage_ids());
    assert_eq!(
        observer_specialized, observer_alias,
        "observer_stage_ids must remain an alias for observer_specialized_stage_ids"
    );
    assert!(
        observer_specialized.iter().all(|stage_id| closed_execution.contains(stage_id)),
        "observer-specialized coverage must stay within the full closed execution surface"
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
