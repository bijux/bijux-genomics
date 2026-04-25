//! Selection argument structs for FASTQ planning.
//! Stable knobs here are considered part of the planner's public API.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastqPlannerMode {
    Shotgun,
    ShotgunStandard,
    ShotgunAdna,
    HostAssociatedMetagenome,
    PremergedSingleEnd,
    AmpliconStandard,
    AmpliconUmi,
    EdnaAmplicon,
    PollenAmplicon,
}

impl FastqPlannerMode {
    #[must_use]
    pub const fn route_family(self) -> &'static str {
        match self {
            Self::Shotgun | Self::ShotgunStandard => "shotgun_standard",
            Self::ShotgunAdna => "shotgun_adna",
            Self::HostAssociatedMetagenome => "host_associated_metagenome",
            Self::PremergedSingleEnd => "premerged_single_end",
            Self::AmpliconStandard | Self::EdnaAmplicon | Self::PollenAmplicon => {
                "amplicon_standard"
            }
            Self::AmpliconUmi => "amplicon_umi",
        }
    }

    #[must_use]
    pub const fn is_shotgun_family(self) -> bool {
        matches!(
            self,
            Self::Shotgun
                | Self::ShotgunStandard
                | Self::ShotgunAdna
                | Self::HostAssociatedMetagenome
                | Self::PremergedSingleEnd
        )
    }

    #[must_use]
    pub const fn pipeline_mode(self) -> bijux_dna_domain_fastq::FastqPipelineMode {
        if self.is_shotgun_family() {
            bijux_dna_domain_fastq::FastqPipelineMode::Shotgun
        } else {
            bijux_dna_domain_fastq::FastqPipelineMode::Amplicon
        }
    }

    #[must_use]
    pub const fn admits_terminal_damage_trim(self) -> bool {
        matches!(self, Self::ShotgunAdna)
    }
}

#[derive(Debug, Clone)]
pub struct BenchFastqTrimArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub adapter_bank_preset: Option<String>,
    pub adapter_bank: Option<String>,
    pub adapter_bank_file: Option<PathBuf>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    pub polyx_preset: Option<String>,
    pub contaminant_preset: Option<String>,
    pub min_length: Option<u32>,
    pub quality_cutoff: Option<u32>,
    pub n_policy: Option<String>,
    pub adapter_policy: Option<String>,
    pub polyx_policy: Option<String>,
    pub contaminant_policy: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqTrimPolygArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub trim_polyg: Option<bool>,
    pub polyx_preset: Option<String>,
    pub min_polyg_run: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqTrimTerminalDamageArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub damage_mode: Option<String>,
    pub execution_policy: Option<String>,
    pub trim_5p_bases: Option<u32>,
    pub trim_3p_bases: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqValidateArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub strict: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub validation_mode: Option<String>,
    pub pair_sync_policy: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqDetectAdaptersArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqProfileReadLengthsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub histogram_bins: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqFilterArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub max_n: Option<u32>,
    pub max_n_fraction: Option<f64>,
    pub max_n_count: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub entropy_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
    pub polyx_policy: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqFilterLowComplexityArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub entropy_threshold: Option<f64>,
    pub polyx_threshold: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqMergeArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub threads: Option<u32>,
    pub merge_overlap: Option<u32>,
    pub min_length: Option<u32>,
    pub unmerged_read_policy: Option<String>,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqRemoveDuplicatesArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub tools_resolved_implicitly: bool,
    pub explain: bool,
    pub threads: Option<u32>,
    pub dedup_mode: Option<String>,
    pub keep_order: Option<bool>,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqRemoveChimerasArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqNormalizePrimersArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub primer_set_id: Option<String>,
    pub orientation_policy: Option<String>,
    pub max_mismatch_rate: Option<f64>,
    pub min_overlap_bp: Option<u32>,
    pub strict_5p_anchor: Option<bool>,
    pub allow_iupac_codes: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqInferAsvsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub denoising_method: Option<String>,
    pub pooling_mode: Option<String>,
    pub chimera_policy: Option<String>,
    pub threads: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqNormalizeAbundanceArgs {
    pub sample_id: String,
    pub table: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub method: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqClusterOtusArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub otu_identity: Option<f64>,
    pub threads: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqIndexReferenceArgs {
    pub sample_id: String,
    pub reference_fasta: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqCorrectArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
    pub quality_encoding: Option<String>,
    pub kmer_size: Option<u32>,
    pub musket_kmer_budget: Option<u64>,
    pub genome_size: Option<u64>,
    pub max_memory_gb: Option<u32>,
    pub trusted_kmer_artifact: Option<PathBuf>,
    pub conservative_mode: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqQcPostArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub aggregation_engine: Option<String>,
    pub aggregation_scope: Option<String>,
    pub governed_qc_manifest: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqUmiArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub out: PathBuf,
    pub umi_pattern: String,
    pub threads: Option<u32>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqScreenArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub database_root: Option<PathBuf>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub threads: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqDepleteHostArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_index: PathBuf,
    pub out: PathBuf,
    pub threads: Option<u32>,
    pub host_identity_threshold: Option<f64>,
    pub retain_unmapped_only: Option<bool>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqDepleteReferenceContaminantsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_index: PathBuf,
    pub out: PathBuf,
    pub threads: Option<u32>,
    pub decoy_mode: Option<String>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqDepleteRrnaArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub threads: Option<u32>,
    pub rrna_db: Option<String>,
    pub min_identity: Option<f64>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqStatsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub threads: Option<u32>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqProfileOverrepresentedArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
    pub threads: Option<u32>,
    pub top_k: Option<u32>,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct BenchFastqPreprocessArgs {
    pub sample_id: String,
    pub profile: Option<String>,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub reference_fasta: Option<PathBuf>,
    pub out: PathBuf,
    pub strict: bool,
    pub auto: bool,
    pub objective: bijux_dna_core::contract::Objective,
    pub bench_corpus: Option<bijux_dna_domain_fastq::BenchCorpusId>,
    pub allow_partial: bool,
    pub dry_run: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
    pub adapter_bank_preset: Option<String>,
    pub adapter_bank: Option<String>,
    pub adapter_bank_file: Option<PathBuf>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    pub polyx_preset: Option<String>,
    pub contaminant_preset: Option<String>,
    pub enable_contaminant_removal: bool,
    pub no_qc_post: bool,
    pub force_merge: bool,
    pub enable_correct: bool,
    pub run_all_governed_tools: bool,
    pub allow_planned: bool,
    pub mode: FastqPlannerMode,
}
