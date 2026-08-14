use bijux_dna_bench_model::{
    validate_stage_tool_benchmark_ready_report, StageToolBenchmarkReadyReport,
    StageToolBenchmarkReadySurfaceStatus, StageToolBenchmarkReadySurfaceSummary,
    STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1,
};

fn ready_report() -> StageToolBenchmarkReadyReport {
    StageToolBenchmarkReadyReport {
        schema_version: STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/stage-tool.json".to_string(),
        passes_gate: true,
        expected_pair_count: 2,
        benchmark_ready_pair_count: 2,
        excluded_pair_count: 0,
        failing_pair_count: 0,
        generated_job_pair_count: 2,
        expected_result_pair_count: 2,
        benchmark_ready_stage_count: 1,
        excluded_registry_gap_count: 0,
        surface_summaries: vec![StageToolBenchmarkReadySurfaceSummary {
            surface_id: "stage_tool_matrix".to_string(),
            surface_status: StageToolBenchmarkReadySurfaceStatus::ReadySliceComplete,
            measured_scope: "admitted_pairs".to_string(),
            expected_count: 2,
            covered_count: 2,
            excluded_count: 0,
            failing_count: 0,
            evidence_paths: vec!["benchmarks/reports/STAGE_TOOL_CLOSURE.json".to_string()],
            detail: "all admitted pairs are represented".to_string(),
        }],
        failing_pairs: Vec::new(),
        excluded_pairs: Vec::new(),
    }
}

#[test]
fn stage_tool_readiness_contract_accepts_closed_ready_report() {
    validate_stage_tool_benchmark_ready_report(&ready_report()).expect("valid readiness report");
}

#[test]
fn stage_tool_readiness_contract_rejects_gate_drift() {
    let mut report = ready_report();
    report.passes_gate = false;
    let error = validate_stage_tool_benchmark_ready_report(&report)
        .expect_err("gate state must close against failing pairs");
    assert!(error.to_string().contains("gate state disagrees"));
}

#[test]
fn stage_tool_readiness_schema_pins_the_protocol_version() {
    let schema = serde_json::to_value(bijux_dna_bench_model::stage_tool_benchmark_ready_schema())
        .expect("serialize schema");
    assert_eq!(
        schema.pointer("/properties/schema_version/const"),
        Some(&serde_json::json!(STAGE_TOOL_BENCHMARK_READY_SCHEMA_V1))
    );
}
