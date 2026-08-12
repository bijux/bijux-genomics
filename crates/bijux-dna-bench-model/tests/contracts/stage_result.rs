use bijux_dna_bench_model::{
    benchmark_stage_result_schema, benchmark_stage_result_validation_schema,
    validate_benchmark_stage_result_manifest, validate_benchmark_stage_result_validation_report,
    BenchmarkStageResultCommand, BenchmarkStageResultManifest, BenchmarkStageResultOutput,
    BenchmarkStageResultResourceMetricSource, BenchmarkStageResultResourceMetrics,
    BenchmarkStageResultRuntime, BenchmarkStageResultStatus, BenchmarkStageResultTool,
    BenchmarkStageResultValidationReport, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
    BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1,
};

fn manifest() -> BenchmarkStageResultManifest {
    BenchmarkStageResultManifest {
        schema_version: BENCHMARK_STAGE_RESULT_SCHEMA_V2.to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
        tool: BenchmarkStageResultTool { id: "seqkit".to_string() },
        command: BenchmarkStageResultCommand {
            rendered: "seqkit stats reads.fastq.gz".to_string(),
        },
        runtime: BenchmarkStageResultRuntime {
            mode: "apptainer".to_string(),
            status: BenchmarkStageResultStatus::Succeeded,
            started_at: "2026-08-06T12:00:00Z".to_string(),
            finished_at: "2026-08-06T12:00:01Z".to_string(),
            elapsed_seconds: 1.0,
            exit_code: 0,
        },
        resource_metrics: BenchmarkStageResultResourceMetrics {
            source: BenchmarkStageResultResourceMetricSource::Measured,
            memory_mb: Some(64.0),
            cpu_threads: Some(1),
        },
        outputs: vec![BenchmarkStageResultOutput {
            artifact_id: "report_json".to_string(),
            declared_path: "report.json".to_string(),
            realized_path: "outputs/report.json".to_string(),
            role: "report".to_string(),
            optional: false,
            exists: true,
        }],
    }
}

#[test]
fn stage_result_round_trip_preserves_the_canonical_record() -> anyhow::Result<()> {
    let expected = manifest();
    validate_benchmark_stage_result_manifest(&expected)?;
    let bytes = serde_json::to_vec(&expected)?;
    let actual: BenchmarkStageResultManifest = serde_json::from_slice(&bytes)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn stage_result_rejects_unknown_fields_and_states() -> anyhow::Result<()> {
    let mut payload = serde_json::to_value(manifest())?;
    payload["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkStageResultManifest>(payload).is_err());

    let mut payload = serde_json::to_value(manifest())?;
    payload["runtime"]["status"] = serde_json::json!("unknown");
    assert!(serde_json::from_value::<BenchmarkStageResultManifest>(payload).is_err());
    Ok(())
}

#[test]
fn stage_result_rejects_invalid_resource_and_output_shapes() {
    let mut missing_resources = manifest();
    missing_resources.resource_metrics.memory_mb = None;
    missing_resources.resource_metrics.cpu_threads = None;
    assert!(validate_benchmark_stage_result_manifest(&missing_resources).is_err());

    let mut missing_outputs = manifest();
    missing_outputs.outputs.clear();
    assert!(validate_benchmark_stage_result_manifest(&missing_outputs).is_err());
}

#[test]
fn checked_stage_result_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_stage_result_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/stage-result.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "stage-result schema drifted from model authority");
    Ok(())
}

#[test]
fn validation_report_round_trip_and_schema_match_model_authority() -> anyhow::Result<()> {
    let report = BenchmarkStageResultValidationReport {
        schema_version: BENCHMARK_STAGE_RESULT_VALIDATION_SCHEMA_V1.to_string(),
        manifest_path: "artifacts/results/stage-result.json".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
        tool_id: "seqkit".to_string(),
        output_count: 1,
        status: BenchmarkStageResultStatus::Succeeded,
        valid: true,
    };
    validate_benchmark_stage_result_validation_report(&report)?;
    let serialized = serde_json::to_vec(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkStageResultValidationReport>(&serialized)?,
        report
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_stage_result_validation_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/stage-result-validation.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "stage-result validation schema drifted from model authority");

    let mut false_claim = report;
    false_claim.valid = false;
    assert!(validate_benchmark_stage_result_validation_report(&false_claim).is_err());
    Ok(())
}
