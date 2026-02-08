#[test]
fn retention_has_definition_and_units() {
    for stage_id in bijux_domain_fastq::stages::ids::STAGES {
        let contract = bijux_domain_fastq::contract_for_stage(stage_id.as_str())
            .unwrap_or_else(|| panic!("contract missing for {}", stage_id.as_str()));
        assert!(
            !contract.retention_definition.trim().is_empty(),
            "{} missing retention_definition",
            stage_id.as_str()
        );
        assert!(
            !contract.retention_units.trim().is_empty(),
            "{} missing retention_units",
            stage_id.as_str()
        );
    }
}
