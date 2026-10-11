//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for BAM observational campaigns.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{
    BenchmarkEffectiveParameterValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BenchmarkPipelineParameterConfig,
    BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

pub(in crate::workflow) struct BamParameterPolicy<'a> {
    pub(in crate::workflow) stage_id: &'a str,
    pub(in crate::workflow) schema_version: &'a str,
    pub(in crate::workflow) defaults_provider: &'a str,
}

pub(in crate::workflow) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    sample_id: &str,
    policy: BamParameterPolicy<'_>,
    resolved: BTreeMap<String, BenchmarkEffectiveParameterValue>,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let source = repository_root.join("benchmarks/contracts/config-pipeline-parameters.yml");
    let source_bytes = bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)
        .with_context(|| format!("read pipeline parameters {}", source.display()))?;
    let config: BenchmarkPipelineParameterConfig = serde_yaml::from_slice(&source_bytes)
        .with_context(|| format!("parse pipeline parameters {}", source.display()))?;
    validate_pipeline_parameter_config(&config).context("validate pipeline parameter config")?;
    let stage = config
        .stages
        .get(policy.stage_id)
        .with_context(|| format!("parameter config lacks {}", policy.stage_id))?;
    if stage.parameter_schema_version != policy.schema_version
        || stage.defaults_provider != policy.defaults_provider
        || !stage.overrides.is_empty()
    {
        bail!("BAM parameter authority differs from executable policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: policy.stage_id.to_string(),
        parameter_schema_version: policy.schema_version.to_string(),
        defaults_provider: policy.defaults_provider.to_string(),
        overrides: BTreeMap::new(),
        resolved_by_sample: BTreeMap::from([(sample_id.to_string(), resolved)]),
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: policy.schema_version.to_string(),
        defaults_provider: policy.defaults_provider.to_string(),
        overrides_path: "config-pipeline-parameters.yml".to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&source_bytes),
        effective_path: "effective-parameters.json".to_string(),
        effective_sha256: bijux_dna_infra::hash_bytes_sha256(&effective_bytes),
    })
}

fn ensure_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("parameter evidence is not a regular non-symlink file: {}", path.display());
            }
            if std::fs::read(path)? != expected {
                bail!("parameter evidence conflicts with immutable bytes: {}", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            Ok(bijux_dna_infra::atomic_create_bytes(path, expected)?)
        }
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}
