use std::collections::BTreeMap;

use bijux_dna_bench_model::GatePolicy;
use bijux_dna_core::metrics::MetricDirection;
use bijux_dna_science::decision::{evaluate_benchmark_gate_policy, validate_benchmark_gate_policy};

fn policy(required_metrics: Vec<String>, thresholds: BTreeMap<String, f64>) -> GatePolicy {
    GatePolicy {
        objective: "scientific acceptance".to_string(),
        required_metrics,
        thresholds,
        allowed_regressions: BTreeMap::new(),
        must_not_regress: Vec::new(),
        semantics_overrides: BTreeMap::new(),
        stage_overrides: BTreeMap::new(),
    }
}

#[test]
fn missing_required_measurements_fail_closed() {
    let policy = policy(vec!["runtime_s".to_string()], BTreeMap::new());
    let decision = evaluate_benchmark_gate_policy(
        &policy,
        "dataset-1",
        "fastq.trim_reads",
        "fastp",
        "params-a",
        &BTreeMap::new(),
    );

    assert!(!decision.passes);
    assert_eq!(decision.missing_metrics, ["runtime_s"]);
    assert!(decision.rationale_trace.iter().any(|reason| reason == "missing_required:runtime_s"));
}

#[test]
fn measured_threshold_failures_record_observation_and_rule() {
    let policy = policy(Vec::new(), BTreeMap::from([("read_retention".to_string(), 0.9)]));
    let metrics = BTreeMap::from([("read_retention".to_string(), 0.5)]);
    let decision = evaluate_benchmark_gate_policy(
        &policy,
        "dataset-1",
        "fastq.trim_reads",
        "fastp",
        "params-a",
        &metrics,
    );

    assert!(!decision.passes);
    assert_eq!(decision.violations.len(), 1);
    assert!((decision.violations[0].observed - 0.5).abs() < f64::EPSILON);
    assert!((decision.violations[0].threshold - 0.9).abs() < f64::EPSILON);
    assert!(!decision.rationale_trace.is_empty());
}

#[test]
fn unknown_metric_semantics_are_rejected_before_evaluation() -> anyhow::Result<()> {
    let policy = policy(
        vec!["unregistered_metric".to_string()],
        BTreeMap::from([("unregistered_metric".to_string(), 1.0)]),
    );
    let error = validate_benchmark_gate_policy(&policy)
        .err()
        .ok_or_else(|| anyhow::anyhow!("unknown metric was accepted"))?;

    assert!(error.to_string().contains("unknown metrics: unregistered_metric"));
    Ok(())
}

#[test]
fn explicit_metric_semantics_make_custom_rules_auditable() -> anyhow::Result<()> {
    let mut policy = policy(
        vec!["governed_custom_metric".to_string()],
        BTreeMap::from([("governed_custom_metric".to_string(), 0.8)]),
    );
    policy
        .semantics_overrides
        .insert("governed_custom_metric".to_string(), MetricDirection::HigherBetter);

    validate_benchmark_gate_policy(&policy)?;
    let decision = evaluate_benchmark_gate_policy(
        &policy,
        "dataset-1",
        "fastq.trim_reads",
        "fastp",
        "params-a",
        &BTreeMap::from([("governed_custom_metric".to_string(), 0.9)]),
    );

    assert!(decision.passes);
    Ok(())
}
