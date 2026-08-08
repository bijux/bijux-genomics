//! Owner: bijux-dna-bench
//! Repository-backed parameter authority for benchmark execution evidence.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{BenchmarkPipelineParameterConfig, BenchmarkStageParameterConfig};
use serde::Deserialize;

const PIPELINE_PARAMETERS_PATH: &str = "benchmarks/contracts/config-pipeline-parameters.yml";
const PARAMETER_REGISTRY_PATH: &str = "configs/ci/params/param_registry.toml";
const PARAMETER_REGISTRY_SCHEMA: &str = "bijux.param_registry.v1";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterRegistry {
    schema_version: String,
    source: String,
    entries: Vec<ParameterRegistryEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ParameterRegistryEntry {
    stage_id: String,
    param_type_id: String,
    schema_version: String,
}

pub(in crate::workflow) struct BenchmarkStageParameterAuthority {
    pub(in crate::workflow) source_bytes: Vec<u8>,
    pub(in crate::workflow) stage: BenchmarkStageParameterConfig,
}

/// Load one benchmark stage parameter contract and prove its schema against the domain registry.
pub(in crate::workflow) fn load_stage_parameter_authority(
    repository_root: &Path,
    stage_id: &str,
) -> Result<BenchmarkStageParameterAuthority> {
    let source = repository_root.join(PIPELINE_PARAMETERS_PATH);
    let source_bytes = bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)
        .with_context(|| format!("read pipeline parameters {}", source.display()))?;
    let config: BenchmarkPipelineParameterConfig = serde_yaml::from_slice(&source_bytes)
        .with_context(|| format!("parse pipeline parameters {}", source.display()))?;
    validate_pipeline_parameter_config(&config)
        .with_context(|| format!("validate pipeline parameters {}", source.display()))?;
    let stage = config
        .stages
        .get(stage_id)
        .with_context(|| format!("pipeline parameter config lacks {stage_id}"))?
        .clone();

    let registry_path = repository_root.join(PARAMETER_REGISTRY_PATH);
    let registry_bytes = bijux_dna_infra::read_to_end_bounded(&registry_path, 16 * 1024 * 1024)
        .with_context(|| format!("read parameter registry {}", registry_path.display()))?;
    let registry_raw = std::str::from_utf8(&registry_bytes)
        .with_context(|| format!("decode parameter registry {}", registry_path.display()))?;
    let registry: ParameterRegistry = toml::from_str(registry_raw)
        .with_context(|| format!("parse parameter registry {}", registry_path.display()))?;
    if registry.schema_version != PARAMETER_REGISTRY_SCHEMA || registry.source.trim().is_empty() {
        bail!("parameter registry metadata is invalid: {}", registry_path.display());
    }
    let matches =
        registry.entries.iter().filter(|entry| entry.stage_id == stage_id).collect::<Vec<_>>();
    let [entry] = matches.as_slice() else {
        bail!("parameter registry must define exactly one entry for {stage_id}");
    };
    if entry.param_type_id.trim().is_empty() || entry.param_type_id == "paramless" {
        bail!("benchmark stage {stage_id} must have a typed parameter registry entry");
    }
    if entry.schema_version != stage.parameter_schema_version {
        bail!("pipeline parameter schema for {stage_id} differs from the domain registry");
    }

    Ok(BenchmarkStageParameterAuthority { source_bytes, stage })
}

#[cfg(test)]
mod tests {
    use super::{
        load_stage_parameter_authority, PARAMETER_REGISTRY_PATH, PIPELINE_PARAMETERS_PATH,
    };

    fn write_authorities(root: &std::path::Path, registry_schema: &str) -> anyhow::Result<()> {
        let pipeline_path = root.join(PIPELINE_PARAMETERS_PATH);
        let registry_path = root.join(PARAMETER_REGISTRY_PATH);
        std::fs::create_dir_all(
            pipeline_path.parent().ok_or_else(|| anyhow::anyhow!("pipeline parent"))?,
        )?;
        std::fs::create_dir_all(
            registry_path.parent().ok_or_else(|| anyhow::anyhow!("registry parent"))?,
        )?;
        std::fs::write(
            pipeline_path,
            r#"{
  "schema_version": "bijux.benchmark.pipeline_parameters.v1",
  "profile": "test-profile",
  "stages": {
    "fastq.test_stage": {
      "parameter_schema_version": "test.parameter.schema.v1",
      "defaults_provider": "test::defaults",
      "overrides": {}
    }
  }
}
"#,
        )?;
        std::fs::write(
            registry_path,
            format!(
                r#"schema_version = "bijux.param_registry.v1"
source = "test domain contract"

[[entries]]
stage_id = "fastq.test_stage"
param_type_id = "fastq.test_stage"
schema_version = "{registry_schema}"
"#
            ),
        )?;
        Ok(())
    }

    #[test]
    fn parameter_authority_rejects_registry_schema_drift() -> anyhow::Result<()> {
        let temporary = bijux_dna_infra::temp_dir("benchmark-parameter-authority")?;
        write_authorities(temporary.path(), "test.different.schema.v1")?;

        let error = load_stage_parameter_authority(temporary.path(), "fastq.test_stage")
            .err()
            .ok_or_else(|| anyhow::anyhow!("schema drift must be rejected"))?;
        assert!(error.to_string().contains("differs from the domain registry"));
        Ok(())
    }
}
