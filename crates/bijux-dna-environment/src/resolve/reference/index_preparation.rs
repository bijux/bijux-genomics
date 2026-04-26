use std::path::Path;

use crate::resolve::commands;

use super::{super::EnvError, ReferenceBuildRequest};

pub(super) fn prepare_reference_indices(
    fasta_target: &Path,
    request: &ReferenceBuildRequest,
    fai: &Path,
    dict: &Path,
    bwa_prefix: &Path,
    bowtie2_prefix: &Path,
) -> Result<(), EnvError> {
    if request.build_fai && !fai.exists() {
        commands::run_command("samtools", &["faidx", path_str(fasta_target)?])?;
    }
    if request.build_dict && !dict.exists() {
        commands::run_command(
            "gatk",
            &[
                "CreateSequenceDictionary",
                "-R",
                path_str(fasta_target)?,
                "-O",
                path_str(dict)?,
            ],
        )?;
    }
    if request.build_bwa_index && !bwa_prefix.with_extension("bwt").exists() {
        commands::run_command("bwa", &["index", path_str(fasta_target)?])?;
    }
    if request.build_bowtie2_index && !bowtie2_prefix.with_extension("1.bt2").exists() {
        commands::run_command(
            "bowtie2-build",
            &[path_str(fasta_target)?, path_str(bowtie2_prefix)?],
        )?;
    }
    Ok(())
}

fn path_str(path: &Path) -> Result<&str, EnvError> {
    path.to_str()
        .ok_or_else(|| EnvError::Platform(format!("path is not valid UTF-8: {}", path.display())))
}
