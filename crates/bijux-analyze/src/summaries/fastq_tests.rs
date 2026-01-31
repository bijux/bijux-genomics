use crate::aggregate::{FastqDeltaMetrics, FastqTrimMetrics, FastqValidateMetrics};

use super::{semantic_trim, semantic_validate, MetricValue};

#[test]
fn semantic_trim_generates_summary() {
    let metrics = FastqTrimMetrics {
        reads_in: 100,
        reads_out: 80,
        bases_in: 1000,
        bases_out: 800,
        pairs_in: None,
        pairs_out: None,
        mean_q_before: 30.0,
        mean_q_after: 31.5,
        delta_metrics: FastqDeltaMetrics {
            read_retention: 0.8,
            base_retention: 0.8,
            mean_q_delta: 1.5,
            gc_delta: 0.1,
        },
        adapter_preset: Some("default".to_string()),
        adapter_bank_id: Some("bank.v1".to_string()),
        adapter_bank_hash: Some("sha256:abc".to_string()),
        adapter_overrides: None,
    };
    let summary = semantic_trim(&metrics);
    assert!(matches!(
        summary.integrity.reads_in.value,
        MetricValue::U64(100)
    ));
    assert!(matches!(
        summary.integrity.reads_out.value,
        MetricValue::U64(80)
    ));
    assert!(summary.quality_shift.is_some());
}

#[test]
fn semantic_validate_generates_summary() {
    let metrics = FastqValidateMetrics {
        reads_in: 50,
        reads_out: 50,
        bases_in: 500,
        bases_out: 500,
        pairs_in: None,
        pairs_out: None,
        reads_total: 50,
        reads_valid: 45,
        reads_invalid: 5,
        mean_q: 32.0,
    };
    let summary = semantic_validate(&metrics);
    assert!(matches!(
        summary.integrity.reads_in.value,
        MetricValue::U64(50)
    ));
    assert!(matches!(
        summary.integrity.reads_out.value,
        MetricValue::U64(45)
    ));
    assert!(summary.quality_shift.is_none());
}
