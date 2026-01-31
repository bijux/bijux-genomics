use std::collections::BTreeMap;

use bijux_bench::GatePolicy;

#[test]
fn policy_rejects_unknown_metric() {
    let mut thresholds = BTreeMap::new();
    thresholds.insert("unknown_metric".to_string(), 1.0);
    let policy = GatePolicy {
        required_metrics: vec!["unknown_metric".to_string()],
        thresholds,
        regression_windows: BTreeMap::new(),
        must_not_regress: Vec::new(),
        stage_overrides: BTreeMap::new(),
    };
    let err = policy.validate().expect_err("expected error");
    assert!(err.to_string().contains("unknown metrics"));
}
