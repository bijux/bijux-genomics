use std::collections::BTreeMap;

use bijux_dna_bench::GatePolicy;

#[test]
fn gate_rejects_based_on_metric_semantics() {
    let mut metrics = BTreeMap::new();
    metrics.insert("runtime_s".to_string(), 12.0);
    metrics.insert("read_retention".to_string(), 0.5);
    let mut thresholds = BTreeMap::new();
    thresholds.insert("runtime_s".to_string(), 10.0);
    thresholds.insert("read_retention".to_string(), 0.9);
    let policy = GatePolicy {
        objective: "balanced".to_string(),
        required_metrics: Vec::new(),
        thresholds,
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    };
    let decision = policy.decide("dataset-1", "fastq.trim", "tool-a", "params-a", &metrics);
    assert!(!decision.passes);
    assert_eq!(decision.violations.len(), 2);
}
