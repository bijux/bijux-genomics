#[test]
fn legacy_manifests_still_load() {
    let domain = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain");
    let registry = bijux_engine::api::load_registry(&domain);
    assert!(registry.is_ok());
}

#[test]
fn old_metrics_schema_is_rejected() {
    let metrics = bijux_analyze::FastqTrimMetrics {
        reads_in: 10,
        reads_out: 9,
        bases_in: 100,
        bases_out: 90,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        delta_metrics: bijux_analyze::FastqDeltaMetrics {
            read_retention: 0.9,
            base_retention: 0.9,
            mean_q_delta: 1.0,
            gc_delta: 0.1,
        },
    };
    let mut set = bijux_analyze::metric_set(metrics);
    set.metrics_schema = "fastq_trim_v0".to_string();
    match bijux_analyze::validate_metric_set(&set) {
        Ok(()) => panic!("expected schema rejection"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}
