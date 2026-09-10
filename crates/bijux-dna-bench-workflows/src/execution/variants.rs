//! Owner: bijux-dna-bench-workflows
//! Local reference copies and observed VCF sample counts for workflow handoffs.
use anyhow::{anyhow, Context, Result};
use std::path::{Path, PathBuf};
pub(crate) fn materialize_reference_fasta(source: &Path, artifacts: &Path) -> Result<PathBuf> {
    let root = artifacts.join("reference");
    bijux_dna_infra::ensure_dir(&root)?;
    let name = source.file_name().context("reference FASTA has no file name")?;
    let destination = root.join(name);
    bijux_dna_infra::copy_file(source, &destination)?;
    Ok(destination)
}
pub(crate) fn parse_output_sample_count(path: &Path) -> Result<u64> {
    let raw = bijux_dna_stages_vcf::vcf_io::read_vcf_text(path)?;
    let header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("VCF output is missing the #CHROM header"))?;
    u64::try_from(header.split('\t').skip(9).count())
        .map_err(|_| anyhow!("VCF output sample count overflowed u64"))
}
