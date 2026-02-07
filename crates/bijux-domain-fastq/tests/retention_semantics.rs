#[test]
fn retention_has_definition_and_units() {
    for stage in bijux_domain_fastq::stages::FastqStage::all() {
        let contract = bijux_domain_fastq::contract_for_stage(stage.as_str())
            .expect("contract");
        assert!(
            !contract.retention_definition.trim().is_empty(),
            "{} missing retention_definition",
            stage.as_str()
        );
        assert!(
            !contract.retention_units.trim().is_empty(),
            "{} missing retention_units",
            stage.as_str()
        );
    }
}
