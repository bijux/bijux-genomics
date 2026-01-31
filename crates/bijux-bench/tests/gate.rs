use std::collections::BTreeMap;

use bijux_bench::gate::GatePolicy;

#[test]
fn gate_rejects_based_on_metric_semantics() {
    let metrics = serde_json::json!({
        "runtime_s": 12.0,
        "read_retention": 0.5
    });
    let mut thresholds = BTreeMap::new();
    thresholds.insert("runtime_s".to_string(), 10.0);
    thresholds.insert("read_retention".to_string(), 0.9);
    let policy = GatePolicy {
        required_metrics: Vec::new(),
        thresholds,
        regression_windows: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    };
    let decision = policy.decide(None, &metrics);
    assert!(!decision.passes);
    assert_eq!(decision.violations.len(), 2);
}
