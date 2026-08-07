//! Owner: bijux-dna-bench
//! Immutable persistence for canonical stage-contract parameter projections.

use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_stage_contract::{validate_effective_tool_parameters, EffectiveToolParametersV1};

pub const EFFECTIVE_TOOL_PARAMETERS_FILE: &str = "effective-tool-parameters.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EffectiveToolParametersPublication {
    pub path: PathBuf,
    pub sha256: String,
}

/// Archive one validated parameter projection without permitting in-place replacement.
///
/// # Errors
/// Returns an error when the artifact is invalid, the attempt directory is unsafe, serialization
/// fails, or existing bytes conflict with the canonical projection.
pub fn archive_effective_tool_parameters(
    attempt_directory: &Path,
    effective: &EffectiveToolParametersV1,
) -> Result<EffectiveToolParametersPublication> {
    validate_effective_tool_parameters(effective)?;
    let metadata = fs::symlink_metadata(attempt_directory)
        .with_context(|| format!("inspect attempt directory {}", attempt_directory.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        bail!("effective parameter attempt directory is not a regular directory");
    }
    let path = attempt_directory.join(EFFECTIVE_TOOL_PARAMETERS_FILE);
    let value = serde_json::to_value(effective).context("serialize effective tool parameters")?;
    let canonical = bijux_dna_core::contract::canonical::canonicalize_json_value(&value);
    let mut bytes =
        serde_json::to_vec_pretty(&canonical).context("encode effective tool parameters")?;
    bytes.push(b'\n');
    ensure_immutable_bytes(&path, &bytes)?;
    Ok(EffectiveToolParametersPublication {
        path,
        sha256: bijux_dna_infra::hash_bytes_sha256(&bytes),
    })
}

fn ensure_immutable_bytes(path: &Path, expected: &[u8]) -> Result<()> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || !metadata.is_file() {
                bail!("effective parameter artifact is not a regular file");
            }
            let observed = bijux_dna_infra::read_to_end_bounded(path, 16 * 1024 * 1024)?;
            if observed != expected {
                bail!("effective parameter artifact conflicts with immutable bytes");
            }
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bijux_dna_infra::atomic_create_bytes(path, expected)
                .with_context(|| format!("archive effective parameters {}", path.display()))
        }
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_core::contract::ToolConstraints;
    use bijux_dna_core::ids::{StageId, ToolId};
    use bijux_dna_stage_contract::{
        EffectiveToolParameterSourceV1, EffectiveToolParameterV1, EffectiveToolParametersV1,
        EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1,
    };

    use super::*;

    #[test]
    fn archive_is_deterministic_and_conflict_safe() -> Result<()> {
        let attempt = tempfile::tempdir()?;
        let effective = effective_parameters();
        let first = archive_effective_tool_parameters(attempt.path(), &effective)?;
        let second = archive_effective_tool_parameters(attempt.path(), &effective)?;
        assert_eq!(first, second);
        assert_eq!(bijux_dna_infra::hash_file_sha256(&first.path)?, first.sha256);

        let mut invalid = effective.clone();
        invalid.schema_version = "bijux.effective_tool_parameters.unsupported".to_owned();
        assert!(archive_effective_tool_parameters(attempt.path(), &invalid)
            .is_err_and(|error| error.to_string().contains("schema is unsupported")));

        fs::write(&first.path, b"conflict\n")?;
        assert!(archive_effective_tool_parameters(attempt.path(), &effective)
            .is_err_and(|error| error.to_string().contains("immutable bytes")));
        Ok(())
    }

    fn effective_parameters() -> EffectiveToolParametersV1 {
        EffectiveToolParametersV1 {
            schema_version: EFFECTIVE_TOOL_PARAMETERS_SCHEMA_V1.to_owned(),
            binding_id: "fastq.trim_reads__fastp".to_owned(),
            stage_id: StageId::new("fastq.trim_reads"),
            tool_id: ToolId::new("fastp"),
            allocated_cpus: 4,
            resources: ToolConstraints {
                runtime: "apptainer".to_owned(),
                mem_gb: 4,
                tmp_gb: 2,
                threads: 4,
            },
            parameters: BTreeMap::from([(
                "threads".to_owned(),
                EffectiveToolParameterV1 {
                    value: "4".to_owned(),
                    source: EffectiveToolParameterSourceV1::ScientificDefault,
                },
            )]),
        }
    }
}
