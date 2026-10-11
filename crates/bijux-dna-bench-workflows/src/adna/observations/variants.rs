//! Owner: bijux-dna-bench-workflows
//! Ancient DNA observations variants.
use super::super::{read_vcf_text, Path, Result};

pub(crate) fn summarize_haploid_vcf(vcf_path: &Path) -> Result<(u64, bool)> {
    let raw = read_vcf_text(vcf_path)?;
    let mut variant_count = 0_u64;
    let mut haploid_compatible = true;
    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        variant_count += 1;
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() >= 10 {
            let format_tokens = fields[8].split(':').collect::<Vec<_>>();
            if let Some(index) = format_tokens.iter().position(|token| *token == "GT") {
                for sample_field in &fields[9..] {
                    if let Some(gt) = sample_field.split(':').nth(index) {
                        if gt.contains('/') || gt.contains('|') {
                            haploid_compatible = false;
                        }
                    }
                }
            }
        }
    }
    Ok((variant_count, haploid_compatible))
}

pub(crate) fn count_vcf_variants(vcf_path: &Path) -> Result<u64> {
    let raw = read_vcf_text(vcf_path)?;
    Ok(raw.lines().filter(|line| !line.trim().is_empty() && !line.starts_with('#')).count() as u64)
}
