//! Binding-declared artifact output closure for generic lifecycle owners.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

use super::{BindingOutputV1, OutputDiscoveryV1, OutputSidecarDiscoveryV1};

/// Expand one primary artifact path to every exact file declared by its binding.
///
/// # Errors
/// Rejects malformed or duplicate declarations, paths without a file name, colliding output
/// paths, and glob outputs presented to an exact-output lifecycle. Native path bytes are retained.
pub fn required_binding_output_paths(
    output: &BindingOutputV1,
    primary_path: &Path,
) -> Result<Vec<PathBuf>> {
    validate_output_discovery(&output.discovery)?;
    if primary_path.file_name().is_none() {
        bail!("binding primary output requires a file name");
    }
    let mut paths = vec![primary_path.to_path_buf()];
    match &output.discovery {
        OutputDiscoveryV1::ExactPath => {}
        OutputDiscoveryV1::ExactPathWithSidecars { sidecars } => {
            for sidecar in sidecars {
                let path = match sidecar {
                    OutputSidecarDiscoveryV1::AppendSuffix { suffix } => {
                        let mut path = primary_path.as_os_str().to_os_string();
                        path.push(suffix);
                        PathBuf::from(path)
                    }
                    OutputSidecarDiscoveryV1::ReplaceExtension { extension } => {
                        primary_path.with_extension(extension)
                    }
                };
                if paths.contains(&path) {
                    bail!("binding output paths collide at {}", path.display());
                }
                paths.push(path);
            }
        }
        OutputDiscoveryV1::Glob { .. } => {
            bail!("glob binding output cannot define an exact benchmark output closure");
        }
    }
    Ok(paths)
}

/// One declaration authority shared by registry admission and concrete path expansion.
pub(super) fn validate_output_discovery(discovery: &OutputDiscoveryV1) -> Result<()> {
    match discovery {
        OutputDiscoveryV1::ExactPath => {}
        OutputDiscoveryV1::ExactPathWithSidecars { sidecars } => {
            if sidecars.is_empty() {
                bail!("binding exact sidecar discovery has no sidecars");
            }
            for (index, sidecar) in sidecars.iter().enumerate() {
                match sidecar {
                    OutputSidecarDiscoveryV1::AppendSuffix { suffix }
                        if suffix.len() <= 1
                            || !suffix.starts_with('.')
                            || suffix.contains(['/', '\\', '\0']) =>
                    {
                        bail!("binding exact sidecar suffix is not canonical");
                    }
                    OutputSidecarDiscoveryV1::ReplaceExtension { extension }
                        if extension.is_empty()
                            || extension.starts_with('.')
                            || extension.contains(['/', '\\', '\0']) =>
                    {
                        bail!("binding exact sidecar extension is not canonical");
                    }
                    _ => {}
                }
                if sidecars[..index].contains(sidecar) {
                    bail!("binding exact sidecar discovery repeats a declaration");
                }
            }
        }
        OutputDiscoveryV1::Glob { pattern } => {
            if pattern.trim().is_empty() {
                bail!("output glob pattern must not be empty");
            }
        }
    }
    Ok(())
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

    fn indexed_output(sidecars: Vec<OutputSidecarDiscoveryV1>) -> BindingOutputV1 {
        BindingOutputV1 {
            name: "alignment".to_owned(),
            artifact_contract_id: "bijux.bam.artifact.coordinate_sorted_indexed.v1".to_owned(),
            discovery: OutputDiscoveryV1::ExactPathWithSidecars { sidecars },
        }
    }

    #[test]
    fn malformed_and_duplicate_sidecars_fail_before_path_expansion() {
        let append =
            |suffix: &str| OutputSidecarDiscoveryV1::AppendSuffix { suffix: suffix.to_owned() };
        let replace = |extension: &str| OutputSidecarDiscoveryV1::ReplaceExtension {
            extension: extension.to_owned(),
        };
        for sidecars in [
            vec![],
            vec![append("")],
            vec![append(".")],
            vec![append("bai")],
            vec![append(".dir/bai")],
            vec![append(".dir\\bai")],
            vec![append(".bai\0")],
            vec![replace("")],
            vec![replace(".bai")],
            vec![replace("dir/bai")],
            vec![replace("dir\\bai")],
            vec![replace("bai\0")],
            vec![append(".bai"), append(".bai")],
            vec![replace("bai"), replace("bai")],
        ] {
            let output = indexed_output(sidecars);
            assert!(super::validate_output_discovery(&output.discovery).is_err(), "{output:?}");
            assert!(required_binding_output_paths(&output, Path::new("sample.bam")).is_err());
        }
    }

    #[test]
    fn sidecars_cannot_alias_the_primary_or_each_other() {
        let same_extension = indexed_output(vec![OutputSidecarDiscoveryV1::ReplaceExtension {
            extension: "bam".to_owned(),
        }]);
        assert!(required_binding_output_paths(&same_extension, Path::new("sample.bam")).is_err());
        let converging_rules = indexed_output(vec![
            OutputSidecarDiscoveryV1::AppendSuffix { suffix: ".bai".to_owned() },
            OutputSidecarDiscoveryV1::ReplaceExtension { extension: "bam.bai".to_owned() },
        ]);
        assert!(required_binding_output_paths(&converging_rules, Path::new("sample.bam")).is_err());
    }

    #[test]
    fn exact_lifecycle_rejects_globs_and_missing_file_names() {
        let mut output = indexed_output(vec![OutputSidecarDiscoveryV1::AppendSuffix {
            suffix: ".bai".to_owned(),
        }]);
        for path in ["", ".", "..", "/"] {
            assert!(required_binding_output_paths(&output, Path::new(path)).is_err());
        }
        output.discovery = OutputDiscoveryV1::Glob { pattern: "*.bam".to_owned() };
        assert!(super::validate_output_discovery(&output.discovery).is_ok());
        assert!(required_binding_output_paths(&output, Path::new("sample.bam")).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn sidecars_preserve_non_unicode_native_path_bytes() -> anyhow::Result<()> {
        use std::ffi::OsString;
        use std::os::unix::ffi::{OsStrExt, OsStringExt};

        let primary = PathBuf::from(OsString::from_vec(b"sample \xff.bam".to_vec()));
        let output = indexed_output(vec![
            OutputSidecarDiscoveryV1::AppendSuffix { suffix: ".bai".to_owned() },
            OutputSidecarDiscoveryV1::ReplaceExtension { extension: "bai".to_owned() },
        ]);
        let paths = required_binding_output_paths(&output, &primary)?;
        assert_eq!(paths[0], primary);
        assert_eq!(paths[1].as_os_str().as_bytes(), b"sample \xff.bam.bai");
        assert_eq!(paths[2].as_os_str().as_bytes(), b"sample \xff.bai");
        Ok(())
    }
}
