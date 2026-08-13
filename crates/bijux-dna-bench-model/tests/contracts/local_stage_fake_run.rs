use bijux_dna_bench_model::{
    benchmark_local_stage_fake_failure_manifest_schema,
    benchmark_local_stage_fake_failure_record_schema,
    benchmark_local_stage_fake_run_manifest_schema,
    validate_benchmark_local_stage_fake_failure_manifest,
    validate_benchmark_local_stage_fake_failure_record,
    validate_benchmark_local_stage_fake_run_manifest, BenchmarkLocalStageFakeFailureManifest,
    BenchmarkLocalStageFakeFailureOutputEntry, BenchmarkLocalStageFakeFailureRecord,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageFakeRunOutputEntry,
    BenchmarkLocalStageFakeRunResult, BenchmarkLocalStageReadinessKind,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
};

const RUN_ROOT: &str = "artifacts/benchmark-smoke/simulated-runs/stages";
const FAILURE_ROOT: &str = "artifacts/benchmark-smoke/simulated-runs/failures";

fn run_manifest() -> BenchmarkLocalStageFakeRunManifest {
    BenchmarkLocalStageFakeRunManifest {
        schema_version: BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1.to_string(),
        fake_run_root: RUN_ROOT.to_string(),
        source_stage_command_manifest_path:
            "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.sh".to_string(),
        stage_count: 1,
        stages: vec![BenchmarkLocalStageFakeRunResult {
            schema_version: BENCHMARK_STAGE_RESULT_SCHEMA_V2.to_string(),
            stage_id: "fastq.report_qc".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
            tool_id: "multiqc".to_string(),
            command: "multiqc results".to_string(),
            stage_manifest_path: format!("{RUN_ROOT}/fastq.report_qc/stage-result.json"),
            declared_output_count: 2,
            created_output_count: 2,
            outputs: vec![
                BenchmarkLocalStageFakeRunOutputEntry {
                    artifact_id: "report_html".to_string(),
                    declared_path: "sample-a/report.html".to_string(),
                    fake_run_path: format!(
                        "{RUN_ROOT}/fastq.report_qc/declared-outputs/sample-a/report.html"
                    ),
                    role: "report".to_string(),
                    optional: false,
                    exists: true,
                },
                BenchmarkLocalStageFakeRunOutputEntry {
                    artifact_id: "report_html".to_string(),
                    declared_path: "sample-b/report.html".to_string(),
                    fake_run_path: format!(
                        "{RUN_ROOT}/fastq.report_qc/declared-outputs/sample-b/report.html"
                    ),
                    role: "report".to_string(),
                    optional: false,
                    exists: true,
                },
            ],
        }],
    }
}

fn failure_record() -> BenchmarkLocalStageFakeFailureRecord {
    BenchmarkLocalStageFakeFailureRecord {
        schema_version: BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1.to_string(),
        stage_id: "fastq.report_qc".to_string(),
        readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
        tool_id: "multiqc".to_string(),
        command: "multiqc results".to_string(),
        exit_code: 1,
        stderr_path: format!("{FAILURE_ROOT}/fastq.report_qc/stderr.txt"),
        failure_record_path: format!("{FAILURE_ROOT}/fastq.report_qc/failure.json"),
        failed_output_count: 2,
        failed_outputs: vec![
            BenchmarkLocalStageFakeFailureOutputEntry {
                artifact_id: "report_html".to_string(),
                declared_path: "sample-a/report.html".to_string(),
                expected_fake_run_path: format!(
                    "{FAILURE_ROOT}/fastq.report_qc/declared-outputs/sample-a/report.html"
                ),
                role: "report".to_string(),
                optional: false,
            },
            BenchmarkLocalStageFakeFailureOutputEntry {
                artifact_id: "report_html".to_string(),
                declared_path: "sample-b/report.html".to_string(),
                expected_fake_run_path: format!(
                    "{FAILURE_ROOT}/fastq.report_qc/declared-outputs/sample-b/report.html"
                ),
                role: "report".to_string(),
                optional: false,
            },
        ],
    }
}

fn failure_manifest() -> BenchmarkLocalStageFakeFailureManifest {
    BenchmarkLocalStageFakeFailureManifest {
        schema_version: BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1.to_string(),
        failure_root: FAILURE_ROOT.to_string(),
        source_stage_command_manifest_path:
            "artifacts/benchmark-smoke/readiness/local-ready/rendered-stage-commands.sh".to_string(),
        stage_count: 1,
        failures: vec![failure_record()],
    }
}

#[test]
fn local_stage_fake_run_records_round_trip_and_match_generated_schemas() -> anyhow::Result<()> {
    let run = run_manifest();
    let failure = failure_manifest();
    validate_benchmark_local_stage_fake_run_manifest(&run)?;
    validate_benchmark_local_stage_fake_failure_manifest(&failure)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalStageFakeRunManifest>(&serde_json::to_vec(&run)?)?,
        run
    );
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalStageFakeFailureManifest>(&serde_json::to_vec(
            &failure
        )?)?,
        failure
    );

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    for (relative_path, generated) in [
        (
            "benchmarks/contracts/schemas/local-stage-fake-runs.schema.json",
            benchmark_local_stage_fake_run_manifest_schema(),
        ),
        (
            "benchmarks/contracts/schemas/local-stage-fake-failures.schema.json",
            benchmark_local_stage_fake_failure_manifest_schema(),
        ),
        (
            "benchmarks/contracts/schemas/local-stage-fake-failure-record.schema.json",
            benchmark_local_stage_fake_failure_record_schema(),
        ),
    ] {
        let generated = serde_json::to_value(generated)?;
        let path = root.join(relative_path);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{relative_path} drifted");
    }
    Ok(())
}

#[test]
fn local_stage_fake_run_admission_rejects_output_and_path_drift() {
    let mut missing = run_manifest();
    missing.stages[0].outputs[0].exists = false;
    missing.stages[0].created_output_count = 0;
    assert!(validate_benchmark_local_stage_fake_run_manifest(&missing).is_err());

    let mut counter = run_manifest();
    counter.stage_count = 2;
    assert!(validate_benchmark_local_stage_fake_run_manifest(&counter).is_err());

    let mut outside = run_manifest();
    outside.stages[0].stage_manifest_path = "artifacts/outside/stage-result.json".to_string();
    assert!(validate_benchmark_local_stage_fake_run_manifest(&outside).is_err());

    let mut unsafe_path = run_manifest();
    unsafe_path.stages[0].outputs[0].fake_run_path = "../outside/report.html".to_string();
    assert!(validate_benchmark_local_stage_fake_run_manifest(&unsafe_path).is_err());

    let mut cross_stage = run_manifest();
    cross_stage.stages[0].outputs[0].fake_run_path =
        format!("{RUN_ROOT}/other-stage/declared-outputs/report.html");
    assert!(validate_benchmark_local_stage_fake_run_manifest(&cross_stage).is_err());

    let mut duplicate_output = run_manifest();
    let repeated = duplicate_output.stages[0].outputs[0].clone();
    duplicate_output.stages[0].outputs.push(repeated);
    duplicate_output.stages[0].declared_output_count = 3;
    duplicate_output.stages[0].created_output_count = 3;
    assert!(validate_benchmark_local_stage_fake_run_manifest(&duplicate_output).is_err());
}

#[test]
fn local_stage_fake_failure_admission_rejects_success_duplicates_and_root_escape() {
    let mut successful = failure_record();
    successful.exit_code = 0;
    assert!(validate_benchmark_local_stage_fake_failure_record(&successful).is_err());

    let mut cross_stage = failure_record();
    cross_stage.stderr_path = format!("{FAILURE_ROOT}/other-stage/stderr.txt");
    assert!(validate_benchmark_local_stage_fake_failure_record(&cross_stage).is_err());

    let mut duplicate = failure_manifest();
    duplicate.failures.push(duplicate.failures[0].clone());
    duplicate.stage_count = 2;
    assert!(validate_benchmark_local_stage_fake_failure_manifest(&duplicate).is_err());

    let mut outside = failure_manifest();
    outside.failures[0].stderr_path = "artifacts/outside/stderr.txt".to_string();
    assert!(validate_benchmark_local_stage_fake_failure_manifest(&outside).is_err());

    let mut count = failure_manifest();
    count.failures[0].failed_output_count = 3;
    assert!(validate_benchmark_local_stage_fake_failure_manifest(&count).is_err());

    let mut duplicate_output = failure_manifest();
    let repeated = duplicate_output.failures[0].failed_outputs[0].clone();
    duplicate_output.failures[0].failed_outputs.push(repeated);
    duplicate_output.failures[0].failed_output_count = 3;
    assert!(validate_benchmark_local_stage_fake_failure_manifest(&duplicate_output).is_err());
}
