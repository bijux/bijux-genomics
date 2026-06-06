use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
#[derive(Debug, Parser)]
#[command(name = "bijux-dna", version, about = "Bijux DNA CLI", subcommand_required = true, arg_required_else_help = true)]
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
    pub command: DnaCommand,
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

macro_rules! nested_root_command_args {
    ($name:ident, $command:ty) => {
        #[derive(Debug, Args)]
        pub struct $name {
            #[command(subcommand)]
            pub command: $command,
        }
    };
}

nested_root_command_args!(EnvRootArgs, EnvCommand);
nested_root_command_args!(RegistryRootArgs, RegistryCommand);
nested_root_command_args!(EnaRootArgs, EnaCommand);
nested_root_command_args!(CorpusRootArgs, CorpusCommand);
nested_root_command_args!(FixturesRootArgs, FixturesCommand);
nested_root_command_args!(ToolRootArgs, ToolCommand);
nested_root_command_args!(DomainRootArgs, DomainCommand);
nested_root_command_args!(LabRootArgs, LabCommand);
nested_root_command_args!(ConfigRootArgs, ConfigCommand);
nested_root_command_args!(SlurmRootArgs, SlurmCommand);
nested_root_command_args!(FastqRootArgs, FastqCommand);
nested_root_command_args!(BamRootArgs, BamCommand);
nested_root_command_args!(VcfRootArgs, VcfCommand);
nested_root_command_args!(PipelinesRootArgs, PipelinesCommand);
nested_root_command_args!(AnalyzeRootArgs, AnalyzeCommand);
nested_root_command_args!(BenchRootArgs, BenchCommand);
nested_root_command_args!(PoliciesRootArgs, PoliciesCommand);
nested_root_command_args!(CiRootArgs, CiCommand);

#[derive(Debug, Subcommand)]
pub enum DnaCommand {
    #[command(name = "env", alias = "environment")]
    Environment(EnvRootArgs),
    Registry(RegistryRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Ena(EnaRootArgs),
    Corpus(CorpusRootArgs),
    Fixtures(FixturesRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Tool(ToolRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Domain(DomainRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Lab(LabRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Config(ConfigRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Slurm(SlurmRootArgs),
    Status(StatusArgs),
    #[command(name = "run")]
    Fastq(FastqRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Bam(BamRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Vcf(VcfRootArgs),
    #[command(name = "plan")]
    Pipelines(PipelinesRootArgs),
    Analyze(AnalyzeRootArgs),
    Explain(AnalyzeRootArgs),
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
    Bench(BenchRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Policies(PoliciesRootArgs),
    #[cfg_attr(not(debug_assertions), command(hide = true))]
    Ci(CiRootArgs),
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
    Evidence(AnalyzeEvidenceRootArgs),
    Bench(AnalyzeBenchArgs),
}

#[derive(Debug, Args)]
pub struct AnalyzeEvidenceRootArgs {
    #[command(subcommand)]
    pub command: AnalyzeEvidenceCommand,
}

#[derive(Debug, Subcommand)]
pub enum AnalyzeEvidenceCommand {
    Verify(AnalyzeEvidenceVerifyArgs),
    Compare(AnalyzeEvidenceCompareArgs),
}

#[derive(Debug, Args)]
pub struct AnalyzeEvidenceVerifyArgs {
    #[arg(long, default_value = "artifacts/bench")]
    pub search_root: PathBuf,
    #[arg(long)]
    pub run_id: Option<String>,
    #[arg(long, value_name = "PATH")]
    pub bundle_path: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct AnalyzeEvidenceCompareArgs {
    #[arg(value_name = "LEFT_BUNDLE")]
    pub left: PathBuf,
    #[arg(value_name = "RIGHT_BUNDLE")]
    pub right: PathBuf,
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
