use super::model::VcfStatsTruthCounts;
use anyhow::{bail, Result};
use bijux_dna_stages_vcf::vcf_io::read_vcf_text;
use std::path::Path;
pub(super) fn summarize_variant_truth(vcf_path: &Path) -> Result<VcfStatsTruthCounts> {
    let raw = read_vcf_text(vcf_path)?;
    let mut variant_count = 0_u64;
    let mut snp_count = 0_u64;
    let mut indel_count = 0_u64;
    let mut transition_count = 0_u64;
    let mut transversion_count = 0_u64;
    let mut sample_count = 0_u64;

    for line in raw.lines() {
        if line.starts_with("#CHROM\t") {
            sample_count = line.split('\t').skip(9).count() as u64;
            continue;
        }
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('\t').collect::<Vec<_>>();
        if fields.len() < 5 {
            bail!("stats smoke row is malformed: {line}");
        }
        variant_count += 1;
        let ref_allele = fields[3];
        let alt_allele = fields[4];
        if ref_allele.len() == 1 && alt_allele.len() == 1 {
            snp_count += 1;
            if is_transition(ref_allele, alt_allele) {
                transition_count += 1;
            } else {
                transversion_count += 1;
            }
        } else {
            indel_count += 1;
        }
    }

    Ok(VcfStatsTruthCounts {
        variants: variant_count,
        snps: snp_count,
        indels: indel_count,
        transitions: transition_count,
        transversions: transversion_count,
        samples: sample_count,
    })
}
pub(super) fn is_transition(ref_allele: &str, alt_allele: &str) -> bool {
    matches!(
        (ref_allele.as_bytes().first().copied(), alt_allele.as_bytes().first().copied()),
        (Some(b'A'), Some(b'G'))
            | (Some(b'G'), Some(b'A'))
            | (Some(b'C'), Some(b'T'))
            | (Some(b'T'), Some(b'C'))
    )
}
