//! call pseudohaploid observations variants.
use crate::call_pseudohaploid::model::PseudohaploidSiteSummary;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use sha2::Digest;
use sha2::Sha256;
use std::path::Path;

pub(crate) fn summarize_pseudohaploid_sites(vcf_path: &Path) -> Result<PseudohaploidSiteSummary> {
    let raw = read_vcf_text(vcf_path)?;
    let mut summary = PseudohaploidSiteSummary { target: 0, covered: 0, called: 0, missing: 0 };
    let mut observed_gt = false;

    for line in raw.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 10 {
            bail!("pseudohaploid smoke output row is missing FORMAT/sample fields: {line}");
        }
        let gt_index = fields[8].split(':').position(|token| token == "GT").ok_or_else(|| {
            anyhow!("pseudohaploid smoke output row is missing GT in FORMAT: {line}")
        })?;
        observed_gt = true;
        summary.target += 1;

        let sample_field = fields[9];
        let gt = sample_field.split(':').nth(gt_index).ok_or_else(|| {
            anyhow!("pseudohaploid smoke sample field is missing GT value: {line}")
        })?;
        parse_pseudohaploid_gt(gt)?;
        summary.covered += 1;
        if gt == "." {
            summary.missing += 1;
        } else {
            summary.called += 1;
        }
    }

    if !observed_gt {
        bail!("pseudohaploid smoke output did not contain any GT fields");
    }
    Ok(summary)
}

pub(crate) fn parse_pseudohaploid_gt(genotype: &str) -> Result<Option<u32>> {
    if genotype == "." {
        return Ok(None);
    }
    if genotype.contains('/') || genotype.contains('|') {
        bail!("pseudohaploid smoke genotype is not haploid-compatible: `{genotype}`");
    }
    let allele = genotype.parse::<u32>().with_context(|| {
        format!("pseudohaploid smoke genotype has non-numeric allele: `{genotype}`")
    })?;
    Ok(Some(allele))
}

pub(crate) fn canonical_vcf_replay_sha256(vcf_path: &Path) -> Result<String> {
    let raw = read_vcf_text(vcf_path)?;
    let canonical = raw
        .lines()
        .filter(|line| {
            !line.starts_with("##bcftoolsCommand=")
                && !line.starts_with("##bcftools_callCommand=")
                && !line.starts_with("##bcftools_viewCommand=")
        })
        .collect::<Vec<_>>()
        .join("\n");
    Ok(sha256_hex(canonical.as_bytes()))
}

pub(crate) fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
