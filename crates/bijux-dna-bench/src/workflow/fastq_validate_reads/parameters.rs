//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for FASTQ validation attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{
    BenchmarkExecutionParameterEvidence, BenchmarkParameterValue, BenchmarkPipelineParameterConfig,
};
use serde::Serialize;

use super::super::fastq_profile_reads::FastqCampaignSample;
use super::planner_contract::load_validation_planner_contract;

const STAGE_ID: &str = "fastq.validate_reads";
const PARAMETER_SCHEMA: &str = "bijux.fastq.params.validate_reads.v1";
const DEFAULTS_PROVIDER: &str = "bijux_dna_domain_fastq::params::defaults::validate_defaults";

#[derive(Serialize)]
struct EffectiveParameters {
    schema_version: &'static str,
    stage_id: &'static str,
    parameter_schema_version: &'static str,
    defaults_provider: &'static str,
    overrides: BTreeMap<&'static str, BenchmarkParameterValue>,
    resolved_by_sample: BTreeMap<String, serde_json::Value>,
}

pub(super) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    samples: &[FastqCampaignSample],
) -> Result<BenchmarkExecutionParameterEvidence> {
    let source = repository_root.join("benchmarks/contracts/config-pipeline-parameters.yml");
    let source_bytes = bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)
        .with_context(|| format!("read pipeline parameters {}", source.display()))?;
    let config: BenchmarkPipelineParameterConfig = serde_yaml::from_slice(&source_bytes)
        .with_context(|| format!("parse pipeline parameters {}", source.display()))?;
    validate_pipeline_parameter_config(&config).context("validate pipeline parameter config")?;
    let stage = config.stages.get(STAGE_ID).context("parameter config lacks FASTQ validation")?;
    let expected_overrides = BTreeMap::from([
        ("threads".to_string(), BenchmarkParameterValue::Integer(4)),
        ("validation_mode".to_string(), BenchmarkParameterValue::Text("strict".to_string())),
        (
            "pair_sync_policy".to_string(),
            BenchmarkParameterValue::Text("layout_derived".to_string()),
        ),
    ]);
    if stage.parameter_schema_version != PARAMETER_SCHEMA
        || stage.defaults_provider != DEFAULTS_PROVIDER
        || stage.overrides != expected_overrides
    {
        bail!("FASTQ validation parameter authority differs from executable policy");
    }
    let planner = load_validation_planner_contract(repository_root)?;
    if planner.parameter_schema_version != PARAMETER_SCHEMA
        || planner.default_provider != DEFAULTS_PROVIDER
    {
        bail!("FASTQ validation planner parameter authority differs from pipeline policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &source_bytes)?;
    let effective = EffectiveParameters {
        schema_version: "bijux.benchmark.effective_parameters.v1",
        stage_id: STAGE_ID,
        parameter_schema_version: PARAMETER_SCHEMA,
        defaults_provider: DEFAULTS_PROVIDER,
        overrides: BTreeMap::from([
            ("threads", BenchmarkParameterValue::Integer(4)),
            ("validation_mode", BenchmarkParameterValue::Text("strict".to_string())),
            ("pair_sync_policy", BenchmarkParameterValue::Text("layout_derived".to_string())),
        ]),
        resolved_by_sample: samples
            .iter()
            .map(|sample| {
                let mut parameters = if sample.input_layout == "paired_end" {
                    planner.domain_defaults.paired_end.clone()
                } else {
                    planner.domain_defaults.single_end.clone()
                };
                let object = parameters
                    .as_object_mut()
                    .context("planner validation defaults are not an object")?;
                object.insert("threads".to_string(), serde_json::json!(4));
                Ok((sample.sample_id.clone(), parameters))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    };
    let mut effective_bytes =
        serde_json::to_vec_pretty(&effective).context("serialize FASTQ validation parameters")?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: PARAMETER_SCHEMA.to_string(),
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides_path: "config-pipeline-parameters.yml".to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&source_bytes),
        effective_path: "effective-parameters.json".to_string(),
        effective_sha256: bijux_dna_infra::hash_bytes_sha256(&effective_bytes),
    })
}

fn ensure_bytes(path: &Path, bytes: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("parameter evidence is not a regular non-symlink file");
            }
            if bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)? != bytes {
                bail!("parameter evidence conflicts with immutable attempt material");
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, bytes)
                .with_context(|| format!("publish parameter evidence {}", path.display()))
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspect parameter evidence {}", path.display()))
        }
    }
}
