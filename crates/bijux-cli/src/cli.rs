use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, ToolId};
use bijux_environment::api::RunnerKind;
use bijux_stages_fastq::args as engine_args;
use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "bijux", version, about = "Bijux DNA CLI")]
pub struct Cli {
    #[arg(long, default_value = "local")]
    pub profile: String,
    #[arg(long)]
    pub platform: Option<String>,
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
    #[arg(long, value_enum, default_value_t = AdapterPresetArg::DefaultAdna)]
    pub adapter_preset: AdapterPresetArg,
    #[arg(long)]
    pub adapter_bank: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
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
    #[arg(long, value_enum, default_value_t = AdapterPresetArg::DefaultAdna)]
    pub adapter_preset: AdapterPresetArg,
    #[arg(long)]
    pub adapter_bank: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
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

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum AdapterPresetArg {
    #[value(name = "default_adna")]
    DefaultAdna,
    #[value(name = "truseq_only")]
    TruseqOnly,
    #[value(name = "ssdna")]
    Ssdna,
    #[value(name = "umi")]
    Umi,
    #[value(name = "nextera")]
    Nextera,
    #[value(name = "capture")]
    Capture,
}

impl AdapterPresetArg {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            AdapterPresetArg::DefaultAdna => "default_adna",
            AdapterPresetArg::TruseqOnly => "truseq_only",
            AdapterPresetArg::Ssdna => "ssdna",
            AdapterPresetArg::Umi => "umi",
            AdapterPresetArg::Nextera => "nextera",
            AdapterPresetArg::Capture => "capture",
        }
    }
}

impl From<ObjectiveArg> for bijux_analyze::selection::Objective {
    fn from(value: ObjectiveArg) -> Self {
        match value {
            ObjectiveArg::Speed => bijux_analyze::selection::Objective::Speed,
            ObjectiveArg::Memory => bijux_analyze::selection::Objective::Memory,
            ObjectiveArg::Retention => bijux_analyze::selection::Objective::Retention,
            ObjectiveArg::Balanced => bijux_analyze::selection::Objective::Balanced,
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
    #[arg(long, value_enum, default_value_t = AdapterPresetArg::DefaultAdna)]
    pub adapter_preset: AdapterPresetArg,
    #[arg(long)]
    pub adapter_bank: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
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
    Filter(CommonArgs),
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
    #[arg(long, value_enum, default_value_t = AdapterPresetArg::DefaultAdna)]
    pub adapter_preset: AdapterPresetArg,
    #[arg(long)]
    pub adapter_bank: Option<PathBuf>,
    #[arg(long)]
    pub enable_adapter: Vec<String>,
    #[arg(long)]
    pub disable_adapter: Vec<String>,
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

#[must_use]
pub fn resolve_stage_tool(command: &Commands) -> (StageId, ToolId, CommonArgs) {
    match command {
        Commands::Fastq { command } => match command {
            FastqCommand::ListStages
            | FastqCommand::Stages
            | FastqCommand::ListTools { .. }
            | FastqCommand::Explain { .. }
            | FastqCommand::Benchmark(_)
            | FastqCommand::Analyze(_)
            | FastqCommand::Compare(_)
            | FastqCommand::Run(_) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                CommonArgs::default(),
            ),
            FastqCommand::Trim(args) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                args.common.clone(),
            ),
            FastqCommand::ValidatePre(args) => (
                StageId("fastq.validate_pre".to_string()),
                ToolId("fastqvalidator".to_string()),
                args.common.clone(),
            ),
            FastqCommand::StatsNeutral(common) => (
                StageId("fastq.stats_neutral".to_string()),
                ToolId("seqkit_stats".to_string()),
                common.clone(),
            ),
            FastqCommand::Filter(common)
            | FastqCommand::Merge(common)
            | FastqCommand::Contam(common)
            | FastqCommand::Umi(common)
            | FastqCommand::ErrorCorrect(common)
            | FastqCommand::Qc(common)
            | FastqCommand::Align(common) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                common.clone(),
            ),
            FastqCommand::Preprocess(args) => (
                StageId("fastq.preprocess".to_string()),
                ToolId("fastp".to_string()),
                args.common.clone(),
            ),
        },
        _ => (
            StageId("fastq.trim".to_string()),
            ToolId("fastp".to_string()),
            CommonArgs::default(),
        ),
    }
}

#[must_use]
pub fn is_bench_requested_trim(args: &FastqTrimArgs) -> bool {
    args.sample_id.is_some() && args.r1.is_some() && args.out.is_some()
}

/// # Errors
/// Returns an error if CLI arguments are invalid for benchmarking.
pub fn bench_args_from_trim(args: &FastqTrimArgs) -> Result<engine_args::BenchFastqTrimArgs> {
    Ok(engine_args::BenchFastqTrimArgs {
        sample_id: args
            .sample_id
            .clone()
            .ok_or_else(|| anyhow!("sample_id required for benchmark"))?,
        r1: args
            .r1
            .clone()
            .ok_or_else(|| anyhow!("r1 required for benchmark"))?,
        out: args
            .out
            .clone()
            .ok_or_else(|| anyhow!("out required for benchmark"))?,
        tools: args.tools.clone(),
        explain: false,
        adapter_preset: args.adapter_preset.as_str().to_string(),
        adapter_bank: args.adapter_bank.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
    })
}

#[must_use]
pub fn is_bench_requested_validate(args: &FastqValidateArgs) -> bool {
    args.sample_id.is_some() && args.r1.is_some() && args.out.is_some()
}

/// # Errors
/// Returns an error if CLI arguments are invalid for benchmarking.
pub fn bench_args_from_validate(
    args: &FastqValidateArgs,
) -> Result<engine_args::BenchFastqValidateArgs> {
    Ok(engine_args::BenchFastqValidateArgs {
        sample_id: args
            .sample_id
            .clone()
            .ok_or_else(|| anyhow!("sample_id required for benchmark"))?,
        r1: args
            .r1
            .clone()
            .ok_or_else(|| anyhow!("r1 required for benchmark"))?,
        out: args
            .out
            .clone()
            .ok_or_else(|| anyhow!("out required for benchmark"))?,
        tools: args.tools.clone(),
        explain: false,
        strict: args.strict,
    })
}

#[must_use]
pub fn bench_args_trim(args: &BenchFastqTrimArgs) -> engine_args::BenchFastqTrimArgs {
    engine_args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        adapter_preset: args.adapter_preset.as_str().to_string(),
        adapter_bank: args.adapter_bank.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
    }
}

#[must_use]
pub fn bench_args_validate(args: &BenchFastqValidateArgs) -> engine_args::BenchFastqValidateArgs {
    engine_args::BenchFastqValidateArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
        strict: args.strict,
    }
}

#[must_use]
pub fn bench_args_filter(args: &BenchFastqFilterArgs) -> engine_args::BenchFastqFilterArgs {
    engine_args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_merge(args: &BenchFastqMergeArgs) -> engine_args::BenchFastqMergeArgs {
    engine_args::BenchFastqMergeArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_correct(args: &BenchFastqCorrectArgs) -> engine_args::BenchFastqCorrectArgs {
    engine_args::BenchFastqCorrectArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_qc_post(args: &BenchFastqQcPostArgs) -> engine_args::BenchFastqQcPostArgs {
    engine_args::BenchFastqQcPostArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_umi(args: &BenchFastqUmiArgs) -> engine_args::BenchFastqUmiArgs {
    engine_args::BenchFastqUmiArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_screen(args: &BenchFastqScreenArgs) -> engine_args::BenchFastqScreenArgs {
    engine_args::BenchFastqScreenArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_stats(args: &BenchFastqStatsArgs) -> engine_args::BenchFastqStatsArgs {
    engine_args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

#[must_use]
pub fn bench_args_preprocess(
    args: &BenchFastqPreprocessArgs,
) -> engine_args::BenchFastqPreprocessArgs {
    engine_args::BenchFastqPreprocessArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        strict: args.strict,
        auto: false,
        objective: bijux_analyze::selection::Objective::Balanced,
        bench_corpus: None,
        allow_partial: false,
        adapter_preset: args.adapter_preset.as_str().to_string(),
        adapter_bank: args.adapter_bank.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
    }
}

/// # Errors
/// Returns an error if CLI arguments are invalid for preprocessing.
pub fn preprocess_args_from_cli(
    args: &FastqPreprocessArgs,
) -> Result<engine_args::BenchFastqPreprocessArgs> {
    let sample_id = args
        .sample_id
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--sample-id is required"))?;
    let r1 = args
        .r1
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--r1 is required"))?;
    let out = args
        .out
        .clone()
        .ok_or_else(|| anyhow::anyhow!("--out is required"))?;
    Ok(engine_args::BenchFastqPreprocessArgs {
        sample_id,
        r1,
        r2: args.r2.clone(),
        out,
        strict: args.strict,
        auto: args.auto,
        objective: args.objective.into(),
        bench_corpus: args.bench_corpus.map(Into::into),
        allow_partial: args.allow_partial,
        adapter_preset: args.adapter_preset.as_str().to_string(),
        adapter_bank: args.adapter_bank.clone(),
        enable_adapters: args.enable_adapter.clone(),
        disable_adapters: args.disable_adapter.clone(),
    })
}

/// # Errors
/// Returns an error if the runner override is not a valid runner kind.
pub fn parse_runner_override(env: Option<&str>) -> Result<Option<RunnerKind>> {
    match env {
        None => Ok(None),
        Some(name) => Ok(Some(
            RunnerKind::from_str(name).map_err(|_| anyhow!("unknown runner {name}"))?,
        )),
    }
}
