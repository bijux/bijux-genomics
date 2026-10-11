//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for FASTQ read-length attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BenchmarkFastqLayout, BenchmarkParameterValue,
    BenchmarkPipelineParameterConfig, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

use crate::AdmittedFastqCorpus;

const STAGE_ID: &str = "fastq.profile_read_lengths";
const PARAMETER_SCHEMA: &str = "bijux.fastq.params.read_length_profile.v1";
const DEFAULTS_PROVIDER: &str =
    "bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults";

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
    let expected_overrides = BTreeMap::from([
        ("histogram_bins".to_string(), BenchmarkParameterValue::Integer(100)),
        ("threads".to_string(), BenchmarkParameterValue::Integer(4)),
    ]);
    if stage.parameter_schema_version != PARAMETER_SCHEMA
        || stage.defaults_provider != DEFAULTS_PROVIDER
        || stage.overrides != expected_overrides
    {
        bail!("FASTQ read-length parameter authority differs from executable policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: STAGE_ID.to_string(),
        parameter_schema_version: PARAMETER_SCHEMA.to_string(),
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides: expected_overrides,
        resolved_by_sample: corpus
            .samples
            .iter()
            .map(|sample| {
                (
                    sample.sample_id.clone(),
                    BTreeMap::from([
                        (
                            "schema_version".to_string(),
                            EffectiveValue::Text(PARAMETER_SCHEMA.to_string()),
                        ),
                        (
                            "paired_mode".to_string(),
                            EffectiveValue::Text(
                                match sample.input_layout {
                                    BenchmarkFastqLayout::SingleEnd => "single_end",
                                    BenchmarkFastqLayout::PairedEnd => "paired_end",
                                }
                                .to_string(),
                            ),
                        ),
                        ("threads".to_string(), EffectiveValue::Integer(4)),
                        ("histogram_bins".to_string(), EffectiveValue::Integer(100)),
                    ]),
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

#[cfg(test)]
mod tests {
    use bijux_dna_bench_model::{
        AdmittedFastqCorpus, AdmittedFastqCorpusFile, AdmittedFastqCorpusSample,
        BenchmarkFastqLayout,
    };

    use super::ensure_parameter_evidence;

    #[test]
    fn parameter_evidence_replays_and_rejects_conflicts() -> anyhow::Result<()> {
        let repository = crate::repo::resolve_repo_root()?;
        let temporary = bijux_dna_infra::temp_dir("bench-fastq-parameter-evidence")?;
        let corpus = AdmittedFastqCorpus {
            corpus_id: "corpus".to_string(),
            data_index_path: "/study/data-index.json".to_string(),
            data_index_sha256: "a".repeat(64),
            catalog_path: "/study/catalog.json".to_string(),
            catalog_sha256: "b".repeat(64),
            sample_count: 1,
            file_count: 1,
            input_bytes: 4,
            samples: vec![AdmittedFastqCorpusSample {
                sample_id: "ERR1".to_string(),
                input_layout: BenchmarkFastqLayout::SingleEnd,
                files: vec![AdmittedFastqCorpusFile {
                    path: "/study/ERR1.fastq".to_string(),
                    sha256: "c".repeat(64),
                    size_bytes: 4,
                    mate: None,
                }],
            }],
        };
        let first = ensure_parameter_evidence(&repository, temporary.path(), &corpus)?;
        let second = ensure_parameter_evidence(&repository, temporary.path(), &corpus)?;
        assert_eq!(first, second);
        std::fs::write(temporary.path().join("effective-parameters.json"), b"conflict\n")?;
        assert!(ensure_parameter_evidence(&repository, temporary.path(), &corpus)
            .is_err_and(|error| error.to_string().contains("conflicts")));
        Ok(())
    }
}
