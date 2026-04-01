use std::path::PathBuf;

use clap::{Args, Subcommand};

use super::BenchBamCommand;

mod config;
mod corpus_fastq;
mod publication;

pub use self::config::{
    BenchConfigCommand, BenchConfigJsonArgs, BenchConfigValidateArgs,
    BenchNormalizeWorkspaceLayoutArgs, BenchRepoChecksArgs, BenchWorkspaceValueArgs,
    BenchWriteScreenTaxonomyDatabaseLineageArgs,
};
pub use self::corpus_fastq::BenchCorpusFastqArgs;
pub use self::publication::{
    BenchCorpusFastqPublicationStatusArgs, BenchCorpusFastqPublishedDossiersArgs,
    BenchCorpusFastqReportArgs, BenchPublicationTargetsArgs,
};

#[derive(Debug, Subcommand)]
pub enum BenchCommand {
    Config {
        #[command(subcommand)]
        command: BenchConfigCommand,
    },
    Run(BenchRunArgs),
    Status,
    #[command(name = "workspace-value")]
    WorkspaceValue(BenchWorkspaceValueArgs),
    #[command(name = "config-json")]
    ConfigJson(BenchConfigJsonArgs),
    #[command(name = "repo-checks")]
    RepoChecks(BenchRepoChecksArgs),
    #[command(name = "write-screen-taxonomy-database-lineage")]
    WriteScreenTaxonomyDatabaseLineage(BenchWriteScreenTaxonomyDatabaseLineageArgs),
    #[command(name = "publication-targets")]
    PublicationTargets(BenchPublicationTargetsArgs),
    #[command(name = "corpus-fastq")]
    CorpusFastq(BenchCorpusFastqArgs),
    #[command(name = "normalize-workspace-layout")]
    NormalizeWorkspaceLayout(BenchNormalizeWorkspaceLayoutArgs),
    #[command(name = "corpus-fastq-report")]
    CorpusFastqReport(BenchCorpusFastqReportArgs),
    #[command(name = "corpus-fastq-publication-status")]
    CorpusFastqPublicationStatus(BenchCorpusFastqPublicationStatusArgs),
    #[command(name = "corpus-fastq-published-dossiers")]
    CorpusFastqPublishedDossiers(BenchCorpusFastqPublishedDossiersArgs),
    Fastq {
        #[command(subcommand)]
        command: BenchFastqCommand,
    },
    Bam {
        #[command(subcommand)]
        command: BenchBamCommand,
    },
    Schema {
        stage: String,
    },
}

#[derive(Debug, Args)]
pub struct BenchRunArgs {
    #[arg(long)]
    pub suite: String,
    #[arg(long, default_value_t = false)]
    pub hpc: bool,
}

#[derive(Debug, Subcommand)]
pub enum BenchFastqCommand {
    #[command(name = "trim-reads", visible_alias = "trim")]
    Trim(BenchFastqTrimArgs),
    #[command(name = "trim-polyg-tails")]
    TrimPolygTails(BenchFastqTrimPolygArgs),
    #[command(name = "trim-terminal-damage")]
    TrimTerminalDamage(BenchFastqTrimTerminalDamageArgs),
    #[command(name = "validate-reads", visible_alias = "validate")]
    Validate(BenchFastqValidateArgs),
    #[command(name = "detect-adapters")]
    DetectAdapters(BenchFastqDetectAdaptersArgs),
    #[command(name = "profile-read-lengths")]
    ProfileReadLengths(BenchFastqProfileReadLengthsArgs),
    Filter(BenchFastqFilterArgs),
    #[command(name = "filter-low-complexity")]
    FilterLowComplexity(BenchFastqFilterLowComplexityArgs),
    Merge(BenchFastqMergeArgs),
    #[command(name = "remove-duplicates")]
    RemoveDuplicates(BenchFastqRemoveDuplicatesArgs),
    #[command(name = "remove-chimeras")]
    RemoveChimeras(BenchFastqRemoveChimerasArgs),
    #[command(name = "normalize-primers")]
    NormalizePrimers(BenchFastqNormalizePrimersArgs),
    #[command(name = "infer-asvs")]
    InferAsvs(BenchFastqInferAsvsArgs),
    #[command(name = "cluster-otus")]
    ClusterOtus(BenchFastqClusterOtusArgs),
    #[command(name = "normalize-abundance")]
    NormalizeAbundance(BenchFastqNormalizeAbundanceArgs),
    Correct(BenchFastqCorrectArgs),
    #[command(name = "report-qc", visible_alias = "qc-post", alias = "qc2")]
    ReportQc(BenchFastqQcPostArgs),
    Umi(BenchFastqUmiArgs),
    #[command(name = "index-reference")]
    IndexReference(BenchFastqIndexReferenceArgs),
    #[command(name = "screen-taxonomy", visible_alias = "screen")]
    Screen(BenchFastqScreenArgs),
    #[command(name = "deplete-host")]
    DepleteHost(BenchFastqDepleteHostArgs),
    #[command(name = "deplete-reference-contaminants")]
    DepleteReferenceContaminants(BenchFastqDepleteReferenceContaminantsArgs),
    #[command(name = "deplete-rrna")]
    DepleteRrna(BenchFastqDepleteRrnaArgs),
    #[command(name = "profile-reads", visible_alias = "stats")]
    Stats(BenchFastqStatsArgs),
    #[command(
        name = "profile-overrepresented-sequences",
        visible_alias = "overrepresented"
    )]
    ProfileOverrepresentedSequences(BenchFastqProfileOverrepresentedArgs),
    Preprocess(BenchFastqPreprocessArgs),
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimArgs {
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
    #[arg(long)]
    pub threads: Option<u32>,
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
    #[arg(long)]
    pub min_length: Option<u32>,
    #[arg(long)]
    pub quality_cutoff: Option<u32>,
    #[arg(long)]
    pub n_policy: Option<String>,
    #[arg(long)]
    pub adapter_policy: Option<String>,
    #[arg(long)]
    pub polyx_policy: Option<String>,
    #[arg(long)]
    pub contaminant_policy: Option<String>,
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimPolygArgs {
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
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub trim_polyg: Option<bool>,
    #[arg(long, help = "PolyX preset name (default: illumina_twocolor)")]
    pub polyx_preset: Option<String>,
    #[arg(long)]
    pub min_polyg_run: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimTerminalDamageArgs {
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
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub damage_mode: Option<String>,
    #[arg(
        long,
        help = "Execution policy: policy_derived | explicit_terminal_trim | preserve_udg_trimmed_ends"
    )]
    pub execution_policy: Option<String>,
    #[arg(long)]
    pub trim_5p_bases: Option<u32>,
    #[arg(long)]
    pub trim_3p_bases: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqValidateArgs {
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
    #[arg(long)]
    pub strict: bool,
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub validation_mode: Option<String>,
    #[arg(long)]
    pub pair_sync_policy: Option<String>,
}

#[derive(Debug, Args)]
pub struct BenchFastqDetectAdaptersArgs {
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
    #[arg(long)]
    pub threads: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqProfileReadLengthsArgs {
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
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long)]
    pub histogram_bins: Option<u32>,
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterArgs {
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
    #[arg(long)]
    pub threads: Option<u32>,
    #[arg(long)]
    pub max_n: Option<u32>,
    #[arg(long)]
    pub max_n_fraction: Option<f64>,
    #[arg(long)]
    pub max_n_count: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub entropy_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
    #[arg(long)]
    pub polyx_policy: Option<String>,
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterLowComplexityArgs {
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
    #[arg(long)]
    pub entropy_threshold: Option<f64>,
    #[arg(long)]
    pub polyx_threshold: Option<u32>,
}

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
pub struct BenchFastqRemoveDuplicatesArgs {
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
    #[arg(long)]
    pub dedup_mode: Option<String>,
    #[arg(long)]
    pub keep_order: Option<bool>,
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
pub struct BenchFastqCorrectArgs {
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
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(long)]
    pub quality_encoding: Option<String>,
    #[arg(long)]
    pub kmer_size: Option<u32>,
    #[arg(long)]
    pub musket_kmer_budget: Option<u64>,
    #[arg(long)]
    pub genome_size: Option<u64>,
    #[arg(long)]
    pub max_memory_gb: Option<u32>,
    #[arg(long)]
    pub trusted_kmer_artifact: Option<PathBuf>,
    #[arg(long)]
    pub conservative_mode: Option<bool>,
}

#[derive(Debug, Args)]
pub struct BenchFastqQcPostArgs {
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
        default_value = "auto",
        help = "Aggregation backend: auto | multiqc"
    )]
    pub aggregation_engine: Option<String>,
    #[arg(long)]
    pub aggregation_scope: Option<String>,
    #[arg(
        long,
        value_name = "PATH",
        help = "Governed QC artifact manifest consumed by fastq.report_qc; required because report_qc aggregates upstream QC outputs and does not regenerate them from raw FASTQ inputs"
    )]
    pub governed_qc_manifest: Option<PathBuf>,
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
pub struct BenchFastqStatsArgs {
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
pub struct BenchFastqProfileOverrepresentedArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, help = "Set governed stage threads before per-job scaling")]
    pub threads: Option<u32>,
    #[arg(
        long,
        help = "Maximum number of ranked sequences to retain in governed outputs"
    )]
    pub top_k: Option<u32>,
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
