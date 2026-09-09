use anyhow::Context;
use bijux_dna_bench_model::{
    benchmark_local_pipeline_dag_config_schema,
    benchmark_local_pipeline_dag_metadata_config_schema,
    benchmark_local_pipeline_dag_stages_config_schema,
    validate_benchmark_local_pipeline_dag_config, BenchmarkLocalPipelineDagConfig,
    BenchmarkLocalPipelineDagMetadataConfig, BenchmarkLocalPipelineDagStagesConfig,
    BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1,
};
use serde_json::{json, Value};

fn config() -> Value {
    json!({
        "schema_version": BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1,
        "pipeline_id": "reads", "domain": "fastq", "summary": "Read validation",
        "default_corpus_id": "mini",
        "nodes": [{"node_id": "validate", "stage_id": "fastq.validate_reads",
            "readiness_kind": "smoke", "summary": "Validate reads",
            "external_inputs": ["reads"], "outputs": ["validated"]}]
    })
}

#[test]
fn configuration_preserves_optional_defaults_and_round_trips() -> anyhow::Result<()> {
    let value = config();
    let config: BenchmarkLocalPipelineDagConfig = serde_json::from_value(value)?;
    validate_benchmark_local_pipeline_dag_config(&config)?;
    assert!(config.project_sources.is_empty());
    assert!(config.reference_context.is_none());
    assert!(config.nodes[0].depends_on.is_empty());
    assert!(config.nodes[0].upstream_inputs.is_empty());
    let encoded = serde_json::to_value(&config)?;
    let decoded: BenchmarkLocalPipelineDagConfig = serde_json::from_value(encoded.clone())?;
    assert_eq!(serde_json::to_value(decoded)?, encoded);
    Ok(())
}

#[test]
fn configuration_retains_project_retry_default_and_override() -> anyhow::Result<()> {
    let mut value = config();
    value["project_sources"] = json!([
        {"project_id": "study", "metadata_url": "https://example.invalid/study"},
        {"project_id": "other", "metadata_url": "https://example.invalid/other", "retries": 5}
    ]);
    let config: BenchmarkLocalPipelineDagConfig = serde_json::from_value(value)?;
    assert_eq!(config.project_sources[0].retries, 3);
    assert_eq!(config.project_sources[1].retries, 5);
    Ok(())
}

#[test]
fn unknown_configuration_fields_are_rejected_at_each_boundary() -> anyhow::Result<()> {
    for path in ["", "/nodes/0", "/project_sources/0"] {
        let mut value = config();
        value["project_sources"] =
            json!([{ "project_id": "study", "metadata_url": "https://example.invalid/study" }]);
        value.pointer_mut(path).context("fixture path")?["unrecognized"] = json!(true);
        assert!(
            serde_json::from_value::<BenchmarkLocalPipelineDagConfig>(value).is_err(),
            "{path}"
        );
    }
    let mut metadata = config();
    metadata.as_object_mut().context("object")?.remove("nodes");
    metadata["unrecognized"] = json!(true);
    assert!(serde_json::from_value::<BenchmarkLocalPipelineDagMetadataConfig>(metadata).is_err());
    let stages = json!({"nodes": config()["nodes"], "unrecognized": true});
    assert!(serde_json::from_value::<BenchmarkLocalPipelineDagStagesConfig>(stages).is_err());
    Ok(())
}

#[test]
fn unknown_domain_and_readiness_values_are_rejected() -> anyhow::Result<()> {
    for path in ["/domain", "/nodes/0/readiness_kind"] {
        let mut value = config();
        *value.pointer_mut(path).context("fixture path")? = json!("unrecognized");
        assert!(serde_json::from_value::<BenchmarkLocalPipelineDagConfig>(value).is_err());
    }
    Ok(())
}

#[test]
fn configuration_identity_and_node_envelope_fail_closed() -> anyhow::Result<()> {
    for field in ["pipeline_id", "summary", "default_corpus_id"] {
        let mut value = config();
        value[field] = json!(" \t");
        let config = serde_json::from_value(value)?;
        assert_eq!(
            validate_benchmark_local_pipeline_dag_config(&config)
                .err()
                .context("blank identity must fail")?
                .to_string(),
            format!("local pipeline DAG must declare a non-empty `{field}`")
        );
    }
    let mut value = config();
    value["schema_version"] = json!("bijux.bench.local_pipeline_dag.v999");
    assert!(validate_benchmark_local_pipeline_dag_config(&serde_json::from_value(value)?).is_err());
    let mut value = config();
    value["nodes"] = json!([]);
    assert_eq!(
        validate_benchmark_local_pipeline_dag_config(&serde_json::from_value(value)?)
            .err()
            .context("empty nodes must fail")?
            .to_string(),
        "local pipeline DAG must declare at least one `nodes` entry"
    );
    Ok(())
}

#[test]
fn input_schemas_preserve_version_and_strict_transport_shapes() -> anyhow::Result<()> {
    for schema in [
        benchmark_local_pipeline_dag_config_schema(),
        benchmark_local_pipeline_dag_metadata_config_schema(),
    ] {
        let value = serde_json::to_value(schema)?;
        assert_eq!(
            value["properties"]["schema_version"]["const"],
            BENCHMARK_LOCAL_PIPELINE_DAG_CONFIG_SCHEMA_V1
        );
        assert_eq!(value["additionalProperties"], false);
    }
    let value = serde_json::to_value(benchmark_local_pipeline_dag_stages_config_schema())?;
    assert_eq!(value["required"], json!(["nodes"]));
    assert_eq!(value["additionalProperties"], false);
    Ok(())
}

#[test]
fn governed_toml_pipelines_decode_through_the_model_owner() -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/configs/pipelines/local");
    let mut count = 0;
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("toml") {
            continue;
        }
        let config: BenchmarkLocalPipelineDagConfig =
            toml::from_str(&std::fs::read_to_string(&path)?)?;
        validate_benchmark_local_pipeline_dag_config(&config)?;
        count += 1;
    }
    assert!(count > 0, "governed input set is empty");
    Ok(())
}
