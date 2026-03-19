//! Selection argument structs for FASTQ planning.
//! Stable knobs here are considered part of the planner's public API.

use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastqPlannerMode {
    Shotgun,
    EdnaAmplicon,
    PollenAmplicon,
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
    pub adapter_bank_preset: Option<String>,
    pub adapter_bank: Option<String>,
    pub adapter_bank_file: Option<PathBuf>,
    pub enable_adapters: Vec<String>,
    pub disable_adapters: Vec<String>,
    pub polyx_preset: Option<String>,
    pub contaminant_preset: Option<String>,
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
    pub polyx_preset: Option<String>,
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
    pub damage_mode: Option<String>,
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
    pub max_n: Option<u32>,
    pub low_complexity_threshold: Option<f64>,
    pub kmer_ref: Option<PathBuf>,
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
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqRemoveChimerasArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
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
}

#[derive(Debug, Clone)]
pub struct BenchFastqInferAsvsArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
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
}

#[derive(Debug, Clone)]
pub struct BenchFastqClusterOtusArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
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
}

#[derive(Debug, Clone)]
pub struct BenchFastqCorrectArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub out: PathBuf,
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
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
}

#[derive(Debug, Clone)]
pub struct BenchFastqUmiArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: PathBuf,
    pub out: PathBuf,
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
    pub tools: Vec<String>,
    pub explain: bool,
    pub replicates: u32,
    pub jobs: u32,
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct BenchFastqDepleteHostArgs {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub out: PathBuf,
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
    pub out: PathBuf,
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
    pub allow_planned: bool,
    pub mode: FastqPlannerMode,
}
