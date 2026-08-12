use bijux_dna_bench_model::{
    benchmark_local_pipeline_dag_validation_schema,
    benchmark_local_pipeline_dag_validation_set_schema,
    validate_benchmark_local_pipeline_dag_validation,
    validate_benchmark_local_pipeline_dag_validation_set, BenchmarkLocalPipelineDagValidationNode,
    BenchmarkLocalPipelineDagValidationProfile, BenchmarkLocalPipelineDagValidationReport,
    BenchmarkLocalPipelineDagValidationSetReport,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
    BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_V1,
};

fn report() -> BenchmarkLocalPipelineDagValidationReport {
    BenchmarkLocalPipelineDagValidationReport {
        schema_version: BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1.to_string(),
        config_path: "benchmarks/configs/pipelines/local/example.toml".to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/example.json"
            .to_string(),
        pipeline_id: "example".to_string(),
        domain: "cross".to_string(),
        default_corpus_id: "corpus-01-mini".to_string(),
        summary: "governed example pipeline".to_string(),
        reference_context: None,
        execution_context: None,
        project_source_count: 0,
        project_sources: Vec::new(),
        ancient_dna_policy: None,
        reference_assets: None,
        variant_assets: None,
        node_count: 2,
        edge_count: 1,
        acyclic: true,
        valid: true,
        topological_order: vec!["prepare".to_string(), "consume".to_string()],
        validation_profiles: vec![BenchmarkLocalPipelineDagValidationProfile {
            profile_id: "example_profile".to_string(),
            check_count: 1,
            checks: vec!["validated downstream handoff".to_string()],
        }],
        nodes: vec![
            BenchmarkLocalPipelineDagValidationNode {
                node_id: "prepare".to_string(),
                stage_id: "fastq.validate_reads".to_string(),
                readiness_kind: "dry_or_smoke".to_string(),
                dependency_count: 0,
                external_input_count: 1,
                upstream_input_count: 0,
                output_count: 1,
                depends_on: Vec::new(),
                external_inputs: vec!["reads".to_string()],
                upstream_inputs: Vec::new(),
                outputs: vec!["validated_reads".to_string()],
            },
            BenchmarkLocalPipelineDagValidationNode {
                node_id: "consume".to_string(),
                stage_id: "fastq.profile_reads".to_string(),
                readiness_kind: "dry_or_smoke".to_string(),
                dependency_count: 1,
                external_input_count: 0,
                upstream_input_count: 1,
                output_count: 1,
                depends_on: vec!["prepare".to_string()],
                external_inputs: Vec::new(),
                upstream_inputs: vec!["validated_reads".to_string()],
                outputs: vec!["profile".to_string()],
            },
        ],
    }
}

fn set_report() -> BenchmarkLocalPipelineDagValidationSetReport {
    let first = report();
    let mut second = first.clone();
    second.pipeline_id = "example-z".to_string();
    second.config_path = "benchmarks/configs/pipelines/local/example-z.toml".to_string();
    second.output_path =
        "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/example-z.json".to_string();
    BenchmarkLocalPipelineDagValidationSetReport {
        schema_version: BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SET_SCHEMA_V1.to_string(),
        benchmark_root: "benchmarks".to_string(),
        config_root: "benchmarks/configs/pipelines/local".to_string(),
        output_path:
            "artifacts/benchmark-smoke/readiness/local-ready/pipeline-dag/all-benchmark-pipelines.json"
                .to_string(),
        pipeline_count: 2,
        valid_pipeline_count: 2,
        all_valid: true,
        pipelines: vec![first, second],
    }
}

#[test]
fn local_pipeline_dag_validation_round_trips_and_admits() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_pipeline_dag_validation(&report)?;
    let round_trip: BenchmarkLocalPipelineDagValidationReport =
        serde_json::from_slice(&serde_json::to_vec(&report)?)?;
    assert_eq!(round_trip, report);
    Ok(())
}

#[test]
fn local_pipeline_dag_validation_rejects_aggregate_and_topology_drift() {
    let mut aggregate_drift = report();
    aggregate_drift.edge_count = 0;
    assert!(validate_benchmark_local_pipeline_dag_validation(&aggregate_drift).is_err());

    let mut topology_drift = report();
    topology_drift.topological_order.swap(0, 1);
    assert!(validate_benchmark_local_pipeline_dag_validation(&topology_drift).is_err());

    let mut unknown_dependency = report();
    unknown_dependency.nodes[1].depends_on[0] = "missing".to_string();
    assert!(validate_benchmark_local_pipeline_dag_validation(&unknown_dependency).is_err());

    let mut missing_producer = report();
    missing_producer.nodes[1].upstream_inputs[0] = "missing_output".to_string();
    assert!(validate_benchmark_local_pipeline_dag_validation(&missing_producer).is_err());

    let mut readiness_drift = report();
    readiness_drift.nodes[0].readiness_kind = "unknown".to_string();
    assert!(validate_benchmark_local_pipeline_dag_validation(&readiness_drift).is_err());
}

#[test]
fn local_pipeline_dag_validation_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-pipeline-dag-validation.schema.json");
    let generated = serde_json::to_value(benchmark_local_pipeline_dag_validation_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local pipeline DAG validation schema drifted");
    Ok(())
}

#[test]
fn local_pipeline_dag_validation_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(report())?;
    value["nodes"][0]["legacy_state"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkLocalPipelineDagValidationReport>(value).is_err());

    let mut missing_retry = serde_json::to_value(report())?;
    missing_retry["project_source_count"] = serde_json::json!(1);
    missing_retry["project_sources"] = serde_json::json!([{
        "project_id": "ena-study",
        "metadata_url": "https://example.invalid/study"
    }]);
    assert!(
        serde_json::from_value::<BenchmarkLocalPipelineDagValidationReport>(missing_retry).is_err()
    );
    Ok(())
}

#[test]
fn local_pipeline_dag_validation_set_round_trips_and_admits() -> anyhow::Result<()> {
    let report = set_report();
    validate_benchmark_local_pipeline_dag_validation_set(&report)?;
    let round_trip: BenchmarkLocalPipelineDagValidationSetReport =
        serde_json::from_slice(&serde_json::to_vec(&report)?)?;
    assert_eq!(round_trip, report);
    Ok(())
}

#[test]
fn local_pipeline_dag_validation_set_rejects_aggregate_order_and_child_drift() {
    let mut aggregate_drift = set_report();
    aggregate_drift.valid_pipeline_count = 1;
    assert!(validate_benchmark_local_pipeline_dag_validation_set(&aggregate_drift).is_err());

    let mut order_drift = set_report();
    order_drift.pipelines.swap(0, 1);
    assert!(validate_benchmark_local_pipeline_dag_validation_set(&order_drift).is_err());

    let mut child_drift = set_report();
    child_drift.pipelines[0].valid = false;
    child_drift.valid_pipeline_count = 1;
    child_drift.all_valid = false;
    assert!(validate_benchmark_local_pipeline_dag_validation_set(&child_drift).is_err());
}

#[test]
fn local_pipeline_dag_validation_set_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-pipeline-dag-validation-set.schema.json");
    let generated = serde_json::to_value(benchmark_local_pipeline_dag_validation_set_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local pipeline DAG validation-set schema drifted");
    Ok(())
}

#[test]
fn local_pipeline_dag_validation_set_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(set_report())?;
    value["legacy_status"] = serde_json::json!("ready");
    assert!(serde_json::from_value::<BenchmarkLocalPipelineDagValidationSetReport>(value).is_err());
    Ok(())
}
