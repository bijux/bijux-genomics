use std::io::BufRead;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bijux_dna_core::contract::ExecutionStep;

fn has_extension(path: &Path, ext: &str) -> bool {
    path.extension()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|value| value.eq_ignore_ascii_case(ext))
}

fn file_name_ends_with(path: &Path, suffix: &str) -> bool {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name.ends_with(suffix))
}

/// # Errors
/// Returns an error when a VCF/BCF input lacks an index.
pub fn validate_bgzip_tabix(input: &Path) -> Result<()> {
    if file_name_ends_with(input, ".vcf.gz") || has_extension(input, "vcf") {
        let tbi = Path::new(&format!("{}.tbi", input.display())).to_path_buf();
        let csi = Path::new(&format!("{}.csi", input.display())).to_path_buf();
        if !tbi.exists() && !csi.exists() {
            bail!(
                "input contract violation: missing VCF index (.tbi/.csi) for {}",
                input.display()
            );
        }
    }
    if has_extension(input, "bcf") {
        let csi = input.with_extension("bcf.csi");
        if !csi.exists() {
            bail!(
                "input contract violation: missing BCF index (.csi) for {}",
                input.display()
            );
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error when BAM/CRAM input lacks an index.
pub fn validate_bam_index(input: &Path) -> Result<()> {
    if has_extension(input, "bam") {
        let bai = input.with_extension("bam.bai");
        let csi = input.with_extension("bam.csi");
        if !bai.exists() && !csi.exists() {
            bail!(
                "input contract violation: missing BAM index (.bai/.csi) for {}",
                input.display()
            );
        }
    }
    if has_extension(input, "cram") {
        let crai = input.with_extension("cram.crai");
        if !crai.exists() {
            bail!(
                "input contract violation: missing CRAM index (.crai) for {}",
                input.display()
            );
        }
    }
    Ok(())
}

/// # Errors
/// Returns an error when FASTQ content does not start with a header marker.
pub fn validate_fastq_format(input: &Path) -> Result<()> {
    if !(has_extension(input, "fastq")
        || has_extension(input, "fq")
        || file_name_ends_with(input, ".fastq.gz")
        || file_name_ends_with(input, ".fq.gz"))
    {
        return Ok(());
    }
    let file = std::fs::File::open(input).with_context(|| format!("open {}", input.display()))?;
    let mut reader = std::io::BufReader::new(file);
    let mut first = String::new();
    let _ = reader.read_line(&mut first)?;
    if !first.starts_with('@') {
        bail!(
            "input contract violation: FASTQ header must start with '@' ({})",
            input.display()
        );
    }
    Ok(())
}

/// # Errors
/// Returns an error if any known input contract fails.
pub fn validate_stage_inputs(step: &ExecutionStep) -> Result<()> {
    for artifact in &step.io.inputs {
        let path = &artifact.path;
        if !path.exists() {
            continue;
        }
        validate_bgzip_tabix(path)?;
        validate_bam_index(path)?;
        validate_fastq_format(path)?;
    }
    Ok(())
}
