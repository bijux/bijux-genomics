use bijux_dna_bench_model::{
    BenchmarkAttemptRecord, BenchmarkCycleStatus, BenchmarkJobReference, BenchmarkLifecycleStatus,
    ColdWarmContract, DecisionTraceRow, EnvironmentSnapshot, FairnessContract, MetricsArtifactRow,
    PostconditionRow, RunRecordRow, ScientificDefaultsStatus, SuiteRunManifest, ToolInvocationRow,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2,
};
use bijux_dna_core::id_catalog;

#[test]
fn checked_suite_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(bijux_dna_bench_model::benchmark_suite_spec_schema())?;
    let path = repository_root.join("benchmarks/contracts/schemas/suite.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "suite schema drifted from model authority");
    Ok(())
}

fn suite_run_manifest() -> SuiteRunManifest {
    SuiteRunManifest {
        schema_version: BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2.to_string(),
        suite_id: "suite-fastq".to_string(),
        run_id: "run-2001".to_string(),
        run_context: "publication".to_string(),
        corpus: "fastq-corpus".to_string(),
        species_id: Some("homo_sapiens".to_string()),
        fairness: FairnessContract {
            threads: 4,
            mem_gb: 8,
            tmp_policy: "isolated".to_string(),
            tmp_root: "/study/scratch/run-2001".to_string(),
            deviations: Vec::new(),
        },
        cold_vs_warm: ColdWarmContract {
            cold_runs: 1,
            warm_runs: 3,
            cold_run_records_image_cost: true,
            warm_runs_exclude_image_cost: true,
        },
        decision_trace: vec![DecisionTraceRow {
            stage: "fastq.trim".to_string(),
            candidates: vec!["fastp".to_string()],
            selected_tool: "fastp".to_string(),
            reason: "declared binding".to_string(),
        }],
        tool_invocations: vec![ToolInvocationRow {
            stage: "fastq.trim".to_string(),
            tool: "fastp".to_string(),
            mode: "warm".to_string(),
            run_index: 1,
            tool_version: "1.0.0".to_string(),
            image_digest: "sha256:0123".to_string(),
        }],
        metrics_artifacts: vec![MetricsArtifactRow {
            stage: "fastq.trim".to_string(),
            tool: "fastp".to_string(),
            mode: "warm".to_string(),
            run_index: 1,
            path: "metrics.json".to_string(),
        }],
        postconditions: vec![PostconditionRow {
            stage: "fastq.trim".to_string(),
            tool: "fastp".to_string(),
            mode: "warm".to_string(),
            run_index: 1,
            ok: true,
            checks: vec!["output_exists".to_string()],
        }],
        run_records: vec![RunRecordRow {
            stage: "fastq.trim".to_string(),
            tool: "fastp".to_string(),
            mode: "warm".to_string(),
            run_index: 1,
            runtime_s: 1.5,
            memory_mb: 512.0,
            read_retention: 0.99,
            length_shift: -2.0,
            delta_metrics: None,
            valid: true,
        }],
        comparability_hash: "0".repeat(64),
        environment: EnvironmentSnapshot {
            apptainer_version: "1.4.0".to_string(),
            kernel: "linux".to_string(),
            site_lock: serde_json::json!({"site": "lunarc"}),
        },
        scientific_defaults: ScientificDefaultsStatus {
            doc_path: "docs/scientific-defaults.md".to_string(),
            reference_defaults_ok: true,
            checked_rows: 1,
        },
        telemetry_path: "telemetry.jsonl".to_string(),
        reproducibility_bundle: "bundle.tar.gz".to_string(),
        created_at_unix: 1_786_467_600,
    }
}

#[test]
fn suite_run_manifest_round_trips_under_model_authority() -> anyhow::Result<()> {
    let expected = suite_run_manifest();
    let mut value = serde_json::to_value(&expected)?;
    let actual = serde_json::from_value::<SuiteRunManifest>(value.clone())?;
    assert_eq!(actual, expected);
    value["schema_version"] = serde_json::json!("bijux.bench.suite_run_manifest.v3");
    assert!(serde_json::from_value::<SuiteRunManifest>(value).is_err());

    let mut unknown = serde_json::to_value(suite_run_manifest())?;
    unknown["unowned_field"] = serde_json::json!(true);
    assert!(serde_json::from_value::<SuiteRunManifest>(unknown).is_err());
    Ok(())
}

#[test]
fn checked_suite_run_manifest_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated =
        serde_json::to_value(bijux_dna_bench_model::benchmark_suite_run_manifest_schema())?;
    assert_eq!(
        generated.pointer("/properties/schema_version/const").and_then(serde_json::Value::as_str),
        Some(BENCHMARK_SUITE_RUN_MANIFEST_SCHEMA_V2)
    );
    let path = repository_root.join("benchmarks/contracts/schemas/suite-run-manifest.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "suite run manifest schema drifted from model authority");
    Ok(())
}

#[test]
fn benchmark_attempt_record_uses_governed_schema_and_status() {
    let mut attempt = BenchmarkAttemptRecord::v1(
        "lunarc-fastq".to_string(),
        "run-2042".to_string(),
        "attempt-01".to_string(),
        id_catalog::FASTQ_PROFILE_READ_LENGTHS.to_string(),
        "seqkit_stats".to_string(),
        BenchmarkLifecycleStatus::Ready,
    );
    attempt.jobs.push(BenchmarkJobReference {
        scheduler: "slurm".to_string(),
        job_id: "4123456".to_string(),
    });
    attempt
        .evidence_paths
        .push("benchmarks/reports/run-2042-fastq-profile-read-lengths.json".to_string());

    assert_eq!(attempt.schema_version, BENCHMARK_ATTEMPT_SCHEMA_V1);
    assert_eq!(attempt.status, BenchmarkLifecycleStatus::Ready);
    assert_eq!(attempt.jobs.len(), 1);
}

#[test]
fn benchmark_cycle_status_uses_governed_schema() {
    let attempt = BenchmarkAttemptRecord::v1(
        "lunarc-fastq".to_string(),
        "run-2043".to_string(),
        "attempt-02".to_string(),
        id_catalog::FASTQ_PROFILE_READ_LENGTHS.to_string(),
        "seqfu".to_string(),
        BenchmarkLifecycleStatus::Submitted,
    );
    let cycle = BenchmarkCycleStatus::v1(
        "lunarc-fastq".to_string(),
        "2026-08-04T00:00:00Z".to_string(),
        vec![attempt],
    );

    assert_eq!(cycle.schema_version, BENCHMARK_CYCLE_STATUS_SCHEMA_V1);
    assert_eq!(cycle.attempts.len(), 1);
    assert_eq!(cycle.attempts[0].status, BenchmarkLifecycleStatus::Submitted);
}
