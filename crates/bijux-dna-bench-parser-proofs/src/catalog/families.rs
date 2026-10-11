use super::{bam::BAM, fastq::FASTQ, vcf::VCF, FamilyProbeSpec};

/// Complete ordered catalog of governed parser proofs.
#[must_use]
pub fn family_probe_specs() -> Vec<FamilyProbeSpec> {
    FASTQ.iter().chain(BAM).chain(VCF).copied().collect()
}
