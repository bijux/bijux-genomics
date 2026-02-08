use std::collections::BTreeMap;

use bijux_dna_benchmark_model::{GateDecision, GatePolicy};
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;

#[test]
fn gate_decision_includes_rationale_and_is_stable() -> anyhow::Result<()> {
    let policy = GatePolicy {
        objective: "runtime".to_string(),
        required_metrics: vec!["runtime_s".to_string()],
        thresholds: BTreeMap::from([("runtime_s".to_string(), 2.0)]),
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    };
    policy.validate()?;
    let mut metrics = BTreeMap::new();
    metrics.insert("runtime_s".to_string(), 1.0);
    let decision: GateDecision =
        policy.decide("dataset-1", "fastq.trim", "fastp", "params-a", &metrics);
    assert!(
        !decision.rationale_trace.is_empty(),
        "decision must include rationale trace"
    );
    let canon_a = to_canonical_json_bytes(&decision)?;
    let canon_b = to_canonical_json_bytes(&decision)?;
    assert_eq!(canon_a, canon_b);
    Ok(())
}
