//! phasing observations variants.
use crate::phasing::model::PhaseSummary;
use anyhow::anyhow;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;

pub(crate) fn summarize_phasing_genotypes(vcf_path: &Path) -> Result<PhaseSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let sample_header = raw
        .lines()
        .find(|line| line.starts_with("#CHROM\t"))
        .ok_or_else(|| anyhow!("phasing VCF is missing the #CHROM header"))?;
    let sample_ids = sample_header.split('\t').skip(9).map(str::to_string).collect::<Vec<_>>();
    let mut in_phase_block = vec![false; sample_ids.len()];
    let mut genotype_count = 0u64;
    let mut phased_genotypes = 0u64;
    let mut unphased_genotypes = 0u64;
    let mut phase_set_count = 0u64;

    for line in raw.lines().filter(|line| !line.starts_with('#') && !line.trim().is_empty()) {
        let fields = line.split('\t').collect::<Vec<_>>();
        for (sample_index, sample_field) in fields.iter().skip(9).enumerate() {
            let gt = sample_field.split(':').next().unwrap_or_default();
            if gt.is_empty() || gt == "." || gt == "./." || gt == ".|." {
                in_phase_block[sample_index] = false;
                continue;
            }
            genotype_count += 1;
            if gt.contains('|') {
                phased_genotypes += 1;
                if !in_phase_block[sample_index] {
                    phase_set_count += 1;
                    in_phase_block[sample_index] = true;
                }
            } else if gt.contains('/') {
                unphased_genotypes += 1;
                in_phase_block[sample_index] = false;
            } else {
                in_phase_block[sample_index] = false;
            }
        }
    }

    Ok(PhaseSummary {
        genotype_count,
        phased_genotypes,
        unphased_genotypes,
        phase_set_count,
        sample_ids,
    })
}
