use bijux_domain_bam::{contract_for_stage, BamStage};

#[test]
fn bam_contracts_document_policies() {
    for stage in BamStage::all() {
        let stage_id = stage.as_str();
        let contract = contract_for_stage(stage_id)
            .unwrap_or_else(|| panic!("contract missing for {stage_id}"));
        assert!(
            !contract.sorting.is_empty(),
            "{stage_id} missing sorting policy"
        );
        assert!(
            !contract.indexing.is_empty(),
            "{stage_id} missing indexing policy"
        );
        assert!(
            !contract.read_group_policy.is_empty(),
            "{stage_id} missing read group policy"
        );
        assert!(
            !contract.duplicate_policy.is_empty(),
            "{stage_id} missing duplicate policy"
        );
        assert!(
            !contract.mapping_quality_policy.is_empty(),
            "{stage_id} missing mapping quality policy"
        );
        if !contract.deterministic {
            assert!(
                contract.nondeterminism_reason.is_some(),
                "{stage_id} nondeterministic without reason"
            );
        }
    }
}
