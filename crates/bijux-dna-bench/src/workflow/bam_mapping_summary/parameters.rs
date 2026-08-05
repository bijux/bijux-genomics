//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for BAM mapping-summary attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_pipeline_parameter_config;
use bijux_dna_bench_model::{
    BenchmarkExecutionParameterEvidence, BenchmarkPipelineParameterConfig,
};
use serde::Serialize;

const STAGE_ID: &str = "bam.mapping_summary";
const PARAMETER_SCHEMA: &str = "bijux.bam.params.mapping_summary.v1";
const DEFAULTS_PROVIDER: &str =
    "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::MappingSummary)";

#[derive(Serialize)]
struct EffectiveParameters<'a> {
    schema_version: &'static str,
    stage_id: &'static str,
    parameter_schema_version: &'static str,
    defaults_provider: &'static str,
    overrides: BTreeMap<String, String>,
    resolved_by_sample: BTreeMap<&'a str, EffectiveSampleParameters>,
}

#[derive(Serialize)]
struct EffectiveSampleParameters {
    schema_version: &'static str,
    regions: Option<String>,
}

pub(super) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    sample_id: &str,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let source = repository_root.join("benchmarks/contracts/config-pipeline-parameters.yml");
    let source_bytes = bijux_dna_infra::read_to_end_bounded(&source, 16 * 1024 * 1024)
        .with_context(|| format!("read pipeline parameters {}", source.display()))?;
    let config: BenchmarkPipelineParameterConfig = serde_yaml::from_slice(&source_bytes)
        .with_context(|| format!("parse pipeline parameters {}", source.display()))?;
    validate_pipeline_parameter_config(&config).context("validate pipeline parameter config")?;
    let stage = config.stages.get(STAGE_ID).context("parameter config lacks BAM stage")?;
    if stage.parameter_schema_version != PARAMETER_SCHEMA
        || stage.defaults_provider != DEFAULTS_PROVIDER
        || !stage.overrides.is_empty()
    {
        bail!("BAM mapping-summary parameter authority differs from executable policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &source_bytes)?;
    let effective = EffectiveParameters {
        schema_version: "bijux.benchmark.effective_parameters.v1",
        stage_id: STAGE_ID,
        parameter_schema_version: PARAMETER_SCHEMA,
        defaults_provider: DEFAULTS_PROVIDER,
        overrides: BTreeMap::new(),
        resolved_by_sample: BTreeMap::from([(
            sample_id,
            EffectiveSampleParameters { schema_version: PARAMETER_SCHEMA, regions: None },
        )]),
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)?;
    effective_bytes.push(b'\n');
    let effective_path = attempt_directory.join("effective-parameters.json");
    ensure_bytes(&effective_path, &effective_bytes)?;
    Ok(BenchmarkExecutionParameterEvidence {
        parameter_schema_version: PARAMETER_SCHEMA.to_string(),
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides_path: overrides_path.display().to_string(),
        overrides_sha256: bijux_dna_infra::hash_bytes_sha256(&source_bytes),
        effective_path: effective_path.display().to_string(),
        effective_sha256: bijux_dna_infra::hash_bytes_sha256(&effective_bytes),
    })
}

fn ensure_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match std::fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("parameter evidence is not a regular non-symlink file: {}", path.display());
            }
            let observed = std::fs::read(path)
                .with_context(|| format!("read parameter evidence {}", path.display()))?;
            if observed != expected {
                bail!("parameter evidence conflicts with immutable bytes: {}", path.display());
            }
            Ok(())
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)
                .with_context(|| format!("create parameter evidence {}", path.display()))
        }
        Err(error) => {
            Err(error).with_context(|| format!("inspect parameter evidence {}", path.display()))
        }
    }
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::symlink;

    use super::ensure_parameter_evidence;

    #[test]
    fn parameter_evidence_rejects_symlink_substitution() -> anyhow::Result<()> {
        let repository =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let temporary = bijux_dna_infra::temp_dir("bam-mapping-summary-parameters")?;
        let attempt = temporary.path().join("attempt");
        std::fs::create_dir(&attempt)?;
        ensure_parameter_evidence(&repository, &attempt, "ERR2112767")?;
        let effective = attempt.join("effective-parameters.json");
        let replacement = attempt.join("replacement.json");
        std::fs::rename(&effective, &replacement)?;
        symlink(&replacement, &effective)?;

        let error = ensure_parameter_evidence(&repository, &attempt, "ERR2112767")
            .expect_err("symlinked parameter evidence must be rejected");
        assert!(error.to_string().contains("regular non-symlink file"));
        Ok(())
    }
}
