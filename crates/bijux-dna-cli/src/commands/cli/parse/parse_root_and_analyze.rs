use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
#[derive(Debug, Parser)]
#[allow(clippy::struct_excessive_bools)]
#[command(name = "bijux", version, about = "Bijux DNA CLI", subcommand_required = true, arg_required_else_help = true)]
pub struct Cli {
    #[arg(short = 'v', long, global = true, default_value_t = false)]
    pub verbose: bool,
    #[arg(short = 'q', long, global = true, default_value_t = false)]
    pub quiet: bool,
    #[arg(long, global = true, value_name = "LEVEL")]
    pub log_level: Option<String>,
    #[arg(long, default_value = "local")]
    pub profile: String,
    #[arg(long)]
    pub platform: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub telemetry_jsonl: Option<PathBuf>,
    #[arg(long, verbatim_doc_comment)]
    /// Print resolved config JSON (skeleton for future defaults).
    pub print_effective_config: bool,
    #[arg(long, verbatim_doc_comment)]
    /// Dump effective config JSON (alias for --print-effective-config).
    pub dump_effective_config: bool,
    #[arg(long, global = true, default_value_t = false)]
    pub json: bool,
    #[command(subcommand)]
    pub command: RootCommand,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum RootCommand {
    Dna {
        #[command(subcommand)]
        command: DnaCommand,
    },
}

#[derive(Debug, Args)]
pub struct StatusArgs {
    #[arg(long, default_value = "pre-hpc")]
    pub scope: String,
    #[arg(long, value_name = "PATH")]
    pub write_checklist: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub placeholders: bool,
    #[arg(long, default_value_t = false)]
    pub contracts: bool,
    #[arg(long, default_value_t = false)]
    pub hpc: bool,
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum DnaCommand {
    #[command(name = "env", alias = "environment")]
    Environment {
        #[command(subcommand)]
        command: EnvCommand,
    },
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Ena {
        #[command(subcommand)]
        command: EnaCommand,
    },
    Corpus {
        #[command(subcommand)]
        command: CorpusCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Tool {
        #[command(subcommand)]
        command: ToolCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Domain {
        #[command(subcommand)]
        command: DomainCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Lab {
        #[command(subcommand)]
        command: LabCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Status(StatusArgs),
    #[command(name = "run")]
    Fastq {
        #[command(subcommand)]
        command: FastqCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Bam {
        #[command(subcommand)]
        command: BamCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Vcf {
        #[command(subcommand)]
        command: VcfCommand,
    },
    #[command(name = "plan")]
    Pipelines {
        #[command(subcommand)]
        command: PipelinesCommand,
    },
    Analyze {
        #[command(subcommand)]
        command: AnalyzeCommand,
    },
    Explain {
        #[command(subcommand)]
        command: AnalyzeCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    ValidateManifests,
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Platform,
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    ImageQa,
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Replay(ReplayArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Compare(CompareArgs),
    Bench {
        #[command(subcommand)]
        command: BenchCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Policies {
        #[command(subcommand)]
        command: PoliciesCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Ci {
        #[command(subcommand)]
        command: CiCommand,
    },
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Debug(DebugArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Collect(CollectArgs),
}

#[derive(Debug, Args)]
pub struct DebugArgs {
    #[arg(long, default_value = "tail")]
    pub view: String,
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    pub run_id: String,
}

#[derive(Debug, Args)]
pub struct CollectArgs {
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    #[arg(long)]
    pub run: String,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum PoliciesCommand {
    #[command(about = "Audit workspace boundaries and output a DOT graph.")]
    Audit {
        #[arg(long, default_value = "artifacts/workspace")]
        out: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct ReplayArgs {
    pub run_id: String,
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    #[arg(long)]
    pub manifest: Option<PathBuf>,
    #[arg(long)]
    pub validate_only: bool,
}

#[derive(Debug, Args)]
pub struct CompareArgs {
    pub run_a: String,
    pub run_b: String,
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long)]
    pub baseline: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum AnalyzeCommand {
    Runs(AnalyzeRunsArgs),
    Summary(AnalyzeSummaryArgs),
    Compare(AnalyzeCompareArgs),
    Rank(AnalyzeRankArgs),
    Report(AnalyzeReportArgs),
    Metrics(AnalyzeMetricsArgs),
    Bench(AnalyzeBenchArgs),
}

#[derive(Debug, Args)]
pub struct AnalyzeBenchArgs {
    #[arg(long)]
    pub suite: String,
    #[arg(long, default_value = "json")]
    pub report: String,
}

#[derive(Debug, Args)]
pub struct AnalyzeRunsArgs {
    #[arg(long, default_value = "runs/bijux-dna-runs/index.jsonl")]
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

#[derive(Debug, Args)]
pub struct AnalyzeSummaryArgs {
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    pub run_id: String,
}

#[derive(Debug, Args)]
pub struct AnalyzeCompareArgs {
    pub run_a: String,
    pub run_b: String,
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = ObjectiveArg::Balanced)]
    pub objective: ObjectiveArg,
    #[arg(long)]
    pub baseline: Option<String>,
}

#[derive(Debug, Args)]
pub struct AnalyzeRankArgs {
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    pub run_id: String,
    #[arg(long)]
    pub stage: String,
}

#[derive(Debug, Args)]
pub struct AnalyzeReportArgs {
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    pub run_id: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub run_dir: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub facts_path: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub sqlite: Option<PathBuf>,
    #[arg(long, default_value = "json")]
    pub format: String,
}

#[derive(Debug, Args)]
pub struct AnalyzeMetricsArgs {
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    pub run_id: String,
}

