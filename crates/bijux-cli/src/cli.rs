use std::path::PathBuf;
use std::str::FromStr;

use anyhow::{anyhow, Result};
use bijux_core::{StageId, ToolId};
use bijux_engine::bench::args as engine_args;
use bijux_environment::api::RunnerKind;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "bijux", version, about = "Bijux DNA CLI")]
pub struct Cli {
    #[arg(long, default_value = "local")]
    pub profile: String,
    #[arg(long)]
    pub platform: Option<String>,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Fastq {
        #[command(subcommand)]
        command: FastqCommand,
    },
    ValidateManifests,
    Platform,
    ImageQa,
    Replay(ReplayArgs),
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
    Qc2(BenchFastqQc2Args),
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
pub struct BenchFastqQc2Args {
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
}

#[derive(Debug, Args, Clone, Default)]
pub struct CommonArgs {
    #[arg(long)]
    pub list_tools: bool,
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Subcommand)]
pub enum FastqCommand {
    #[command(
        about = "Filter FASTQ reads.",
        after_help = "Examples:\n  bijux fastq filter --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastp\n  bijux fastq filter --list-tools"
    )]
    Filter(CommonArgs),
    Merge(CommonArgs),
    #[command(
        about = "Trim FASTQ reads.",
        after_help = "Examples:\n  bijux fastq trim --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastp\n  bijux fastq trim --list-tools"
    )]
    Trim(FastqTrimArgs),
    Contam(CommonArgs),
    #[command(
        about = "Run the FASTQ preprocess pipeline (validate → trim → filter → stats).",
        after_help = "Examples:\n  bijux fastq preprocess --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE\n  bijux fastq preprocess --list-tools"
    )]
    Preprocess(CommonArgs),
    Umi(CommonArgs),
    #[command(name = "error-correct")]
    ErrorCorrect(CommonArgs),
    Qc(CommonArgs),
    #[command(
        about = "Validate FASTQ reads.",
        after_help = "Examples:\n  bijux fastq validate --r1 reads.fastq.gz --out artifacts --sample-id SAMPLE --tools fastqvalidator_official\n  bijux fastq validate --list-tools"
    )]
    Validate(FastqValidateArgs),
    Align(CommonArgs),
}

#[derive(Debug, Args, Clone)]
pub struct FastqTrimArgs {
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

pub fn resolve_stage_tool(command: &Commands) -> (StageId, ToolId, CommonArgs) {
    match command {
        Commands::Fastq { command } => match command {
            FastqCommand::Trim(args) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                args.common.clone(),
            ),
            FastqCommand::Validate(args) => (
                StageId("fastq.validate".to_string()),
                ToolId("fastqvalidator".to_string()),
                args.common.clone(),
            ),
            FastqCommand::Filter(common)
            | FastqCommand::Merge(common)
            | FastqCommand::Contam(common)
            | FastqCommand::Preprocess(common)
            | FastqCommand::Umi(common)
            | FastqCommand::ErrorCorrect(common)
            | FastqCommand::Qc(common)
            | FastqCommand::Align(common) => (
                StageId("fastq.trim".to_string()),
                ToolId("fastp".to_string()),
                common.clone(),
            ),
        },
        _ => (
            StageId("fastq.trim".to_string()),
            ToolId("fastp".to_string()),
            CommonArgs::default(),
        ),
    }
}

pub fn is_bench_requested_trim(args: &FastqTrimArgs) -> bool {
    args.sample_id.is_some() && args.r1.is_some() && args.out.is_some()
}

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
    })
}

pub fn is_bench_requested_validate(args: &FastqValidateArgs) -> bool {
    args.sample_id.is_some() && args.r1.is_some() && args.out.is_some()
}

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

pub fn bench_args_trim(args: &BenchFastqTrimArgs) -> engine_args::BenchFastqTrimArgs {
    engine_args::BenchFastqTrimArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

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

pub fn bench_args_filter(args: &BenchFastqFilterArgs) -> engine_args::BenchFastqFilterArgs {
    engine_args::BenchFastqFilterArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

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

pub fn bench_args_qc2(args: &BenchFastqQc2Args) -> engine_args::BenchFastqQc2Args {
    engine_args::BenchFastqQc2Args {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

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

pub fn bench_args_screen(args: &BenchFastqScreenArgs) -> engine_args::BenchFastqScreenArgs {
    engine_args::BenchFastqScreenArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

pub fn bench_args_stats(args: &BenchFastqStatsArgs) -> engine_args::BenchFastqStatsArgs {
    engine_args::BenchFastqStatsArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        out: args.out.clone(),
        tools: args.tools.clone(),
        explain: args.explain,
    }
}

pub fn bench_args_preprocess(
    args: &BenchFastqPreprocessArgs,
) -> engine_args::BenchFastqPreprocessArgs {
    engine_args::BenchFastqPreprocessArgs {
        sample_id: args.sample_id.clone(),
        r1: args.r1.clone(),
        r2: args.r2.clone(),
        out: args.out.clone(),
        strict: args.strict,
    }
}

pub fn parse_runner_override(env: Option<&str>) -> Result<Option<RunnerKind>> {
    match env {
        None => Ok(None),
        Some(name) => Ok(Some(
            RunnerKind::from_str(name).map_err(|_| anyhow!("unknown runner {name}"))?,
        )),
    }
}
