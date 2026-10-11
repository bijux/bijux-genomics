//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for BAM observational campaigns.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkEffectiveParameterValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

use super::super::parameter_authority::load_stage_parameter_authority;

pub(in crate::workflow) struct BamParameterPolicy<'a> {
    pub(in crate::workflow) stage_id: &'a str,
    pub(in crate::workflow) defaults_provider: &'a str,
}

pub(in crate::workflow) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    sample_id: &str,
    policy: BamParameterPolicy<'_>,
    mut resolved: BTreeMap<String, BenchmarkEffectiveParameterValue>,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let authority = load_stage_parameter_authority(repository_root, policy.stage_id)?;
    let stage = authority.stage;
    if stage.defaults_provider != policy.defaults_provider || !stage.overrides.is_empty() {
        bail!("BAM parameter authority differs from executable policy");
    }
    resolved.insert(
        "schema_version".to_string(),
        BenchmarkEffectiveParameterValue::Text(stage.parameter_schema_version.clone()),
    );

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &authority.source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: policy.stage_id.to_string(),
        parameter_schema_version: stage.parameter_schema_version.clone(),
        defaults_provider: policy.defaults_provider.to_string(),
        overrides: BTreeMap::new(),
        resolved_by_sample: BTreeMap::from([(sample_id.to_string(), resolved)]),
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: stage.parameter_schema_version,
        defaults_provider: policy.defaults_provider.to_string(),
        overrides_path: "config-pipeline-parameters.yml".to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&authority.source_bytes),
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
