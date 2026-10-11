#![cfg(test)]
use super::checks::check_normalized_metrics_ready;
use super::declarations::*;
use super::models::LocalEndogenousContentSmokeMetrics;
fn metrics() -> LocalEndogenousContentSmokeMetrics {
    LocalEndogenousContentSmokeMetrics {
        schema_version: EXPECTED_STAGE_METRICS_SCHEMA_VERSION.into(),
        stage_id: EXPECTED_STAGE_ID.into(),
        sample_id: EXPECTED_SAMPLE_ID.into(),
        expected_method: EXPECTED_METHOD.into(),
        method: EXPECTED_METHOD.into(),
        expected_host_reference_scope: EXPECTED_HOST_REFERENCE_SCOPE.into(),
        host_reference_scope: Some(EXPECTED_HOST_REFERENCE_SCOPE.into()),
        expected_mapped_reads: EXPECTED_MAPPED_READS,
        mapped_reads: EXPECTED_MAPPED_READS,
        mapped_read_delta: 0,
        expected_endogenous_reads: EXPECTED_ENDOGENOUS_READS,
        endogenous_reads: EXPECTED_ENDOGENOUS_READS,
        expected_total_reads: EXPECTED_TOTAL_READS,
        total_reads: EXPECTED_TOTAL_READS,
        total_read_delta: 0,
        expected_endogenous_fraction: EXPECTED_ENDOGENOUS_FRACTION,
        endogenous_fraction: EXPECTED_ENDOGENOUS_FRACTION,
        endogenous_fraction_delta: 0.0,
        prealignment_fraction: None,
        expectation_matched: true,
    }
}
#[test]
fn rejects_foreign_sample_scope_and_metric_denominator() {
    assert!(check_normalized_metrics_ready(&metrics()));
    let mut value = metrics();
    value.sample_id = "foreign".into();
    assert!(!check_normalized_metrics_ready(&value));
    let mut value = metrics();
    value.host_reference_scope = Some("foreign".into());
    assert!(!check_normalized_metrics_ready(&value));
    let mut value = metrics();
    value.total_reads += 1;
    assert!(!check_normalized_metrics_ready(&value));
    let mut value = metrics();
    value.schema_version = "unknown".into();
    assert!(!check_normalized_metrics_ready(&value));
}
#[test]
fn rejects_nonfinite_fractions_and_invented_prealignment_observations() {
    for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        let mut value = metrics();
        value.endogenous_fraction = bad;
        assert!(!check_normalized_metrics_ready(&value));
    }
    let mut value = metrics();
    value.prealignment_fraction = Some(0.0);
    assert!(!check_normalized_metrics_ready(&value));
    let mut value = metrics();
    value.endogenous_fraction_delta = 0.1;
    assert!(!check_normalized_metrics_ready(&value));
}
