use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_execution_admission_schema, benchmark_execution_spec_schema, validate_execution_spec,
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkAcceptanceOperator,
    BenchmarkAttemptIdentity, BenchmarkEffectiveParameters, BenchmarkExecutionAcceptance,
    BenchmarkExecutionAdmission, BenchmarkExecutionCachePolicy, BenchmarkExecutionCommands,
    BenchmarkExecutionDeterminism, BenchmarkExecutionDownstreamProbe, BenchmarkExecutionImage,
    BenchmarkExecutionInput, BenchmarkExecutionInputKind, BenchmarkExecutionInterface,
    BenchmarkExecutionMeasurement, BenchmarkExecutionOutputs, BenchmarkExecutionParameterEvidence,
    BenchmarkExecutionResources, BenchmarkExecutionRetryPolicy, BenchmarkExecutionRuntimePolicy,
    BenchmarkExecutionSourceLock, BenchmarkExecutionSpec, BenchmarkOutputFileEvidence,
    BenchmarkParameterValue, BenchmarkRunIdentity, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
    BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1,
};

fn specification() -> BenchmarkExecutionSpec {
    BenchmarkExecutionSpec {
        schema_version: BENCHMARK_EXECUTION_SPEC_SCHEMA_V1.to_string(),
        identity: BenchmarkAttemptIdentity {
            campaign_id: "human-short-read-length-comparison".to_string(),
            run: BenchmarkRunIdentity {
                run_id: "run-2042".to_string(),
                domain: "fastq".to_string(),
                stage_id: "fastq.profile_read_lengths".to_string(),
            },
            attempt_id: "attempt-001".to_string(),
            tool_id: "seqkit".to_string(),
        },
        operator: "bijan".to_string(),
        prepared_at_utc: "2026-08-05T00:00:00Z".to_string(),
        source: BenchmarkExecutionSourceLock {
            repository_commit: "1".repeat(40),
            repository_path: "/study/worktrees/source".to_string(),
            contract_sha256: BTreeMap::from([
                ("campaign_registry".to_string(), "2".repeat(64)),
                ("container_catalog".to_string(), "3".repeat(64)),
                ("image_inventory".to_string(), "4".repeat(64)),
                ("stage_interface_registry".to_string(), "5".repeat(64)),
                ("corpus_catalog".to_string(), "6".repeat(64)),
                ("expectation_manifest".to_string(), "7".repeat(64)),
            ]),
        },
        image: BenchmarkExecutionImage {
            path: "/study/containers/seqkit.sif".to_string(),
            sha256: "8".repeat(64),
        },
        inputs: vec![BenchmarkExecutionInput {
            id: "fastq_err1".to_string(),
            kind: BenchmarkExecutionInputKind::Data,
            path: "/study/data/ERR1.fastq.gz".to_string(),
            mount_path: "/input/ERR1.fastq.gz".to_string(),
            sha256: "9".repeat(64),
        }],
        commands: BenchmarkExecutionCommands {
            version: vec!["seqkit".to_string(), "version".to_string()],
            help: vec!["seqkit".to_string(), "--help".to_string()],
            warmup: vec!["sh".to_string(), "-c".to_string(), "true".to_string()],
            measured: vec!["sh".to_string(), "-c".to_string(), "true".to_string()],
            negative: vec!["seqkit".to_string(), "missing.fastq".to_string()],
        },
        parameters: BenchmarkExecutionParameterEvidence {
            parameter_schema_version: "bijux.fastq.params.read_length_profile.v1".to_string(),
            defaults_provider: "bijux_dna_domain_fastq::read_length_profile_defaults".to_string(),
            overrides_path: "config-pipeline-parameters.yml".to_string(),
            overrides_sha256: "a".repeat(64),
            effective_path: "effective-parameters.json".to_string(),
            effective_sha256: "b".repeat(64),
        },
        interface: BenchmarkExecutionInterface {
            accepted_input_contracts: vec!["bijux.fastq.artifact.single_end.v1".to_string()],
            output_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
            normalized_result_contract: "benchmarks/contracts/schemas/result.json".to_string(),
            adapter: "bijux_dna_planner_fastq::profile_read_lengths".to_string(),
            downstream_probe: BenchmarkExecutionDownstreamProbe {
                stage_id: "benchmark.fastq.profile_read_lengths.compare".to_string(),
                input_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
                source: "benchmarks/execution/fastq_read_lengths.awk".to_string(),
                artifact: "repetitions/repetition-1.report-qc.json".to_string(),
            },
        },
        measurement: BenchmarkExecutionMeasurement {
            warmup_repetitions: 1,
            measured_repetitions: 3,
            resource_file_template: "repetitions/repetition-{repetition}.resource.tsv".to_string(),
            runtime_scope: "containerized_stage_end_to_end".to_string(),
        },
        outputs: BenchmarkExecutionOutputs {
            result_root: "/study/results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit/attempts/human-short-read-length-comparison/attempt-001".to_string(),
            mount_path: "/result".to_string(),
            required_files: vec!["repetitions/repetition-1.metrics.json".to_string()],
        },
        resources: BenchmarkExecutionResources {
            account: "lu2024-12-24".to_string(),
            partition: "compute".to_string(),
            cpus: 4,
            memory_mb: 8192,
            walltime: "01:00:00".to_string(),
            scratch_mb: 4096,
            retry: BenchmarkExecutionRetryPolicy {
                max_attempts: 1,
                backoff_seconds: 0,
                retryable_exit_codes: Vec::new(),
            },
        },
        runtime: BenchmarkExecutionRuntimePolicy {
            locale: "C".to_string(),
            timezone: "UTC".to_string(),
            runtime_network: false,
            threads: 4,
            random_seed: None,
            cache_policy: BenchmarkExecutionCachePolicy::Warm,
            determinism: BenchmarkExecutionDeterminism::Deterministic,
        },
        acceptance: acceptance(),
    }
}

fn acceptance() -> BenchmarkExecutionAcceptance {
    BenchmarkExecutionAcceptance {
        version_regex: "^[0-9]+[.][0-9]+".to_string(),
        negative_exit_nonzero: true,
        metrics: vec![BenchmarkAcceptanceMetric {
            id: "read_count".to_string(),
            path: "repetitions/repetition-1.metrics.json".to_string(),
            json_pointer: "/read_count".to_string(),
            operator: BenchmarkAcceptanceOperator::GreaterThan,
            expected: BenchmarkAcceptanceExpectedValue::Integer(0),
            unit: "reads".to_string(),
        }],
    }
}

#[test]
fn execution_spec_validates_and_round_trips() -> anyhow::Result<()> {
    let expected = specification();
    validate_execution_spec(&expected)?;
    let bytes = serde_json::to_vec(&expected)?;
    let actual: BenchmarkExecutionSpec = serde_json::from_slice(&bytes)?;
    assert_eq!(actual, expected);
    Ok(())
}

#[test]
fn execution_spec_rejects_idle_cpu_allocation() {
    let mut spec = specification();
    spec.runtime.threads = spec.resources.cpus - 1;

    assert!(validate_execution_spec(&spec)
        .is_err_and(|error| error.to_string().contains("runtime policy")));
}

#[test]
fn execution_spec_schema_freezes_version_and_source_identities() -> anyhow::Result<()> {
    let schema = serde_json::to_value(benchmark_execution_spec_schema())?;
    assert_eq!(
        schema.pointer("/properties/schema_version/const").and_then(serde_json::Value::as_str),
        Some(BENCHMARK_EXECUTION_SPEC_SCHEMA_V1)
    );
    assert_eq!(
        schema
            .pointer("/$defs/BenchmarkExecutionSourceLock/properties/repository_commit/pattern")
            .and_then(serde_json::Value::as_str),
        Some("^[0-9a-f]{40}$")
    );
    assert_eq!(
        schema
            .pointer("/$defs/BenchmarkExecutionSourceLock/properties/contract_sha256/additionalProperties/pattern")
            .and_then(serde_json::Value::as_str),
        Some("^[0-9a-f]{64}$")
    );
    Ok(())
}

#[test]
fn checked_execution_schemas_match_model() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        ("execution-spec.schema.json", serde_json::to_value(benchmark_execution_spec_schema())?),
        (
            "effective-parameters.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_effective_parameters_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "execution schema drifted: {name}");
    }
    Ok(())
}

#[test]
fn effective_parameters_round_trip_and_reject_schema_drift() -> anyhow::Result<()> {
    let parameters = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        parameter_schema_version: "bijux.fastq.params.validate_reads.v1".to_string(),
        defaults_provider: "bijux_dna_domain_fastq::params::defaults::validate_defaults"
            .to_string(),
        overrides: BTreeMap::from([("threads".to_string(), BenchmarkParameterValue::Integer(4))]),
        resolved_by_sample: BTreeMap::new(),
    };
    let value = serde_json::to_value(&parameters)?;
    assert_eq!(serde_json::from_value::<BenchmarkEffectiveParameters>(value.clone())?, parameters);
    let mut unsupported = value;
    unsupported["schema_version"] = serde_json::json!("bijux.benchmark.effective_parameters.v2");
    assert!(serde_json::from_value::<BenchmarkEffectiveParameters>(unsupported).is_err());
    Ok(())
}

#[test]
fn execution_admission_round_trips_and_rejects_schema_drift() -> anyhow::Result<()> {
    let admission = BenchmarkExecutionAdmission {
        schema_version: BENCHMARK_EXECUTION_ADMISSION_SCHEMA_V1.to_string(),
        identity: specification().identity,
        admitted_at_utc: "2026-08-06T00:00:00Z".to_string(),
        execution_spec_sha256: "1".repeat(64),
        completion_sha256: "2".repeat(64),
        output_manifest_sha256: "3".repeat(64),
        downstream_probe: BenchmarkOutputFileEvidence {
            path: "repetitions/repetition-1.report-qc.json".to_string(),
            sha256: "4".repeat(64),
            size_bytes: 1,
        },
        validation_files: Vec::new(),
        outputs: Vec::new(),
        metrics: Vec::new(),
    };
    let value = serde_json::to_value(&admission)?;
    assert_eq!(serde_json::from_value::<BenchmarkExecutionAdmission>(value.clone())?, admission);
    let mut unsupported = value;
    unsupported["schema_version"] = serde_json::json!("bijux.benchmark.execution_admission.v2");
    assert!(serde_json::from_value::<BenchmarkExecutionAdmission>(unsupported).is_err());
    Ok(())
}

#[test]
fn checked_report_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "stage-report.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_stage_report_schema())?,
        ),
        (
            "report-admission.schema.json",
            serde_json::to_value(bijux_dna_bench_model::admitted_benchmark_stage_report_schema())?,
        ),
        (
            "execution-admission.schema.json",
            serde_json::to_value(benchmark_execution_admission_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "report schema drifted: {name}");
    }
    Ok(())
}

#[test]
fn checked_core_record_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "corpus.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_corpus_manifest_schema())?,
        ),
        (
            "bundle.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_bundle_manifest_schema())?,
        ),
        (
            "observation.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_observation_schema())?,
        ),
        (
            "summary.schema.json",
            serde_json::to_value(bijux_dna_bench_model::benchmark_summary_schema())?,
        ),
        (
            "decision.schema.json",
            serde_json::to_value(bijux_dna_bench_model::gate_decision_schema())?,
        ),
    ];
    let schema_root = repository_root.join("benchmarks/contracts/schemas");
    for (name, generated) in schemas {
        let path = schema_root.join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "core record schema drifted: {name}");
    }
    Ok(())
}

#[test]
fn execution_spec_rejects_missing_source_lock_and_mutable_result_root() {
    let mut missing_lock = specification();
    missing_lock.source.contract_sha256.remove("image_inventory");
    assert!(validate_execution_spec(&missing_lock).is_err());

    let mut mutable_root = specification();
    mutable_root.outputs.result_root =
        "/study/results/benchmarks/run-2042-fastq-profile-read-lengths/seqkit".to_string();
    assert!(validate_execution_spec(&mutable_root).is_err());
}

#[test]
fn execution_spec_rejects_in_place_retry_and_undeclared_acceptance_output() {
    let mut retry = specification();
    retry.resources.retry.max_attempts = 2;
    assert!(validate_execution_spec(&retry).is_err());

    let mut undeclared = specification();
    undeclared.acceptance.metrics[0].path = "metrics.json".to_string();
    assert!(validate_execution_spec(&undeclared).is_err());
}

#[test]
fn execution_spec_rejects_seed_policy_without_exact_seed() {
    let mut seeded = specification();
    seeded.runtime.determinism = BenchmarkExecutionDeterminism::Seeded;
    assert!(validate_execution_spec(&seeded).is_err());

    seeded.runtime.random_seed = Some(42);
    assert!(validate_execution_spec(&seeded).is_ok());
}
