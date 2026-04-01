use std::path::Path;

use super::{commands, EnvError, ReferenceBuildRequest};

pub(super) fn prepare_reference_indices(
    fasta_target: &Path,
    request: &ReferenceBuildRequest,
    fai: &Path,
    dict: &Path,
    bwa_prefix: &Path,
    bowtie2_prefix: &Path,
) -> Result<(), EnvError> {
    if request.build_fai && !fai.exists() {
        commands::run_command("samtools", &["faidx", fasta_target.to_str().unwrap_or("")])?;
    }
    if request.build_dict && !dict.exists() {
        commands::run_command(
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
        commands::run_command("bwa", &["index", fasta_target.to_str().unwrap_or("")])?;
    }
    if request.build_bowtie2_index && !bowtie2_prefix.with_extension("1.bt2").exists() {
        commands::run_command(
            "bowtie2-build",
            &[
                fasta_target.to_str().unwrap_or(""),
                bowtie2_prefix.to_str().unwrap_or(""),
            ],
        )?;
    }
    Ok(())
}
