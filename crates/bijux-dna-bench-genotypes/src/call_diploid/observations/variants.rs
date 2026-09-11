//! call diploid observations variants.
use crate::call_diploid::model::DiploidGenotypeSummary;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;

pub(crate) fn summarize_diploid_genotypes(vcf_path: &Path) -> Result<DiploidGenotypeSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut summary = DiploidGenotypeSummary {
        called_genotypes: 0,
        heterozygous_count: 0,
        homozygous_ref_count: 0,
        homozygous_alt_count: 0,
        missing_count: 0,
    };
    let mut observed_gt = false;

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("diploid smoke output row is missing FORMAT/sample fields: {line}");
        }
        let gt_index = fields[8]
            .split(':')
            .position(|token| token == "GT")
            .ok_or_else(|| anyhow!("diploid smoke output row is missing GT in FORMAT: {line}"))?;
        observed_gt = true;

        for sample_field in &fields[9..] {
            let gt = sample_field
                .split(':')
                .nth(gt_index)
                .ok_or_else(|| anyhow!("diploid smoke sample field is missing GT value: {line}"))?;
            let (left, right, missing) = parse_diploid_genotype(gt)?;
            if missing {
                summary.missing_count += 1;
                continue;
            }
            summary.called_genotypes += 1;
            match (left, right) {
                (0, 0) => summary.homozygous_ref_count += 1,
                (left, right) if left == right => summary.homozygous_alt_count += 1,
                _ => summary.heterozygous_count += 1,
            }
        }
    }

    if !observed_gt {
        bail!("diploid smoke output did not contain any GT fields");
    }
    Ok(summary)
}

pub(crate) fn parse_diploid_genotype(genotype: &str) -> Result<(u32, u32, bool)> {
    let separator = if genotype.contains('|') {
        '|'
    } else if genotype.contains('/') {
        '/'
    } else {
        bail!("diploid smoke genotype is not diploid-compatible: `{genotype}`");
    };
    let parts = genotype.split(separator).collect::<Vec<_>>();
    if parts.len() != 2 {
        bail!("diploid smoke genotype must contain exactly two alleles: `{genotype}`");
    }
    if parts.iter().all(|allele| *allele == ".") {
        return Ok((0, 0, true));
    }
    let left = parts[0].parse::<u32>().with_context(|| {
        format!("diploid smoke genotype has non-numeric left allele: `{genotype}`")
    })?;
    let right = parts[1].parse::<u32>().with_context(|| {
        format!("diploid smoke genotype has non-numeric right allele: `{genotype}`")
    })?;
    Ok((left, right, false))
}
