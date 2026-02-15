use bijux_dna_analyze::{FastqValidateMetrics, StageMetricSchema};

#[test]
fn fastq_validate_metrics_invariants_pass() {
    let metrics = FastqValidateMetrics {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: None,
        pairs_out: None,
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
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: None,
        pairs_out: None,
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
