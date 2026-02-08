#[test]
fn stage_contracts_have_required_policies() {
    for stage in bijux_domain_bam::BamStage::all() {
        let contract = bijux_domain_bam::contract_for_stage(stage.as_str())
            .unwrap_or_else(|| panic!("contract missing for {}", stage.as_str()));
        assert!(
            !contract.sorting.trim().is_empty(),
            "{} missing sorting",
            stage.as_str()
        );
        assert!(
            !contract.indexing.trim().is_empty(),
            "{} missing indexing",
            stage.as_str()
        );
        assert!(
            !contract.read_group_policy.trim().is_empty(),
            "{} missing read_group_policy",
            stage.as_str()
        );
        assert!(
            !contract.mapping_quality_policy.trim().is_empty(),
            "{} missing mapq policy",
            stage.as_str()
        );
        if !contract.deterministic {
            assert!(
                contract.nondeterminism_reason.is_some(),
                "{} nondeterministic without reason",
                stage.as_str()
            );
        }
    }
}
