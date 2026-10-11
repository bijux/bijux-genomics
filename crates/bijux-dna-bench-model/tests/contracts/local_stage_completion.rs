use bijux_dna_bench_model::{
    benchmark_local_stage_manifest_completion_schema,
    benchmark_local_stage_output_completion_schema,
    validate_benchmark_local_stage_manifest_completion,
    validate_benchmark_local_stage_output_completion, BenchmarkLocalStageManifestCompletionEntry,
    BenchmarkLocalStageManifestCompletionReport, BenchmarkLocalStageMissingOutputEntry,
    BenchmarkLocalStageOutputCompletionEntry, BenchmarkLocalStageOutputCompletionReport,
    BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1,
};

fn manifest_report() -> BenchmarkLocalStageManifestCompletionReport {
    BenchmarkLocalStageManifestCompletionReport {
        schema_version: BENCHMARK_LOCAL_STAGE_MANIFEST_COMPLETION_SCHEMA_V1.to_string(),
        fake_run_root: "artifacts/benchmark-smoke/simulated-runs/stages".to_string(),
        report_output_path: "artifacts/benchmark-smoke/readiness/manifests.json".to_string(),
        source_stage_command_manifest_path:
            "artifacts/benchmark-smoke/readiness/rendered-stage-commands.sh".to_string(),
        stage_count: 1,
        complete_stage_count: 1,
        incomplete_stage_count: 0,
        complete: true,
        stages: vec![BenchmarkLocalStageManifestCompletionEntry {
            stage_id: "fastq.report_qc".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::DryOrSmoke,
            tool_id: "multiqc".to_string(),
            stage_manifest_path: "artifacts/results/stage-result.json".to_string(),
            manifest_exists: true,
            declared_output_count: 1,
            present_output_count: 1,
            complete: true,
        }],
    }
}

fn output_report() -> BenchmarkLocalStageOutputCompletionReport {
    BenchmarkLocalStageOutputCompletionReport {
        schema_version: BENCHMARK_LOCAL_STAGE_OUTPUT_COMPLETION_SCHEMA_V1.to_string(),
        fake_run_root: "artifacts/benchmark-smoke/simulated-runs/stages".to_string(),
        report_output_path: "artifacts/benchmark-smoke/readiness/outputs.json".to_string(),
        source_stage_command_manifest_path:
            "artifacts/benchmark-smoke/readiness/rendered-stage-commands.sh".to_string(),
        stage_count: 1,
        complete_stage_count: 0,
        incomplete_stage_count: 1,
        complete: false,
        stages: vec![BenchmarkLocalStageOutputCompletionEntry {
            stage_id: "fastq.report_qc".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::DryOrSmoke,
            tool_id: "multiqc".to_string(),
            declared_output_count: 1,
            present_output_count: 0,
            missing_output_count: 1,
            complete: false,
            missing_outputs: vec![BenchmarkLocalStageMissingOutputEntry {
                artifact_id: "report_json".to_string(),
                declared_path: "report.json".to_string(),
                expected_fake_run_path: "artifacts/results/report.json".to_string(),
                role: "report".to_string(),
                optional: false,
            }],
        }],
    }
}

#[test]
fn local_stage_completion_reports_round_trip_and_match_model_schemas() -> anyhow::Result<()> {
    let manifest = manifest_report();
    let output = output_report();
    validate_benchmark_local_stage_manifest_completion(&manifest)?;
    validate_benchmark_local_stage_output_completion(&output)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalStageManifestCompletionReport>(
            &serde_json::to_vec(&manifest)?
        )?,
        manifest
    );
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalStageOutputCompletionReport>(&serde_json::to_vec(
            &output
        )?)?,
        output
    );

    assert_schema(
        "local-stage-manifest-completion.schema.json",
        benchmark_local_stage_manifest_completion_schema(),
    )?;
    assert_schema(
        "local-stage-output-completion.schema.json",
        benchmark_local_stage_output_completion_schema(),
    )
}

#[test]
fn local_stage_completion_rejects_inconsistent_counts() {
    let mut manifest = manifest_report();
    manifest.stages[0].complete = false;
    assert!(validate_benchmark_local_stage_manifest_completion(&manifest).is_err());

    let mut output = output_report();
    output.stages[0].missing_output_count = 0;
    assert!(validate_benchmark_local_stage_output_completion(&output).is_err());
}

fn assert_schema(file_name: &str, schema: schemars::Schema) -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(schema)?;
    let path = repository_root.join("benchmarks/contracts/schemas").join(file_name);
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    anyhow::ensure!(checked == generated, "{file_name} drifted from model authority");
    Ok(())
}
