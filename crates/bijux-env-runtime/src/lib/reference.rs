use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::EnvError;

#[must_use]
pub fn reference_cache_dir() -> PathBuf {
    let home = std::env::var_os("HOME").map_or_else(|| PathBuf::from("."), PathBuf::from);
    home.join(".cache").join("bijux").join("references")
}

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
        Self {
            root: reference_cache_dir(),
        }
    }

    /// # Errors
    /// Returns an error if the reference cannot be registered or prepared.
    pub fn prepare_reference(
        &self,
        fasta: &Path,
        request: &ReferenceBuildRequest,
    ) -> Result<ReferenceRecord, EnvError> {
        std::fs::create_dir_all(&self.root)?;
        let digest = hash_file_sha256(fasta)?;
        let ref_root = self.root.join(&digest);
        std::fs::create_dir_all(&ref_root)?;
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

        if request.build_fai && !fai.exists() {
            run_command("samtools", &["faidx", fasta_target.to_str().unwrap_or("")])?;
        }
        if request.build_dict && !dict.exists() {
            run_command(
                "gatk",
                &[
                    "CreateSequenceDictionary",
                    "-R",
                    fasta_target.to_str().unwrap_or(""),
                    "-O",
                    dict.to_str().unwrap_or(""),
                ],
            )?;
        }
        if request.build_bwa_index && !bwa_prefix.with_extension("bwt").exists() {
            run_command("bwa", &["index", fasta_target.to_str().unwrap_or("")])?;
        }
        if request.build_bowtie2_index
            && !bowtie2_prefix.with_extension("1.bt2").exists()
        {
            run_command(
                "bowtie2-build",
                &[
                    fasta_target.to_str().unwrap_or(""),
                    bowtie2_prefix.to_str().unwrap_or(""),
                ],
            )?;
        }

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

fn run_command(cmd: &str, args: &[&str]) -> Result<(), EnvError> {
    let status = Command::new(cmd).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(EnvError::Platform(format!("command failed: {cmd} {args:?}")))
    }
}

fn hash_file_sha256(path: &Path) -> Result<String, EnvError> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 8192];
    loop {
        let n = std::io::Read::read(&mut file, &mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}
