#[test]
fn legacy_manifests_still_load() {
    let domain = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("domain");
    let registry = bijux_engine::planner::load_registry(&domain);
    assert!(registry.is_ok());
}

#[test]
fn old_metrics_schema_is_rejected() {
    let metrics = bijux_bench::FastqTrimMetrics {
        reads_in: 10,
        reads_out: 9,
        bases_in: 100,
        bases_out: 90,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
    };
    let mut set = bijux_bench::MetricSet::new(metrics);
    set.metrics_schema = "fastq_trim_v0".to_string();
    match set.validate() {
        Ok(()) => panic!("expected schema rejection"),
        Err(err) => assert!(err.to_string().contains("metric schema mismatch")),
    }
}
