use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
pub struct LocalBamGenotypingConfig {
    pub tool_id: String,
    pub bam: PathBuf,
    pub reference_fasta: PathBuf,
    pub sites_vcf: PathBuf,
    pub regions: PathBuf,
    pub sample_id: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalBamKinshipConfig {
    pub tool_id: String,
    pub cases: Vec<LocalBamKinshipCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalBamKinshipCase {
    pub bam: PathBuf,
    pub reference_panel: String,
    pub reference_build: String,
    pub population_scope: String,
    pub min_overlap_snps: u32,
    pub requires_cohort_context: bool,
    pub expected_status: String,
}

#[derive(Debug, Deserialize)]
pub struct LocalBamSexConfig {
    pub tool_id: String,
    pub cases: Vec<LocalBamSexCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct LocalBamSexCase {
    pub bam: PathBuf,
    pub reference: PathBuf,
    pub chromosome_system: String,
    pub minimum_y_sites: u32,
}

/// Published synthetic parser evidence and its declared tool identity.
#[derive(Debug, Clone)]
pub struct MaterializedProof {
    pub proof_path: PathBuf,
    pub observed_tool_id: String,
}

/// Explicit alignment inputs projected from an admitted canonical stage plan.
pub struct AlignmentProbe<'a> {
    pub tool_id: &'a str,
    pub reference_fasta: &'a Path,
    pub input_r1: &'a Path,
    pub input_r2: Option<&'a Path>,
    pub sample_id: &'a str,
    pub read_group: &'a bijux_dna_domain_bam::params::ReadGroupSpec,
    pub sensitivity_profile: Option<&'a str>,
    pub preset: Option<&'a str>,
    pub seed_length: Option<u32>,
}

/// Declared contamination parser fixture parameters; estimates remain synthetic.
pub struct ContaminationProbe<'a> {
    pub tool_id: &'a str,
    pub scope: bijux_dna_domain_bam::params::ContaminationScope,
    pub assumptions: &'a str,
}

/// Declared terminal-damage observations consumed by tiny BAM proof summaries.
pub struct DamageProbe<'a> {
    pub tool_id: &'a str,
    pub bam: &'a Path,
    pub c_to_t_5p: f64,
    pub g_to_a_3p: f64,
    pub strict_profile_upgraded: bool,
}
