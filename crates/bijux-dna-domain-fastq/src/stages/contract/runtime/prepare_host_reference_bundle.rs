use std::path::Path;

use anyhow::{anyhow, Result};
use sha2::{Digest, Sha256};

use crate::artifacts::{
    HostReferenceBundleFileV1, PrepareHostReferenceBundleReportV1,
    PREPARE_HOST_REFERENCE_BUNDLE_REPORT_SCHEMA_VERSION,
};

/// Prepare immutable host-reference bundle identity.
///
/// # Errors
/// Returns an error when no source files are provided or any source cannot be hashed.
pub fn prepare_host_reference_bundle(
    host_reference_sources: &[&Path],
    reference_build: Option<&str>,
) -> Result<PrepareHostReferenceBundleReportV1> {
    if host_reference_sources.is_empty() {
        return Err(anyhow!(
            "fastq.prepare_host_reference_bundle requires at least one reference source"
        ));
    }

    let mut files = Vec::with_capacity(host_reference_sources.len());
    for path in host_reference_sources {
        if !path.exists() {
            return Err(anyhow!("host reference source missing: {}", path.display()));
        }
        let digest = bijux_dna_infra::hash_file_sha256(path)
            .map_err(|err| anyhow!("hash {}: {err}", path.display()))?;
        files.push(HostReferenceBundleFileV1 { path: path.display().to_string(), sha256: digest });
    }
    files.sort_by(|left, right| left.path.cmp(&right.path));

    let mut hasher = Sha256::new();
    for file in &files {
        hasher.update(file.path.as_bytes());
        hasher.update(b"=");
        hasher.update(file.sha256.as_bytes());
        hasher.update(b"\n");
    }
    let bundle_hash =
        hasher.finalize().iter().map(|byte| format!("{byte:02x}")).collect::<String>();

    Ok(PrepareHostReferenceBundleReportV1 {
        schema_version: PREPARE_HOST_REFERENCE_BUNDLE_REPORT_SCHEMA_VERSION.to_string(),
        stage: "fastq.prepare_host_reference_bundle".to_string(),
        stage_id: "fastq.prepare_host_reference_bundle".to_string(),
        tool_id: "bijux".to_string(),
        reference_build: reference_build.unwrap_or("governed").to_string(),
        bundle_hash,
        bundle_file_count: files.len() as u64,
        files,
    })
}

#[cfg(test)]
mod tests {
    use super::prepare_host_reference_bundle;

    #[test]
    fn prepare_host_reference_bundle_hashes_and_sorts_files() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-fastq-host-bundle")?;
        let ref_a = temp.path().join("b.fa");
        let ref_b = temp.path().join("a.fa");
        std::fs::write(&ref_a, ">chr1\nACGT\n")?;
        std::fs::write(&ref_b, ">chr2\nTGCA\n")?;

        let report =
            prepare_host_reference_bundle(&[ref_a.as_path(), ref_b.as_path()], Some("hg38"))?;
        assert_eq!(report.bundle_file_count, 2);
        assert_eq!(report.files[0].path, ref_b.display().to_string());
        assert_eq!(report.reference_build, "hg38");
        assert!(!report.bundle_hash.is_empty());
        Ok(())
    }
}
