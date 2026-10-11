//! Scientific observation records consumed by metric and report projections.
use serde::Deserialize;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalVcfQcSmokeSampleMissingnessRow {
    pub sample_id: String,
    pub total_genotype_count: u64,
    pub missing_genotype_count: u64,
    pub missingness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalVcfQcSmokeVariantMissingnessRow {
    pub variant_id: String,
    pub contig: String,
    pub position: u64,
    pub reference: String,
    pub alternate: String,
    pub total_sample_count: u64,
    pub missing_sample_count: u64,
    pub missingness: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalVcfQcSmokeMafSummary {
    pub allele_frequency_mean: f64,
    pub maf_bin_counts: BTreeMap<String, u64>,
    pub observed_variant_count: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalVcfQcSmokeHeterozygosity {
    pub heterozygous_call_count: u64,
    pub homozygous_alt_call_count: u64,
    pub het_hom_ratio: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalVcfQcSmokeHweSummary {
    pub tested_variant_count: u64,
    pub pvalue_mean: Option<f64>,
    pub status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct LocalVcfQcSummary {
    pub(crate) sample_missingness: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub(crate) variant_missingness: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub(crate) maf_summary: LocalVcfQcSmokeMafSummary,
    pub(crate) heterozygosity: LocalVcfQcSmokeHeterozygosity,
    pub(crate) hwe_summary: LocalVcfQcSmokeHweSummary,
    pub(crate) excluded_samples: Vec<LocalVcfQcSmokeSampleMissingnessRow>,
    pub(crate) excluded_variants: Vec<LocalVcfQcSmokeVariantMissingnessRow>,
    pub(crate) sample_missingness_exclusion_threshold: f64,
    pub(crate) variant_missingness_exclusion_threshold: f64,
}

#[derive(Debug, Clone)]
pub(crate) struct GovernedQcOutputFile {
    pub(crate) artifact_id: &'static str,
    pub(crate) declared_path: String,
    pub(crate) path: std::path::PathBuf,
    pub(crate) role: &'static str,
}

#[derive(Debug, Clone)]
pub(crate) struct AlleleFrequencyRow {
    pub(crate) variant_id: String,
    pub(crate) contig: String,
    pub(crate) position: u64,
    pub(crate) reference: String,
    pub(crate) alternate: String,
    pub(crate) allele_frequency: f64,
    pub(crate) observed_alleles: u64,
}

#[derive(Debug, Clone)]
pub(crate) struct SampleGenotypeStats {
    pub(crate) sample_id: String,
    pub(crate) nonmissing_genotype_count: u64,
    pub(crate) heterozygous_call_count: u64,
    pub(crate) homozygous_alt_call_count: u64,
    pub(crate) homozygous_ref_call_count: u64,
}
