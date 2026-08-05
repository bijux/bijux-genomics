use std::collections::BTreeMap;

use bijux_dna_bench_model::contract::{
    validate_defined_interface_campaign_alignment, validate_interface_parameter_alignment,
    validate_pipeline_parameter_config, validate_stage_interface_catalog,
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_STAGE_INTERFACES_SCHEMA_V1,
};
use bijux_dna_bench_model::{
    BenchmarkCachePolicy, BenchmarkEvidenceTier, BenchmarkExecutionRuntime,
    BenchmarkExecutionScheduler, BenchmarkParameterValue, BenchmarkPipelineParameterConfig,
    BenchmarkStageCampaignRegistry, BenchmarkStageCampaignSpec, BenchmarkStageDownstreamProbe,
    BenchmarkStageInterfaceCatalog, BenchmarkStageInterfaceSpec, BenchmarkStageParameterConfig,
};

fn interface_catalog() -> BenchmarkStageInterfaceCatalog {
    BenchmarkStageInterfaceCatalog {
        schema_version: BENCHMARK_STAGE_INTERFACES_SCHEMA_V1.to_string(),
        interfaces: vec![BenchmarkStageInterfaceSpec {
            stage_id: "fastq.profile_read_lengths".to_string(),
            accepted_input_contracts: vec!["bijux.fastq.artifact.single_end.v1".to_string()],
            output_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
            normalized_result_contract:
                "benchmarks/contracts/schemas/fastq-profile-read-lengths-result.schema.json"
                    .to_string(),
            parameter_schema_version: "bijux.fastq.params.read_length_profile.v1".to_string(),
            default_provider:
                "bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults".to_string(),
            parameter_config: "benchmarks/contracts/config-pipeline-parameters.yml".to_string(),
            planner_contract: Some(
                "benchmarks/contracts/planner/fastq-profile-read-lengths.planner-contract.json"
                    .to_string(),
            ),
            operator_override_keys: vec!["threads".to_string(), "histogram_bins".to_string()],
            input_derived_parameters: vec!["paired_mode".to_string()],
            immutable_parameters: vec!["schema_version".to_string()],
            effective_parameters_artifact: "effective-parameters.json".to_string(),
            tool_ids: vec!["seqfu".to_string(), "seqkit".to_string()],
            provenance_sources: vec!["domain/fastq/stages/profile_read_lengths.yaml".to_string()],
            adapters: BTreeMap::from([
                ("seqfu".to_string(), "planner::profile_read_lengths".to_string()),
                ("seqkit".to_string(), "planner::profile_read_lengths".to_string()),
            ]),
            parameter_projections: BTreeMap::from([(
                "seqkit".to_string(),
                BTreeMap::from([("threads".to_string(), "seqkit --threads".to_string())]),
            )]),
            downstream_probe: BenchmarkStageDownstreamProbe {
                consumer_stage_id: "benchmark.fastq.profile_read_lengths.compare".to_string(),
                input_contract: "bijux.benchmark.fastq.profile_read_lengths.bundle.v1".to_string(),
                source: "benchmarks/execution/fastq_read_lengths.awk".to_string(),
                assertions: vec!["all records preserve exact length".to_string()],
            },
        }],
    }
}

fn parameter_config() -> BenchmarkPipelineParameterConfig {
    BenchmarkPipelineParameterConfig {
        schema_version: BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1.to_string(),
        profile: "generic-fastq".to_string(),
        stages: BTreeMap::from([(
            "fastq.profile_read_lengths".to_string(),
            BenchmarkStageParameterConfig {
                parameter_schema_version: "bijux.fastq.params.read_length_profile.v1".to_string(),
                defaults_provider:
                    "bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults"
                        .to_string(),
                overrides: BTreeMap::from([
                    ("threads".to_string(), BenchmarkParameterValue::Integer(4)),
                    ("histogram_bins".to_string(), BenchmarkParameterValue::Integer(100)),
                ]),
            },
        )]),
    }
}

fn campaign_registry() -> BenchmarkStageCampaignRegistry {
    BenchmarkStageCampaignRegistry {
        schema_version: "bijux.benchmark.stage_campaigns.v1".to_string(),
        campaigns: vec![BenchmarkStageCampaignSpec {
            run_id: "run-2042".to_string(),
            slug: "fastq-profile-read-lengths".to_string(),
            domain: "fastq".to_string(),
            stage_id: "fastq.profile_read_lengths".to_string(),
            evidence_tier: BenchmarkEvidenceTier::Comparative,
            profile_id: "fastq-observational-qc".to_string(),
            corpus_ids: vec!["human-short-read-diversity".to_string()],
            scientific_question: "Are read lengths preserved?".to_string(),
            comparison_scope: "exact cohort".to_string(),
            tool_ids: vec!["seqfu".to_string(), "seqkit".to_string()],
            corpus_catalog_ids: vec!["human-short-read-diversity".to_string()],
            reference_catalog_ids: Vec::new(),
            database_catalog_ids: Vec::new(),
            input_size_keys: vec!["fastq_input_bytes".to_string()],
            result_artifact: "read-lengths.json".to_string(),
            runtime: BenchmarkExecutionRuntime::Apptainer,
            scheduler: BenchmarkExecutionScheduler::Slurm,
            container_only: true,
            threads: None,
            threads_by_tool: BTreeMap::new(),
            warmup_repetitions: 1,
            measured_repetitions: 3,
            cache_policy: BenchmarkCachePolicy::Warm,
            result_root_template: "results/benchmarks/{run_id}-{slug}/{tool_id}".to_string(),
            accuracy_claim: "exact length".to_string(),
            interpretation: "exact agreement required".to_string(),
            performance_metrics: vec!["wall_seconds".to_string()],
            scientific_metrics: vec!["record_count".to_string()],
            comparison_metrics: Vec::new(),
            limitations: vec!["operational cohort".to_string()],
        }],
    }
}

#[test]
fn stage_interface_contracts_accept_aligned_catalog() -> anyhow::Result<()> {
    let catalog = interface_catalog();
    let parameters = parameter_config();
    validate_stage_interface_catalog(&catalog)?;
    validate_pipeline_parameter_config(&parameters)?;
    validate_interface_parameter_alignment(&catalog.interfaces[0], &parameters)?;
    validate_defined_interface_campaign_alignment(&catalog, &campaign_registry())?;
    Ok(())
}

#[test]
fn stage_interface_contracts_allow_no_input_derived_parameters() -> anyhow::Result<()> {
    let mut catalog = interface_catalog();
    catalog.interfaces[0].input_derived_parameters.clear();

    validate_stage_interface_catalog(&catalog)?;
    Ok(())
}

#[test]
fn stage_interface_contracts_reject_incomplete_adapter_coverage() {
    let mut catalog = interface_catalog();
    catalog.interfaces[0].adapters.remove("seqkit");

    let error = validate_stage_interface_catalog(&catalog).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("exact tool set")));
}

#[test]
fn stage_interface_contracts_reject_parameter_ownership_overlap() {
    let mut catalog = interface_catalog();
    catalog.interfaces[0].input_derived_parameters.push("threads".to_string());

    let error = validate_stage_interface_catalog(&catalog).err().map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("ownership overlaps")));
}

#[test]
fn checked_interface_schemas_match_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let schemas = [
        (
            repository_root.join("benchmarks/contracts/schemas/stage-interfaces.schema.json"),
            serde_json::to_value(bijux_dna_bench_model::benchmark_stage_interface_catalog_schema())?,
        ),
        (
            repository_root.join("benchmarks/contracts/schemas/pipeline-parameters.schema.json"),
            serde_json::to_value(
                bijux_dna_bench_model::benchmark_pipeline_parameter_config_schema(),
            )?,
        ),
    ];
    for (path, generated) in schemas {
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(checked, generated, "interface schema drifted: {}", path.display());
    }
    Ok(())
}

#[test]
fn stage_interface_contracts_reject_campaign_tool_drift() {
    let catalog = interface_catalog();
    let mut registry = campaign_registry();
    registry.campaigns[0].tool_ids.pop();

    let error = validate_defined_interface_campaign_alignment(&catalog, &registry)
        .err()
        .map(|error| error.to_string());
    assert!(error.as_deref().is_some_and(|message| message.contains("tool drift")));
}
