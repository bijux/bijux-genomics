use std::collections::BTreeMap;

use bijux_dna_bench_model::GatePolicy;
use bijux_dna_core::metrics::MetricDirection;

#[test]
fn gate_policy_preserves_explicit_metric_semantics_inputs() {
    let policy = GatePolicy {
        objective: "runtime".to_string(),
        required_metrics: vec!["governed_custom_metric".to_string()],
        thresholds: BTreeMap::new(),
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::from([(
            "governed_custom_metric".to_string(),
            MetricDirection::HigherBetter,
        )]),
        stage_overrides: BTreeMap::new(),
    };
    assert_eq!(
        policy.semantics_overrides.get("governed_custom_metric"),
        Some(&MetricDirection::HigherBetter)
    );
}
