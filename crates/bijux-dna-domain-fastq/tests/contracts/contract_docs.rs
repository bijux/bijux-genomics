use bijux_dna_domain_fastq::{contract_for_stage, STAGES};

#[test]
fn fastq_contracts_document_retention_and_preservation() {
    for stage_id in STAGES {
        let stage_str = stage_id.as_str();
        let Some(contract) = contract_for_stage(stage_str) else {
            continue;
        };
        assert!(
            !contract.retention_definition.is_empty(),
            "{stage_str} missing retention definition"
        );
        assert!(
            !contract.retention_units.is_empty(),
            "{stage_str} missing retention units"
        );
        assert!(
            !contract.preserves.is_empty(),
            "{stage_str} missing preservation list"
        );
        if contract.may_drop_reads {
            assert!(
                !contract.may_drop.is_empty(),
                "{stage_str} may drop reads but has empty drop list"
            );
        }
    }
}
