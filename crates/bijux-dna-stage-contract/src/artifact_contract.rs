//! Binding-declared artifact output closure for generic lifecycle owners.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use crate::{BindingOutputV1, OutputDiscoveryV1, OutputSidecarDiscoveryV1};

/// Expand one primary artifact path to every exact file declared by its binding.
///
/// # Errors
/// Returns an error when an exact sidecar declaration is empty or contains a path separator, or
/// when a glob output is presented to an exact-output lifecycle.
pub fn required_binding_output_paths(
    output: &BindingOutputV1,
    primary_path: &Path,
) -> Result<Vec<PathBuf>> {
    let mut paths = vec![primary_path.to_path_buf()];
    match &output.discovery {
        OutputDiscoveryV1::ExactPath => {}
        OutputDiscoveryV1::ExactPathWithSidecars { sidecars } => {
            if sidecars.is_empty() {
                bail!("binding exact sidecar discovery has no sidecars");
            }
            for sidecar in sidecars {
                match sidecar {
                    OutputSidecarDiscoveryV1::AppendSuffix { suffix } => {
                        if suffix.is_empty()
                            || suffix.contains(['/', '\\'])
                            || !suffix.starts_with('.')
                        {
                            bail!("binding exact sidecar suffix is not canonical");
                        }
                        paths.push(PathBuf::from(format!(
                            "{}{}",
                            primary_path.to_string_lossy(),
                            suffix
                        )));
                    }
                    OutputSidecarDiscoveryV1::ReplaceExtension { extension } => {
                        if extension.is_empty()
                            || extension.starts_with('.')
                            || extension.contains(['/', '\\'])
                        {
                            bail!("binding exact sidecar extension is not canonical");
                        }
                        paths.push(primary_path.with_extension(extension));
                    }
                }
            }
        }
        OutputDiscoveryV1::Glob { .. } => {
            bail!("glob binding output cannot define an exact benchmark output closure");
        }
    }
    Ok(paths)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::required_binding_output_paths;
    use crate::{BindingOutputV1, OutputDiscoveryV1, OutputSidecarDiscoveryV1};

    #[test]
    fn indexed_bam_binding_closes_primary_and_bai_paths() -> anyhow::Result<()> {
        let output = BindingOutputV1 {
            name: "marked_bam".to_owned(),
            artifact_contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_owned(),
            discovery: OutputDiscoveryV1::ExactPathWithSidecars {
                sidecars: vec![OutputSidecarDiscoveryV1::AppendSuffix {
                    suffix: ".bai".to_owned(),
                }],
            },
        };
        assert_eq!(
            required_binding_output_paths(&output, Path::new("marked.bam"))?,
            [PathBuf::from("marked.bam"), PathBuf::from("marked.bam.bai")]
        );
        Ok(())
    }

    #[test]
    fn indexed_vcf_binding_preserves_declared_csi_naming() -> anyhow::Result<()> {
        let output = BindingOutputV1 {
            name: "called_vcf".to_owned(),
            artifact_contract_id: "bijux.vcf.artifact.coordinate_sorted_indexed.v1".to_owned(),
            discovery: OutputDiscoveryV1::ExactPathWithSidecars {
                sidecars: vec![OutputSidecarDiscoveryV1::AppendSuffix {
                    suffix: ".csi".to_owned(),
                }],
            },
        };
        assert_eq!(
            required_binding_output_paths(&output, Path::new("calls.vcf.gz"))?,
            [PathBuf::from("calls.vcf.gz"), PathBuf::from("calls.vcf.gz.csi")]
        );
        Ok(())
    }

    #[test]
    fn picard_bam_binding_replaces_extension_for_bai_path() -> anyhow::Result<()> {
        let output = BindingOutputV1 {
            name: "marked_bam".to_owned(),
            artifact_contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_owned(),
            discovery: OutputDiscoveryV1::ExactPathWithSidecars {
                sidecars: vec![OutputSidecarDiscoveryV1::ReplaceExtension {
                    extension: "bai".to_owned(),
                }],
            },
        };
        assert_eq!(
            required_binding_output_paths(&output, Path::new("marked.bam"))?,
            [PathBuf::from("marked.bam"), PathBuf::from("marked.bai")]
        );
        Ok(())
    }
}
