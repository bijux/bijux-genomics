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
        validated_inputs: Some(1),
        validated_pairs: None,
        pair_sync_checked: Some(false),
        pair_sync_pass: None,
        pair_count_match: None,
        strict_pass: Some(true),
        failure_class: Some("none".to_string()),
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
        validated_inputs: Some(2),
        validated_pairs: Some(45),
        pair_sync_checked: Some(true),
        pair_sync_pass: None,
        pair_count_match: Some(false),
        strict_pass: Some(false),
        failure_class: Some("pair_count_mismatch".to_string()),
    };
    let err = match metrics.validate() {
        Ok(()) => panic!("expected validation error"),
        Err(err) => err,
    };
    assert!(err.to_string().contains("reads_valid"));
}

#[test]
fn fastq_validate_metrics_require_pair_sync_outcome_when_checked() {
    let metrics = FastqValidateMetrics {
        reads_in: 100,
        reads_out: 100,
        bases_in: 1000,
        bases_out: 1000,
        pairs_in: Some(50),
        pairs_out: Some(50),
        reads_total: 100,
        reads_valid: 100,
        reads_invalid: 0,
        mean_q: 32.5,
        validated_inputs: Some(2),
        validated_pairs: Some(50),
        pair_sync_checked: Some(true),
        pair_sync_pass: None,
        pair_count_match: Some(true),
        strict_pass: Some(true),
        failure_class: Some("none".to_string()),
    };
    let err = metrics.validate().expect_err("pair sync outcome required");
    assert!(err.to_string().contains("pair_sync_pass"));
}
