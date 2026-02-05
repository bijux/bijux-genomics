#[test]
fn legacy_manifests_still_load() {
    let domain = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain");
    let registry = bijux_runtime::manifests::load_manifests(&domain);
    assert!(registry.is_ok());
}

#[test]
fn domain_onboarding_checklist_is_satisfied() {
    let domain = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain");
    let registry = bijux_runtime::manifests::load_manifests(&domain);
    assert!(registry.is_ok(), "stage registry missing");

    let mut stages = bijux_domain_fastq::canonical_stage_order();
    for (_, branch) in bijux_domain_fastq::optional_branches() {
        stages.extend(branch.iter().copied());
    }
    stages.sort_unstable();
    stages.dedup();

    for stage in stages {
        assert!(
            bijux_core::metrics_registry::metrics_schema_for_stage(stage).is_some(),
            "missing metrics schema for {stage}"
        );
        assert!(
            bijux_domain_fastq::stage_semantics(stage).is_some(),
            "missing report semantics for {stage}"
        );
        assert!(
            bijux_domain_fastq::contract_for_stage(stage).is_some(),
            "missing artifact contract for {stage}"
        );
    }
}
