use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(name = "bijux", version, about = "Bijux DNA CLI")]
pub struct Cli {
    #[arg(long, default_value = "local")]
    pub profile: String,
    #[arg(long)]
    pub platform: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub telemetry_jsonl: Option<PathBuf>,
    #[arg(long, verbatim_doc_comment)]
    /// Print resolved config JSON (skeleton for future defaults).
    pub print_effective_config: bool,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Fastq {
        #[command(subcommand)]
        command: FastqCommand,
    },
    Analyze {
        #[command(subcommand)]
        command: AnalyzeCommand,
    },
    ValidateManifests,
    Platform,
    ImageQa,
    Replay(ReplayArgs),
    Compare(CompareArgs),
    Env {
        #[command(subcommand)]
        command: EnvCommand,
    },
    Bench {
        #[command(subcommand)]
        command: BenchCommand,
    },
}

#[derive(Debug, Args)]
pub struct ReplayArgs {
    pub run_id: String,
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
}

#[derive(Debug, Args)]
pub struct CompareArgs {
    pub run_a: String,
    pub run_b: String,
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum AnalyzeCommand {
    Runs(AnalyzeRunsArgs),
}

#[derive(Debug, Args)]
pub struct AnalyzeRunsArgs {
    #[arg(long, default_value = "runs/bijux-runs/index.jsonl")]
    pub index: PathBuf,
    #[arg(long)]
    pub stage: Option<String>,
    #[arg(long)]
    pub tool: Option<String>,
    #[arg(long, value_enum)]
    pub objective: Option<ObjectiveArg>,
    #[arg(long)]
    pub success: Option<bool>,
}

#[derive(Debug, Subcommand)]
pub enum EnvCommand {
    Images,
    Info,
    Doctor,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum BenchCommand {
    Fastq {
        #[command(subcommand)]
        command: BenchFastqCommand,
    },
    Schema {
        stage: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum BenchFastqCommand {
    Trim(BenchFastqTrimArgs),
    Validate(BenchFastqValidateArgs),
    Filter(BenchFastqFilterArgs),
    Merge(BenchFastqMergeArgs),
    Correct(BenchFastqCorrectArgs),
    #[command(name = "qc-post", alias = "qc2")]
    QcPost(BenchFastqQcPostArgs),
    Umi(BenchFastqUmiArgs),
    Screen(BenchFastqScreenArgs),
    Stats(BenchFastqStatsArgs),
    Preprocess(BenchFastqPreprocessArgs),
}

#[derive(Debug, Args)]
pub struct BenchFastqTrimArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long, help = "Adapter bank preset name (default: ancientdna-illumina)")]
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
}

#[derive(Debug, Args)]
pub struct BenchFastqValidateArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long)]
    pub strict: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
    #[arg(long)]
    pub max_n: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
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
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
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
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqQcPostArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqUmiArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqScreenArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqStatsArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub explain: bool,
}

#[derive(Debug, Args)]
pub struct BenchFastqPreprocessArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long)]
    pub strict: bool,
    #[arg(long, help = "Adapter bank preset name (default: ancientdna-illumina)")]
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
    pub no_qc_post: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ObjectiveArg {
    Speed,
    Memory,
    Retention,
    Balanced,
}

impl ObjectiveArg {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            ObjectiveArg::Speed => "speed",
            ObjectiveArg::Memory => "memory",
            ObjectiveArg::Retention => "retention",
            ObjectiveArg::Balanced => "balanced",
        }
    }
}

impl From<ObjectiveArg> for bijux_core::selection::Objective {
    fn from(value: ObjectiveArg) -> Self {
        match value {
            ObjectiveArg::Speed => bijux_core::selection::Objective::Speed,
            ObjectiveArg::Memory => bijux_core::selection::Objective::Memory,
            ObjectiveArg::Retention => bijux_core::selection::Objective::Retention,
            ObjectiveArg::Balanced => bijux_core::selection::Objective::Balanced,
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchCorpusArg {
    #[value(name = "fastq_5set")]
    Fastq5Set,
}

impl From<BenchCorpusArg> for bijux_stages_fastq::BenchCorpusId {
    fn from(value: BenchCorpusArg) -> Self {
        match value {
            BenchCorpusArg::Fastq5Set => bijux_stages_fastq::BenchCorpusId::Fastq5Set,
        }
    }
}

#[derive(Debug, Args, Clone, Default)]
pub struct CommonArgs {
    #[arg(long)]
    pub list_tools: bool,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Args, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct FastqPreprocessArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub list_adapter_presets: bool,
    #[arg(long)]
    pub list_adapters: bool,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub strict: bool,
    #[arg(long)]
    pub auto: bool,
    #[arg(long, value_enum, default_value_t = ObjectiveArg::Balanced)]
    pub objective: ObjectiveArg,
    #[arg(long, value_enum)]
    pub bench_corpus: Option<BenchCorpusArg>,
    #[arg(long)]
    pub allow_partial: bool,
    #[arg(long, help = "Adapter bank preset name (default: ancientdna-illumina)")]
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
    pub no_qc_post: bool,
}

#[derive(Debug, Args, Clone)]
pub struct FastqBenchmarkArgs {
    #[arg(long, default_value = "runs")]
    pub runs: PathBuf,
    #[arg(long)]
    pub stage: String,
    #[arg(long, value_enum, default_value_t = ObjectiveArg::Balanced)]
    pub objective: ObjectiveArg,
}

#[derive(Debug, Args, Clone)]
pub struct FastqRunArgs {
    #[command(flatten)]
    pub args: FastqPreprocessArgs,
}

#[derive(Debug, Args, Clone)]
pub struct FastqCompareArgs {
    #[arg(long)]
    pub run_a: String,
    #[arg(long)]
    pub run_b: String,
    #[arg(long, default_value = "runs")]
    pub search_root: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum FastqCommand {
    #[command(about = "List FASTQ stages.")]
    ListStages,
    #[command(about = "List FASTQ stage ids and versions.")]
    Stages,
    #[command(about = "List tools for a FASTQ stage.")]
    ListTools {
        #[arg(long)]
        stage: String,
    },
    #[command(about = "Explain a FASTQ stage or pipeline.")]
    Explain {
        stage: String,
    },
    #[command(
        about = "Filter FASTQ reads.",
        after_help = "Examples:\n  bijux fastq filter --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastp\n  bijux fastq filter --list-tools"
    )]
    Filter(FastqFilterArgs),
    #[command(
        about = "Merge paired-end FASTQ reads.",
        after_help = "Example:\n  bijux fastq merge --r1 reads_1.fastq.gz --r2 reads_2.fastq.gz --out artifacts --sample-id SAMPLE --tools vsearch\n\nNext stages: filter -> stats"
    )]
    Merge(CommonArgs),
    #[command(
        about = "Trim FASTQ reads (quality/adapters) and emit canonical outputs.",
        after_help = "Example:\n  bijux fastq trim --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastp\n\nNext stages: filter -> stats"
    )]
    Trim(FastqTrimArgs),
    Contam(CommonArgs),
    #[command(
        about = "Run the FASTQ preprocess pipeline (validate → trim → filter → stats).",
        after_help = "Examples:\n  bijux fastq preprocess --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq preprocess --auto --objective speed --bench-corpus fastq_5set --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq preprocess --list-tools"
    )]
    Preprocess(FastqPreprocessArgs),
    #[command(
        about = "Run the FASTQ pipeline (validate → trim → filter → stats).",
        after_help = "Examples:\n  bijux fastq run --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq run --auto --objective speed --bench-corpus fastq_5set --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE"
    )]
    Run(FastqRunArgs),
    #[command(
        name = "stats-neutral",
        alias = "stats",
        about = "Summarize FASTQ read statistics (neutral).",
        after_help = "Example:\n  bijux fastq stats-neutral --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools seqkit_stats\n\nNext stages: report/compare"
    )]
    StatsNeutral(CommonArgs),
    Umi(CommonArgs),
    #[command(name = "error-correct")]
    ErrorCorrect(CommonArgs),
    Qc(CommonArgs),
    #[command(
        name = "validate-pre",
        alias = "validate",
        about = "Validate FASTQ reads (pre).",
        after_help = "Examples:\n  bijux fastq validate-pre --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastqvalidator_official\n  bijux fastq validate-pre --list-tools"
    )]
    ValidatePre(FastqValidateArgs),
    #[command(about = "Benchmark existing FASTQ runs without re-execution.")]
    Benchmark(FastqBenchmarkArgs),
    #[command(about = "Analyze FASTQ runs without re-execution.")]
    Analyze(FastqBenchmarkArgs),
    #[command(about = "Compare two FASTQ runs.")]
    Compare(FastqCompareArgs),
    Align(CommonArgs),
}

#[derive(Debug, Args, Clone)]
pub struct FastqTrimArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub list_adapter_presets: bool,
    #[arg(long)]
    pub list_adapters: bool,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long, help = "Adapter bank preset name (default: ancientdna-illumina)")]
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
}

#[derive(Debug, Args, Clone)]
pub struct FastqFilterArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub max_n: Option<u32>,
    #[arg(long = "low-complexity-threshold")]
    pub low_complexity_threshold: Option<f64>,
    #[arg(long)]
    pub kmer_ref: Option<PathBuf>,
}

#[derive(Debug, Args, Clone)]
pub struct FastqValidateArgs {
    #[command(flatten)]
    pub common: CommonArgs,
    #[arg(long)]
    pub env: Option<String>,
    #[arg(long, alias = "sample")]
    pub sample_id: Option<String>,
    #[arg(long)]
    pub r1: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long, value_delimiter = ',')]
    pub tools: Vec<String>,
    #[arg(long)]
    pub strict: bool,
}
