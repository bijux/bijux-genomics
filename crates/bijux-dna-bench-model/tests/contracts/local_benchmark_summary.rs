use bijux_dna_bench_model::{
    benchmark_local_summary_schema, validate_benchmark_local_summary,
    BenchmarkLocalReadinessStatus, BenchmarkLocalStageReadinessKind, BenchmarkLocalSummaryReport,
    BenchmarkLocalSummaryStage, BenchmarkStageResultStatus, BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1,
};

fn report() -> BenchmarkLocalSummaryReport {
    BenchmarkLocalSummaryReport {
        schema_version: BENCHMARK_LOCAL_SUMMARY_SCHEMA_V1.to_string(),
        fake_run_root: "artifacts/benchmark-smoke/simulated-runs/stages".to_string(),
        report_output_path: "artifacts/benchmark-smoke/readiness/benchmark-summary.json"
            .to_string(),
        markdown_output_path: "artifacts/benchmark-smoke/readiness/benchmark-summary.md"
            .to_string(),
        source_stage_command_manifest_path:
            "artifacts/benchmark-smoke/readiness/rendered-stage-commands.sh".to_string(),
        manifest_completion_report_path:
            "artifacts/benchmark-smoke/readiness/manifest-completion.json".to_string(),
        output_completion_report_path: "artifacts/benchmark-smoke/readiness/output-completion.json"
            .to_string(),
        runtime_metrics_report_path: "artifacts/benchmark-smoke/readiness/runtime-metrics.json"
            .to_string(),
        tool_comparison_template_path: "artifacts/benchmark-smoke/readiness/tool-comparison.tsv"
            .to_string(),
        stage_count: 1,
        ready_stage_count: 1,
        incomplete_stage_count: 0,
        failed_stage_count: 0,
        stages: vec![BenchmarkLocalSummaryStage {
            stage_id: "fastq.report_qc".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::DryOrSmoke,
            tool_id: "multiqc".to_string(),
            readiness_status: BenchmarkLocalReadinessStatus::Ready,
            manifest_exists: true,
            declared_output_count: 1,
            present_output_count: 1,
            missing_output_count: 0,
            runtime_seconds: "1.0".to_string(),
            runtime_status: BenchmarkStageResultStatus::Succeeded,
            memory_mb: "32".to_string(),
            output_metric: "report_count=1".to_string(),
            failure_reason: String::new(),
        }],
    }
}

#[test]
fn local_benchmark_summary_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_summary(&report)?;
    let serialized = serde_json::to_vec(&report)?;
    assert_eq!(serde_json::from_slice::<BenchmarkLocalSummaryReport>(&serialized)?, report);

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_summary_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-benchmark-summary.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local benchmark-summary schema drifted from model authority");
    Ok(())
}

#[test]
fn local_benchmark_summary_rejects_duplicate_and_inconsistent_rows() {
    let mut duplicate = report();
    duplicate.stages.push(duplicate.stages[0].clone());
    duplicate.stage_count = 2;
    duplicate.ready_stage_count = 2;
    assert!(validate_benchmark_local_summary(&duplicate).is_err());

    let mut inconsistent = report();
    inconsistent.stages[0].missing_output_count = 1;
    assert!(validate_benchmark_local_summary(&inconsistent).is_err());

    let mut stale_counts = report();
    stale_counts.incomplete_stage_count = 1;
    assert!(validate_benchmark_local_summary(&stale_counts).is_err());

    let mut contradictory = report();
    contradictory.stages[0].readiness_status = BenchmarkLocalReadinessStatus::Failed;
    contradictory.ready_stage_count = 0;
    contradictory.incomplete_stage_count = 1;
    contradictory.failed_stage_count = 1;
    assert!(validate_benchmark_local_summary(&contradictory).is_err());
}
