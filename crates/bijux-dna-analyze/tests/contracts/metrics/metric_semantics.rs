use bijux_dna_analyze::metric_semantics;

#[test]
fn metrics_used_for_ranking_have_semantics() {
    let metric_ids = [
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "merge_rate",
        "error_reduction_proxy",
    ];
    for metric_id in metric_ids {
        assert!(
            metric_semantics(metric_id).is_some(),
            "missing semantics for {metric_id}"
        );
    }
}
