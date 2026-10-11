//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for FASTQ read-profile attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{
    BenchmarkExecutionParameterEvidence, BenchmarkFastqLayout, BenchmarkParameterValue,
    BenchmarkPipelineParameterConfig,
};
use serde::Serialize;

use crate::AdmittedFastqCorpus;

const STAGE_ID: &str = "fastq.profile_reads";
const PARAMETER_SCHEMA: &str = "bijux.fastq.params.stats.v1";
const DEFAULTS_PROVIDER: &str = "bijux_dna_domain_fastq::params::defaults::stats_defaults";

#[derive(Serialize)]
struct EffectiveParameters {
    schema_version: &'static str,
    stage_id: &'static str,
    parameter_schema_version: &'static str,
    defaults_provider: &'static str,
    overrides: BTreeMap<&'static str, i64>,
    resolved_by_sample: BTreeMap<String, EffectiveSampleParameters>,
}

#[derive(Serialize)]
struct EffectiveSampleParameters {
    schema_version: &'static str,
    paired_mode: &'static str,
    threads: i64,
}

pub(super) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    corpus: &AdmittedFastqCorpus,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let source = repository_root.join("benchmarks/contracts/config-pipeline-parameters.yml");
    let source_bytes = bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)
        .with_context(|| format!("read pipeline parameters {}", source.display()))?;
    let config: BenchmarkPipelineParameterConfig = serde_yaml::from_slice(&source_bytes)
        .with_context(|| format!("parse pipeline parameters {}", source.display()))?;
    validate_pipeline_parameter_config(&config).context("validate pipeline parameter config")?;
    let stage = config.stages.get(STAGE_ID).context("parameter config lacks FASTQ stage")?;
    let expected_overrides =
        BTreeMap::from([("threads".to_string(), BenchmarkParameterValue::Integer(4))]);
    if stage.parameter_schema_version != PARAMETER_SCHEMA
        || stage.defaults_provider != DEFAULTS_PROVIDER
        || stage.overrides != expected_overrides
    {
        bail!("FASTQ profile parameter authority differs from executable policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &source_bytes)?;
    let effective = EffectiveParameters {
        schema_version: "bijux.benchmark.effective_parameters.v1",
        stage_id: STAGE_ID,
        parameter_schema_version: PARAMETER_SCHEMA,
        defaults_provider: DEFAULTS_PROVIDER,
        overrides: BTreeMap::from([("threads", 4)]),
        resolved_by_sample: corpus
            .samples
            .iter()
            .map(|sample| {
                (
                    sample.sample_id.clone(),
                    EffectiveSampleParameters {
                        schema_version: PARAMETER_SCHEMA,
                        paired_mode: match sample.input_layout {
                            BenchmarkFastqLayout::SingleEnd => "single_end",
                            BenchmarkFastqLayout::PairedEnd => "paired_end",
                        },
                        threads: 4,
                    },
                )
            })
            .collect(),
    };
    let mut effective_bytes =
        serde_json::to_vec_pretty(&effective).context("serialize effective FASTQ parameters")?;
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
            let existing = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)?;
            if existing != bytes {
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
