use bijux_analyze::{metric_set, FastqDeltaMetrics, FastqTrimMetrics};
use bijux_core::metrics::MetricSet;

#[test]
fn metric_set_converts_to_envelope() {
    let metrics = FastqTrimMetrics {
        reads_in: 1,
        reads_out: 1,
        mean_q_before: 30.0,
        mean_q_after: 31.0,
        bases_in: 10,
        bases_out: 10,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 1.0,
            base_retention: 1.0,
            mean_q_delta: 1.0,
            gc_delta: 0.0,
        },
        adapter_preset: None,
        adapter_bank_id: None,
        adapter_bank_hash: None,
        adapter_overrides: None,
    };
    let set: MetricSet<_> = metric_set(metrics);
    assert_eq!(set.metrics_schema, "fastq_trim_v2");
    assert_eq!(set.version, 2);
}
