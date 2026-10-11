use bijux_dna_bench_model::{
    benchmark_attempt_identity_schema, benchmark_attempt_identity_sha256,
    benchmark_case_definition_schema, benchmark_run_definition_schema,
    benchmark_run_definition_sha256, validate_attempt_identity, validate_benchmark_case_definition,
    validate_benchmark_run_definition, validate_benchmark_run_definition_revision,
    BenchmarkAttemptIdentity, BenchmarkCaseDefinition, BenchmarkCaseInputLock,
    BenchmarkCaseQualificationReceipt, BenchmarkContractLock, BenchmarkCorpusRole,
    BenchmarkRunDefinition, BenchmarkRunIdentity, BenchmarkRunState,
    BENCHMARK_CASE_DEFINITION_SCHEMA_V1, BENCHMARK_RUN_DEFINITION_SCHEMA_V1,
};

fn run_identity() -> BenchmarkRunIdentity {
    BenchmarkRunIdentity {
        run_id: "run-2042".to_string(),
        domain: "fastq".to_string(),
        stage_id: "fastq.profile_read_lengths".to_string(),
    }
}

fn run_definition() -> BenchmarkRunDefinition {
    BenchmarkRunDefinition {
        schema_version: BENCHMARK_RUN_DEFINITION_SCHEMA_V1.to_string(),
        identity: run_identity(),
        protocol_version: "1.0.0".to_string(),
        title: "FASTQ read-length profiling".to_string(),
        purpose: "Compare retained read-length profiling tools on governed real data.".to_string(),
        owner: "bijux-dna-bench".to_string(),
        state: BenchmarkRunState::Active,
        state_reason: None,
        tool_ids: vec!["seqfu".to_string(), "seqkit".to_string()],
        contracts: vec![BenchmarkContractLock {
            contract_id: "stage-interface".to_string(),
            repository_path: "benchmarks/contracts/stage-interfaces.toml".to_string(),
            sha256: "a".repeat(64),
        }],
        corpus_roles: vec![BenchmarkCorpusRole {
            role: "evaluation".to_string(),
            corpus_ids: vec!["corpus-real-fastq".to_string()],
        }],
        acceptance_policy_id: "comparative-stage-v1".to_string(),
        citation_key: "run-2042-fastq-profile-read-lengths".to_string(),
        supersedes_run_ids: Vec::new(),
    }
}

fn attempt_identity(attempt_id: &str) -> BenchmarkAttemptIdentity {
    BenchmarkAttemptIdentity {
        campaign_id: "all-domains-20260806".to_string(),
        run: run_identity(),
        attempt_id: attempt_id.to_string(),
        tool_id: "seqkit".to_string(),
    }
}

fn case_definition() -> BenchmarkCaseDefinition {
    BenchmarkCaseDefinition {
        schema_version: BENCHMARK_CASE_DEFINITION_SCHEMA_V1.to_string(),
        case_id: "run-2042-seqkit-real-fastq".to_string(),
        campaign_id: "all-domains-20260806".to_string(),
        run: run_identity(),
        tool_id: "seqkit".to_string(),
        repository_commit: "a".repeat(40),
        corpus_snapshot_sha256: "b".repeat(64),
        parameter_lock_sha256: "c".repeat(64),
        execution_spec_sha256: "d".repeat(64),
        inputs: vec![BenchmarkCaseInputLock {
            role: "reads".to_string(),
            asset_id: "corpus-real-fastq".to_string(),
            sha256: "e".repeat(64),
        }],
        qualification: BenchmarkCaseQualificationReceipt {
            tool_id: "seqkit".to_string(),
            uri: "study://evidence/containers/run-1141-seqkit/qualification.json".to_string(),
            receipt_sha256: "f".repeat(64),
            image_sha256: "1".repeat(64),
        },
        warmup_repetitions: 1,
        measured_repetitions: 3,
    }
}

#[test]
fn protocol_records_validate_and_round_trip() -> anyhow::Result<()> {
    let run = run_definition();
    validate_benchmark_run_definition(&run)?;
    let actual: BenchmarkRunDefinition = serde_json::from_slice(&serde_json::to_vec(&run)?)?;
    assert_eq!(actual, run);

    let case = case_definition();
    validate_benchmark_case_definition(&case)?;
    let actual: BenchmarkCaseDefinition = serde_json::from_slice(&serde_json::to_vec(&case)?)?;
    assert_eq!(actual, case);
    Ok(())
}

#[test]
fn protocol_records_reject_unknown_fields_and_states() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(run_definition())?;
    value["unknown"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkRunDefinition>(value).is_err());

    let mut value = serde_json::to_value(run_definition())?;
    value["state"] = serde_json::json!("invented");
    assert!(serde_json::from_value::<BenchmarkRunDefinition>(value).is_err());
    Ok(())
}

#[test]
fn protocol_validation_rejects_ambiguous_or_unqualified_records() {
    let mut run = run_definition();
    run.tool_ids.reverse();
    assert!(validate_benchmark_run_definition(&run).is_err());

    let mut run = run_definition();
    run.citation_key = "not/a/key".to_string();
    assert!(validate_benchmark_run_definition(&run).is_err());

    let mut run = run_definition();
    run.protocol_version = "01.0.0".to_string();
    assert!(validate_benchmark_run_definition(&run).is_err());

    let mut reserved = run_definition();
    reserved.state = BenchmarkRunState::Reserved;
    reserved.state_reason = Some("reserved for reviewed protocol allocation".to_string());
    assert!(validate_benchmark_run_definition(&reserved).is_err());

    let mut case = case_definition();
    case.qualification.tool_id = "seqfu".to_string();
    assert!(validate_benchmark_case_definition(&case).is_err());

    let mut case = case_definition();
    case.measured_repetitions = 1;
    assert!(validate_benchmark_case_definition(&case).is_err());

    let mut case = case_definition();
    case.inputs.push(BenchmarkCaseInputLock {
        role: "alignment".to_string(),
        asset_id: "reference-equus".to_string(),
        sha256: "2".repeat(64),
    });
    assert!(validate_benchmark_case_definition(&case).is_err());
}

#[test]
fn protocol_revisions_require_versions_or_explicit_supersession() -> anyhow::Result<()> {
    let previous = run_definition();
    let mut same_run_revision = previous.clone();
    same_run_revision.protocol_version = "1.1.0".to_string();
    validate_benchmark_run_definition_revision(&previous, &same_run_revision)?;

    let mut unversioned = previous.clone();
    unversioned.purpose = "Changed protocol without an identity revision.".to_string();
    assert!(validate_benchmark_run_definition_revision(&previous, &unversioned).is_err());

    let mut replacement = previous.clone();
    replacement.identity.run_id = "run-2043".to_string();
    replacement.supersedes_run_ids = vec!["run-2042".to_string()];
    validate_benchmark_run_definition_revision(&previous, &replacement)?;
    Ok(())
}

#[test]
fn run_and_attempt_identity_hashes_are_deterministic_and_scheduler_independent(
) -> anyhow::Result<()> {
    let run = run_definition();
    assert_eq!(benchmark_run_definition_sha256(&run)?, benchmark_run_definition_sha256(&run)?);

    let first = attempt_identity("attempt-001");
    let retry = attempt_identity("attempt-002");
    validate_attempt_identity(&first)?;
    validate_attempt_identity(&retry)?;
    assert_eq!(first.run, retry.run);
    assert_ne!(
        benchmark_attempt_identity_sha256(&first)?,
        benchmark_attempt_identity_sha256(&retry)?
    );

    let schema = serde_json::to_value(benchmark_attempt_identity_schema())?;
    assert!(!schema.to_string().contains("slurm"));
    Ok(())
}

#[test]
fn checked_protocol_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            "attempt-identity.schema.json",
            serde_json::to_value(benchmark_attempt_identity_schema())?,
        ),
        ("run-definition.schema.json", serde_json::to_value(benchmark_run_definition_schema())?),
        ("case-definition.schema.json", serde_json::to_value(benchmark_case_definition_schema())?),
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
        assert_eq!(checked, generated, "protocol schema drifted: {name}");
    }
    Ok(())
}
