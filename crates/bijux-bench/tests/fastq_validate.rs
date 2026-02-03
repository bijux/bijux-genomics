use bijux_bench::{FastqValidateMetrics, StageMetricSchema};

#[test]
fn fastq_validate_metrics_invariants_pass() {
    let metrics = FastqValidateMetrics {
        reads_total: 100,
        reads_valid: 100,
        reads_invalid: 0,
        mean_q: 32.5,
    };
    assert!(metrics.validate().is_ok());
}

#[test]
fn fastq_validate_metrics_invariants_fail() {
    let metrics = FastqValidateMetrics {
        reads_total: 100,
        reads_valid: 90,
        reads_invalid: 20,
        mean_q: 50.0,
    };
    let err = match metrics.validate() {
        Ok(()) => panic!("expected validation error"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("reads_valid"));
}
