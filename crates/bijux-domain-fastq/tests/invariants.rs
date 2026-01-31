#[test]
fn legacy_manifests_still_load() {
    let domain = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain");
    let registry = bijux_core::load_manifests(&domain);
    assert!(registry.is_ok());
}

#[test]
fn old_metrics_schema_is_rejected() {
    let metrics = bijux_analyze::FastqTrimMetrics {
        reads_in: 10,
        reads_out: 9,
        bases_in: 100,
        bases_out: 90,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: bijux_analyze::FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    };
    let mut set = bijux_analyze::metric_set(metrics);
    set.metrics_schema = "fastq_trim_v0".to_string();
    match bijux_analyze::validate_metric_set(&set) {
        Ok(()) => panic!("expected schema rejection"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}

#[test]
fn domain_onboarding_checklist_is_satisfied() {
    let domain = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain");
    let registry = bijux_core::load_manifests(&domain);
    assert!(registry.is_ok(), "stage registry missing");

    let mut stages = bijux_domain_fastq::canonical_stage_order();
    for (_, branch) in bijux_domain_fastq::optional_branches() {
        stages.extend(branch.iter().copied());
    }
    stages.sort_unstable();
    stages.dedup();

    for stage in stages {
        assert!(
            bijux_core::metrics_schema_for_stage(stage).is_some(),
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
