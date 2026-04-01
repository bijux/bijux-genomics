use std::path::PathBuf;

use clap::Args;

#[derive(Debug, Args)]
pub struct BenchFastqMergeArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub merge_overlap: Option<u32>,
    #[arg(long)]
    pub min_length: Option<u32>,
    #[arg(long, help = "emit_unmerged_pairs | omit_unmerged_pairs")]
    pub unmerged_read_policy: Option<String>,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqRemoveChimerasArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqNormalizePrimersArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(
        long,
        help = "Primer governance set id (for example: 16S_universal_v1)"
    )]
    pub primer_set_id: Option<String>,
    #[arg(
        long,
        help = "Primer orientation policy (for example: normalize_to_forward_primer)"
    )]
    pub orientation_policy: Option<String>,
    #[arg(
        long,
        help = "Maximum primer mismatch rate admitted by the governed runtime"
    )]
    pub max_mismatch_rate: Option<f64>,
    #[arg(long, help = "Minimum primer overlap in base pairs")]
    pub min_overlap_bp: Option<u32>,
    #[arg(long, help = "Require a strict 5' primer anchor")]
    pub strict_5p_anchor: Option<bool>,
    #[arg(long, help = "Allow IUPAC ambiguity codes in governed primer matching")]
    pub allow_iupac_codes: Option<bool>,
}

#[derive(Debug, Args)]
pub struct BenchFastqInferAsvsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Denoising backend contract: dada2")]
    pub denoising_method: Option<String>,
    #[arg(long, help = "Pooling mode: independent | pseudo_pool | pooled")]
    pub pooling_mode: Option<String>,
    #[arg(long, help = "Chimera policy: remove_bimera_denovo | keep_candidates")]
    pub chimera_policy: Option<String>,
    #[arg(long, help = "Thread count for the governed ASV backend")]
    pub threads: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqNormalizeAbundanceArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub table: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(
        long,
        help = "Normalization method: relative_abundance | counts_per_million"
    )]
    pub method: Option<String>,
}

#[derive(Debug, Args)]
pub struct BenchFastqUmiArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        default_value = "NNNNNNNN",
        help = "UMI barcode pattern passed to umi_tools extract"
    )]
    pub umi_pattern: String,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqClusterOtusArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Set the governed OTU identity threshold")]
    pub otu_identity: Option<f64>,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqScreenArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, help = "Set the governed taxonomy database root")]
    pub database_root: Option<PathBuf>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqIndexReferenceArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub reference_fasta: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqDepleteHostArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub reference_index: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long, help = "Set the governed host identity threshold")]
    pub host_identity_threshold: Option<f64>,
    #[arg(long, help = "Choose whether only unmapped reads are retained")]
    pub retain_unmapped_only: Option<bool>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqDepleteReferenceContaminantsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub reference_index: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long, help = "Set the governed contaminant decoy mode")]
    pub decoy_mode: Option<String>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqDepleteRrnaArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long, help = "Set the governed rRNA reference selector")]
    pub rrna_db: Option<String>,
    #[arg(long, help = "Set the governed minimum identity threshold")]
    pub min_identity: Option<f64>,
    #[arg(
        long,
        value_delimiter = ',',
        default_value = "auto",
        help = "Tool selection: auto | all | <csv>"
    )]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqPreprocessArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long, help = "Pipeline profile id (default, minimal)")]
    pub pipeline_profile: Option<String>,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Reference FASTA for reference-guided FASTQ stages"
    )]
    pub reference_fasta: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub strict: bool,
    #[arg(long, help = "Allow experimental and silver-tier tools")]
    pub allow_experimental: bool,
    #[arg(long, default_value_t = 1)]
    pub replicates: u32,
    #[arg(long, default_value_t = 1)]
    pub jobs: u32,
    #[arg(long)]
    pub ci_bootstrap: Option<u32>,
    #[arg(long, help = "Adapter bank preset name (default: illumina-default)")]
    pub adapter_bank_preset: Option<String>,
    #[arg(
        long,
        help = "Adapter bank selection: preset:<name> (deprecated; use --adapter-bank-preset)"
    )]
    pub adapter_bank: Option<String>,
    #[arg(long, help = "Adapter bank file (yaml/json)")]
    pub adapter_bank_file: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
    #[arg(long, help = "PolyX preset name (default: illumina_twocolor)")]
    pub polyx_preset: Option<String>,
    #[arg(long, help = "Contaminant preset name (default: illumina_default)")]
    pub contaminant_preset: Option<String>,
    #[arg(
        long,
        help = "Enable contaminant k-mer removal when contaminant preset is set."
    )]
    pub enable_contaminant_removal: bool,
    #[arg(long)]
    pub no_qc_post: bool,
    #[arg(long)]
    pub force_merge: bool,
    #[arg(long, help = "Enable error correction stage")]
    pub enable_correct: bool,
    #[arg(
        long,
        help = "Expand each preprocess stage into all governed runtime tools"
    )]
    pub run_all_governed_tools: bool,
    #[arg(long, help = "Allow planned/out-of-scope stages in planning")]
    pub allow_planned: bool,
}
