use bijux_dna_bench_model::GateDecision;
use bijux_dna_core::contract::canonical::to_canonical_json_bytes;

#[test]
fn gate_decision_includes_rationale_and_is_stable() -> anyhow::Result<()> {
    let decision = GateDecision {
        schema_version: "bijux.bench.gate.v1".to_string(),
        dataset_id: "dataset-1".to_string(),
        stage_id: "fastq.trim_reads".to_string(),
        tool_id: "fastp".to_string(),
        params_hash: "params-a".to_string(),
        passes: true,
        violations: Vec::new(),
        missing_metrics: Vec::new(),
        completeness_score: 1.0,
        rationale_trace: vec!["metric:runtime_s:1 threshold:2".to_string()],
    };
    assert!(!decision.rationale_trace.is_empty(), "decision must include rationale trace");
    let canon_a = to_canonical_json_bytes(&decision)?;
    let canon_b = to_canonical_json_bytes(&decision)?;
    assert_eq!(canon_a, canon_b);
    Ok(())
}
