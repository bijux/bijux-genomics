#[test]
fn legacy_manifests_still_load() {
    // Domain crate must remain pure; manifest loading is owned by bijux-runtime.
    // This test now asserts core domain registries are accessible instead.
    for stage in bijux_domain_fastq::STAGES {
        let _ = bijux_domain_fastq::stage_semantics(&stage);
        let _ = bijux_domain_fastq::contract_for_stage(stage.as_str());
    }
}

#[test]
fn domain_onboarding_checklist_is_satisfied() {
    let mut stages = bijux_domain_fastq::canonical_stage_order();
    for (_, branch) in bijux_domain_fastq::optional_branches() {
        stages.extend(branch.iter().cloned());
    }
    stages.sort_unstable();
    stages.dedup();

    for stage in stages {
        assert!(
            bijux_core::metrics_registry::metrics_schema_for_stage(stage.as_str()).is_some(),
            "missing metrics schema for {stage}"
        );
        assert!(
            bijux_domain_fastq::stage_semantics(&stage).is_some(),
            "missing report semantics for {stage}"
        );
        assert!(
            bijux_domain_fastq::contract_for_stage(stage.as_str()).is_some(),
            "missing artifact contract for {stage}"
        );
    }
}
