use std::collections::BTreeMap;

use bijux_dna_bench_model::GatePolicy;

#[test]
fn gate_policy_rejects_unknown_metrics() {
    let policy = GatePolicy {
        objective: "runtime".to_string(),
        required_metrics: vec!["not_a_metric".to_string()],
        thresholds: BTreeMap::new(),
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    };
    assert!(policy.validate().is_err());
}
