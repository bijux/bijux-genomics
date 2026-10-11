//! call bam support execution reference.
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

pub(crate) fn materialize_reference_fasta(
    source_reference: &Path,
    artifacts_root: &Path,
) -> Result<PathBuf> {
    let reference_root = artifacts_root.join("reference");
    bijux_dna_infra::ensure_dir(&reference_root)
        .with_context(|| format!("create {}", reference_root.display()))?;
    let file_name = source_reference.file_name().ok_or_else(|| {
        anyhow!("reference FASTA has no file name: {}", source_reference.display())
    })?;
    let materialized_reference = reference_root.join(file_name);
    fs::copy(source_reference, &materialized_reference).with_context(|| {
        format!(
            "copy governed reference {} to {}",
            source_reference.display(),
            materialized_reference.display()
        )
    })?;
    Ok(materialized_reference)
}
