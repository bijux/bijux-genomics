//! Strict mini-fixture genotype observations over caller-acquired VCF text.
use std::collections::BTreeMap;
use std::path::Path;

mod genotypes;
mod homozygosity;
mod summary;
pub use summary::summarize_fixture_genotypes;

/// Borrowed input whose path is diagnostic metadata, never an acquisition instruction.
#[derive(Debug, Clone, Copy)]
pub struct VcfFixtureGenotypeInput<'a> {
    pub text: &'a str,
    pub source_path: &'a Path,
}

/// Ordered sample observations and last-seen site frequencies for strict mini-fixture truth.
#[derive(Debug, Clone)]
pub struct VcfFixtureGenotypeSummary {
    pub sample_ids: Vec<String>,
    pub sample_count: usize,
    pub variant_count: u64,
    pub contigs: Vec<String>,
    pub genotype_states: BTreeMap<String, VcfFixtureSampleGenotypes>,
    pub allele_frequencies: BTreeMap<(String, u64), VcfFixtureAlleleFrequency>,
}

/// Zygosity, phase and record-ordered observations accumulated for one sample identity.
#[derive(Debug, Clone, Default)]
pub struct VcfFixtureSampleGenotypes {
    pub hom_ref: u64,
    pub het: u64,
    pub hom_alt: u64,
    pub missing: u64,
    pub phased_calls: u64,
    pub unphased_calls: u64,
    pub dosages: Vec<u64>,
    pub contig_is_homozygous: BTreeMap<String, Vec<bool>>,
}

/// Nonzero-allele frequency at one observed site; later duplicate sites replace this row.
#[derive(Debug, Clone)]
pub struct VcfFixtureAlleleFrequency {
    pub alt_allele_count: u64,
    pub called_allele_count: u64,
    pub alt_allele_frequency: f64,
}

#[cfg(test)]
mod contracts;
