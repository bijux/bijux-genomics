mod transform_and_profiling;
mod validation_and_filter;

pub use transform_and_profiling::*;
pub use validation_and_filter::*;

#[cfg(test)]
mod tests {
    use super::super::summary::{semantic_trim, MetricValue};
    use crate::aggregate::{FastqDeltaMetrics, FastqTrimMetrics, FastqValidateMetrics};
    use crate::report::bench::semantic_validate;

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
            paired_mode: None,
            adapter_policy: None,
            polyx_policy: None,
            n_policy: None,
            contaminant_policy: None,
            raw_backend_report_format: None,
            adapter_preset: Some("default".to_string()),
            adapter_bank_id: Some("bank.v1".to_string()),
            adapter_bank_hash: Some("sha256:abc".to_string()),
            adapter_overrides: None,
        };
        let summary = semantic_trim(&metrics);
        assert!(matches!(summary.integrity.reads_in.value, MetricValue::U64(100)));
        assert!(matches!(summary.integrity.reads_out.value, MetricValue::U64(80)));
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
            validated_inputs: Some(2),
            validated_pairs: Some(22),
            pair_sync_checked: Some(true),
            pair_sync_pass: Some(false),
            pair_count_match: Some(false),
            strict_pass: Some(false),
            failure_class: Some("header_sync_mismatch".to_string()),
        };
        let summary = semantic_validate(&metrics);
        assert!(matches!(summary.integrity.reads_in.value, MetricValue::U64(50)));
        assert!(matches!(summary.integrity.reads_out.value, MetricValue::U64(45)));
        assert!(summary.quality_shift.is_none());
    }
}
