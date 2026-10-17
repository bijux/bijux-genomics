//! Physical identity of admitted executable and metadata inputs across test consumption.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{ensure, Context, Result};
use serde::Serialize;
use serde_json::{json, Value};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct FileFingerprint {
    sha256: String,
    bytes: u64,
    readonly: bool,
    #[cfg(unix)]
    mode: u32,
}

pub(crate) struct ExecutableSeal {
    expected: BTreeMap<PathBuf, FileFingerprint>,
    published_manifest: Option<(PathBuf, FileFingerprint)>,
}

pub(crate) struct SealObservation {
    pub(crate) unchanged: bool,
    pub(crate) receipt: Value,
    pub(crate) file_sha256: BTreeMap<PathBuf, String>,
}

impl ExecutableSeal {
    pub(crate) fn capture(paths: &BTreeSet<PathBuf>) -> Result<Self> {
        ensure!(!paths.is_empty(), "compiled input seal must contain physical inputs");
        let expected = paths
            .iter()
            .map(|path| Ok((path.clone(), fingerprint(path)?)))
            .collect::<Result<BTreeMap<_, _>>>()?;
        Ok(Self { expected, published_manifest: None })
    }

    pub(crate) fn require_readonly(&self) -> Result<()> {
        ensure!(
            self.expected.values().all(|file| file.readonly),
            "extracted compiled inputs must remain read-only"
        );
        Ok(())
    }

    pub(crate) fn digest_manifest(&self) -> BTreeMap<PathBuf, String> {
        self.expected.iter().map(|(path, file)| (path.clone(), file.sha256.clone())).collect()
    }

    pub(crate) fn publish_manifest(&mut self, path: &Path) -> Result<Value> {
        ensure!(self.published_manifest.is_none(), "compiled input manifest is already sealed");
        ensure!(!self.expected.contains_key(path), "compiled input manifest overlaps its inputs");
        let mut bytes = serde_json::to_vec_pretty(&json!({
            "schema_version": "bijux.ci.compiled_input_manifest.v1",
            "files": self.expected,
        }))?;
        bytes.push(b'\n');
        bijux_dna_infra::write_bytes(path, &bytes)?;
        self.published_manifest = Some((path.to_path_buf(), fingerprint(path)?));
        Ok(json!({"path": path, "sha256": bijux_dna_infra::hash_bytes_sha256(&bytes),
            "file_count": self.expected.len(), "identity": "bytes_size_permissions_and_canonical_path"}))
    }

    pub(crate) fn verify(&self) -> SealObservation {
        let mut observed = BTreeMap::new();
        let mut errors = BTreeMap::new();
        for path in self.expected.keys() {
            match fingerprint(path) {
                Ok(file) => {
                    observed.insert(path.clone(), file);
                }
                Err(error) => {
                    errors.insert(path.clone(), format!("{error:#}"));
                }
            }
        }
        let mut changed = self
            .expected
            .iter()
            .filter_map(|(path, file)| (observed.get(path) != Some(file)).then_some(path))
            .collect::<Vec<_>>();
        let manifest_observation = self.published_manifest.as_ref().map(|(path, expected)| {
            let result = fingerprint(path);
            if !result.as_ref().is_ok_and(|file| file == expected) {
                changed.push(path);
            }
            match result {
                Ok(file) => json!({"path": path, "observed": file}),
                Err(error) => {
                    errors.insert(path.clone(), format!("{error:#}"));
                    json!({"path": path, "unavailable": true})
                }
            }
        });
        let unchanged = errors.is_empty() && changed.is_empty();
        let file_sha256 =
            observed.iter().map(|(path, file)| (path.clone(), file.sha256.clone())).collect();
        SealObservation {
            unchanged,
            file_sha256,
            receipt: json!({
                "checksum_stable": unchanged, "files_after_execution": observed,
                "changed_inputs": changed, "unavailable_inputs": errors,
                "manifest_after_execution": manifest_observation,
            }),
        }
    }
}

fn fingerprint(path: &Path) -> Result<FileFingerprint> {
    let metadata = path
        .symlink_metadata()
        .with_context(|| format!("inspect compiled input {}", path.display()))?;
    ensure!(
        metadata.is_file() && !metadata.file_type().is_symlink(),
        "compiled input must be an ordinary file: {}",
        path.display()
    );
    ensure!(
        path.canonicalize()? == path,
        "compiled input traverses a symbolic path alias: {}",
        path.display()
    );
    Ok(FileFingerprint {
        sha256: bijux_dna_infra::hash_file_sha256(path)?,
        bytes: metadata.len(),
        readonly: metadata.permissions().readonly(),
        #[cfg(unix)]
        mode: {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sealed_inputs_detect_byte_mutation_deletion_and_retain_manifest_identity() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().canonicalize()?;
        let binary = root.join("binary");
        std::fs::write(&binary, b"original executable")?;
        let mut seal = ExecutableSeal::capture(&[binary.clone()].into_iter().collect())?;
        let manifest = root.join("manifest.json");
        let receipt = seal.publish_manifest(&manifest)?;
        assert!(seal.verify().unchanged);
        std::fs::write(&binary, b"modified executable")?;
        let changed = seal.verify();
        assert!(!changed.unchanged);
        assert_eq!(changed.receipt["changed_inputs"][0], binary.display().to_string());
        std::fs::remove_file(&binary)?;
        let missing = seal.verify();
        assert!(!missing.unchanged);
        assert!(missing.receipt["unavailable_inputs"].get(binary.display().to_string()).is_some());
        assert_eq!(receipt["sha256"], bijux_dna_infra::hash_file_sha256(&manifest)?);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn sealed_inputs_reject_symlink_replacement_and_permission_changes() -> Result<()> {
        use std::os::unix::fs::{symlink, PermissionsExt};

        let directory = tempfile::tempdir()?;
        let root = directory.path().canonicalize()?;
        let binary = root.join("binary");
        std::fs::write(&binary, b"unchanged executable")?;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o555))?;
        let seal = ExecutableSeal::capture(&[binary.clone()].into_iter().collect())?;
        seal.require_readonly()?;
        std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o755))?;
        assert!(!seal.verify().unchanged);
        std::fs::remove_file(&binary)?;
        let replacement = root.join("replacement");
        std::fs::write(&replacement, b"unchanged executable")?;
        symlink(&replacement, &binary)?;
        assert!(!seal.verify().unchanged);
        assert!(ExecutableSeal::capture(&[binary].into_iter().collect()).is_err());
        Ok(())
    }

    #[test]
    fn published_input_manifest_cannot_be_regenerated_or_changed() -> Result<()> {
        let directory = tempfile::tempdir()?;
        let root = directory.path().canonicalize()?;
        let binary = root.join("binary");
        std::fs::write(&binary, b"compiled executable")?;
        let mut seal = ExecutableSeal::capture(&[binary].into_iter().collect())?;
        let manifest = root.join("inputs.json");
        seal.publish_manifest(&manifest)?;
        assert!(seal.publish_manifest(&manifest).is_err());
        std::fs::write(&manifest, b"replaced evidence")?;
        assert!(!seal.verify().unchanged);
        Ok(())
    }
}
