//! Owner: bijux-dna-bench
//! Immutable effective-parameter evidence for BAM mapping-summary attempts.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkEffectiveParameterValue as EffectiveValue, BenchmarkEffectiveParameters,
    BenchmarkExecutionParameterEvidence, BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1,
};

use super::super::parameter_authority::load_stage_parameter_authority;

const STAGE_ID: &str = "bam.mapping_summary";
const DEFAULTS_PROVIDER: &str =
    "bijux_dna_domain_bam::stage_specs::stage_spec(BamStage::MappingSummary)";

pub(super) fn ensure_parameter_evidence(
    repository_root: &Path,
    attempt_directory: &Path,
    sample_id: &str,
) -> Result<BenchmarkExecutionParameterEvidence> {
    let authority = load_stage_parameter_authority(repository_root, STAGE_ID)?;
    let stage = authority.stage;
    if stage.defaults_provider != DEFAULTS_PROVIDER || !stage.overrides.is_empty() {
        bail!("BAM mapping-summary parameter authority differs from executable policy");
    }

    let overrides_path = attempt_directory.join("config-pipeline-parameters.yml");
    ensure_bytes(&overrides_path, &authority.source_bytes)?;
    let effective = BenchmarkEffectiveParameters {
        schema_version: BENCHMARK_EFFECTIVE_PARAMETERS_SCHEMA_V1.to_string(),
        stage_id: STAGE_ID.to_string(),
        parameter_schema_version: stage.parameter_schema_version.clone(),
        defaults_provider: DEFAULTS_PROVIDER.to_string(),
        overrides: BTreeMap::new(),
        resolved_by_sample: BTreeMap::from([(
            sample_id.to_string(),
            BTreeMap::from([
                (
                    "schema_version".to_string(),
                    EffectiveValue::Text(stage.parameter_schema_version.clone()),
                ),
                ("regions".to_string(), EffectiveValue::Null),
            ]),
        )]),
    };
    let mut effective_bytes = serde_json::to_vec_pretty(&effective)?;
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
