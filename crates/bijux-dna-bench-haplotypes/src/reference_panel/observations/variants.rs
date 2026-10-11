//! reference panel observations variants.
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;

pub(crate) fn parse_output_sample_ids(vcf_path: &Path) -> Result<Vec<String>> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("prepared panel output is missing the #CHROM header"))?;
    Ok(sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>())
}

pub(crate) fn parse_record_count(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    let count =
        raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()).count();
    u64::try_from(count).map_err(|_| anyhow!("record count overflow"))
}
