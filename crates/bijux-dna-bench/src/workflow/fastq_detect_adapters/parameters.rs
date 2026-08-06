//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for FASTQ adapter-inspection attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BenchmarkParameterValue, BenchmarkPipelineParameterConfig,
    BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

use super::super::fastq_profile_reads::FastqCampaignSample;

const STAGE_ID: &str = "fastq.detect_adapters";
const PARAMETER_SCHEMA: &str = "bijux.fastq.params.detect_adapters.v1";
const DEFAULTS_PROVIDER: &str =
    "bijux_dna_domain_fastq::params::defaults::detect_adapters_defaults";

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
    let stage =
        config.stages.get(STAGE_ID).context("parameter config lacks FASTQ detect adapters")?;
    let expected_overrides =
        BTreeMap::from([("threads".to_string(), BenchmarkParameterValue::Integer(4))]);
    if stage.parameter_schema_version != PARAMETER_SCHEMA
        || stage.defaults_provider != DEFAULTS_PROVIDER
        || stage.overrides != expected_overrides
    {
        bail!("FASTQ detect-adapters parameter authority differs from executable policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: STAGE_ID.to_string(),
        parameter_schema_version: PARAMETER_SCHEMA.to_string(),
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides: expected_overrides,
        resolved_by_sample: samples
            .iter()
            .map(|sample| {
                Ok((
                    sample.sample_id.clone(),
                    BTreeMap::from([
                        (
                            "schema_version".to_string(),
                            EffectiveValue::Text(PARAMETER_SCHEMA.to_string()),
                        ),
                        (
                            "paired_mode".to_string(),
                            EffectiveValue::Text(sample.input_layout.to_string()),
                        ),
                        ("threads".to_string(), EffectiveValue::Integer(4)),
                        ("sample_reads".to_string(), EffectiveValue::Null),
                        (
                            "inspection_mode".to_string(),
                            EffectiveValue::Text("evidence_only".to_string()),
                        ),
                        ("report_only".to_string(), EffectiveValue::Boolean(true)),
                        ("evidence_engine".to_string(), EffectiveValue::Text("fastqc".to_string())),
                        (
                            "evidence_scope".to_string(),
                            EffectiveValue::Text("full_input".to_string()),
                        ),
                        (
                            "evidence_format".to_string(),
                            EffectiveValue::Text("fastqc_summary".to_string()),
                        ),
                        (
                            "evidence_artifact_id".to_string(),
                            EffectiveValue::Text("report_json".to_string()),
                        ),
                    ]),
                ))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)
        .context("serialize FASTQ detect-adapters parameters")?;
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
