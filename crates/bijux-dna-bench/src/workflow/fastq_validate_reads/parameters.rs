//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for FASTQ validation attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BenchmarkParameterValue,
    BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

use super::super::fastq_profile_reads::FastqCampaignSample;
use super::super::parameter_authority::load_stage_parameter_authority;
use super::planner_contract::{load_validation_planner_contract, PlannerEffectiveParameters};

const STAGE_ID: &str = "fastq.validate_reads";
const DEFAULTS_PROVIDER: &str = "bijux_dna_domain_fastq::params::defaults::validate_defaults";

pub(super) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    samples: &[FastqCampaignSample],
) -> Result<BenchmarkExecutionParameterEvidence> {
    let authority = load_stage_parameter_authority(repository_root, STAGE_ID)?;
    let stage = authority.stage;
    let expected_overrides = BTreeMap::from([
        ("threads".to_string(), BenchmarkParameterValue::Integer(4)),
        ("validation_mode".to_string(), BenchmarkParameterValue::Text("strict".to_string())),
    ]);
    if stage.defaults_provider != DEFAULTS_PROVIDER || stage.overrides != expected_overrides {
        bail!("FASTQ validation parameter authority differs from executable policy");
    }
    let planner = load_validation_planner_contract(repository_root)?;
    if planner.parameter_schema_version != stage.parameter_schema_version
        || planner.default_provider != DEFAULTS_PROVIDER
    {
        bail!("FASTQ validation planner parameter authority differs from pipeline policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &authority.source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: STAGE_ID.to_string(),
        parameter_schema_version: stage.parameter_schema_version.clone(),
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides: BTreeMap::from([
            ("threads".to_string(), BenchmarkParameterValue::Integer(4)),
            ("validation_mode".to_string(), BenchmarkParameterValue::Text("strict".to_string())),
        ]),
        resolved_by_sample: samples
            .iter()
            .map(|sample| {
                let parameters = if sample.input_layout == "paired_end" {
                    planner.domain_defaults.paired_end.clone()
                } else {
                    planner.domain_defaults.single_end.clone()
                };
                if parameters.threads != 4 || parameters.validation_mode != "strict" {
                    bail!("planner validation defaults differ from campaign policy");
                }
                Ok((sample.sample_id.clone(), effective_parameter_values(&parameters)))
            })
            .collect::<Result<BTreeMap<_, _>>>()?,
    };
    let mut effective_bytes =
        serde_json::to_vec_pretty(&effective).context("serialize FASTQ validation parameters")?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: stage.parameter_schema_version,
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides_path: "config-pipeline-parameters.yml".to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&authority.source_bytes),
        effective_path: "effective-parameters.json".to_string(),
        effective_sha256: bijux_dna_infra::hash_bytes_sha256(&effective_bytes),
    })
}

fn effective_parameter_values(
    parameters: &PlannerEffectiveParameters,
) -> BTreeMap<String, EffectiveValue> {
    BTreeMap::from([
        ("schema_version".to_string(), EffectiveValue::Text(parameters.schema_version.clone())),
        ("threads".to_string(), EffectiveValue::Integer(parameters.threads.cast_signed())),
        ("validation_mode".to_string(), EffectiveValue::Text(parameters.validation_mode.clone())),
        ("paired_mode".to_string(), EffectiveValue::Text(parameters.paired_mode.clone())),
        ("pair_sync_policy".to_string(), EffectiveValue::Text(parameters.pair_sync_policy.clone())),
    ])
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
