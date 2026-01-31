use std::collections::BTreeMap;

use bijux_bench::GatePolicy;

#[test]
fn gate_rejects_based_on_metric_semantics() {
    let mut metrics = BTreeMap::new();
    metrics.insert("runtime_s".to_string(), 12.0);
    metrics.insert("read_retention".to_string(), 0.5);
    let mut thresholds = BTreeMap::new();
    thresholds.insert("runtime_s".to_string(), 10.0);
    thresholds.insert("read_retention".to_string(), 0.9);
    let policy = GatePolicy {
        required_metrics: Vec::new(),
        thresholds,
        regression_windows: BTreeMap::new(),
        must_not_regress: Vec::new(),
        stage_overrides: BTreeMap::new(),
    };
    let decision = policy.decide(None, &metrics);
    assert!(!decision.passes);
    assert_eq!(decision.violations.len(), 2);
}
