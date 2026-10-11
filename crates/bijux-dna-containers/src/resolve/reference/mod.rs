use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{cache, EnvError};
use digest::hash_file_sha256;

mod digest;
mod index_preparation;

use index_preparation::prepare_reference_indices;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceRecord {
    pub digest: String,
    pub root: PathBuf,
    pub fasta: PathBuf,
    pub fai: PathBuf,
    pub dict: PathBuf,
    pub bwa_prefix: PathBuf,
    pub bowtie2_prefix: PathBuf,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct ReferenceBuildRequest {
    pub build_fai: bool,
    pub build_dict: bool,
    pub build_bwa_index: bool,
    pub build_bowtie2_index: bool,
}

#[derive(Debug, Clone)]
pub struct ReferenceRegistry {
    root: PathBuf,
}

impl ReferenceRegistry {
    #[must_use]
    pub fn new() -> Self {
        Self { root: cache::reference_cache_dir() }
    }

    /// # Errors
    /// Returns an error if the reference cannot be registered or prepared.
    pub fn prepare_reference(
        &self,
        fasta: &Path,
        request: &ReferenceBuildRequest,
    ) -> Result<ReferenceRecord, EnvError> {
        bijux_dna_infra::ensure_dir(&self.root)?;
        let digest = hash_file_sha256(fasta)?;
        let ref_root = self.root.join(&digest);
        bijux_dna_infra::ensure_dir(&ref_root)?;
        let fasta_target = ref_root.join(
            fasta
                .file_name()
                .ok_or_else(|| EnvError::Parse("invalid reference path".to_string()))?,
        );
        if !fasta_target.exists() {
            std::fs::copy(fasta, &fasta_target)?;
        }
        let fai = fasta_target.with_extension("fai");
        let dict = fasta_target.with_extension("dict");
        let bwa_prefix = fasta_target.clone();
        let bowtie2_prefix = fasta_target.clone();
        prepare_reference_indices(
            &fasta_target,
            request,
            &fai,
            &dict,
            &bwa_prefix,
            &bowtie2_prefix,
        )?;

        Ok(ReferenceRecord {
            digest,
            root: ref_root,
            fasta: fasta_target,
            fai,
            dict,
            bwa_prefix,
            bowtie2_prefix,
        })
    }
}

impl Default for ReferenceRegistry {
    fn default() -> Self {
        Self::new()
    }
}
