use std::collections::BTreeMap;

use bijux_benchmark::GatePolicy;

#[test]
fn policy_rejects_unknown_metric() {
    let mut thresholds = BTreeMap::new();
    thresholds.insert("unknown_metric".to_string(), 1.0);
    let policy = GatePolicy {
        objective: "balanced".to_string(),
        required_metrics: vec!["unknown_metric".to_string()],
        thresholds,
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    };
    let err = policy.validate().expect_err("expected error");
    assert!(err.to_string().contains("unknown metrics"));
}
