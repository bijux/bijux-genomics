use clap::ValueEnum;
use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;
#[derive(Debug, Parser)]
#[command(name = "bijux", version, about = "Bijux DNA CLI", subcommand_required = true, arg_required_else_help = true)]
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
    #[arg(long, verbatim_doc_comment)]
    /// Dump effective config JSON (alias for --print-effective-config).
    pub dump_effective_config: bool,
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
    #[command(alias = "env")]
    Environment {
        #[command(subcommand)]
        command: EnvCommand,
    },
    Registry {
        #[command(subcommand)]
        command: RegistryCommand,
    },
    Ena {
        #[command(subcommand)]
        command: EnaCommand,
    },
    Corpus {
        #[command(subcommand)]
        command: CorpusCommand,
    },
    Tool {
        #[command(subcommand)]
        command: ToolCommand,
    },
    Domain {
        #[command(subcommand)]
        command: DomainCommand,
    },
    Lab {
        #[command(subcommand)]
        command: LabCommand,
    },
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    Status(StatusArgs),
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
    Fastq {
        #[command(subcommand)]
        command: FastqCommand,
    },
    Bam {
        #[command(subcommand)]
        command: BamCommand,
    },
    Vcf {
        #[command(subcommand)]
        command: VcfCommand,
    },
    Pipelines {
        #[command(subcommand)]
        command: PipelinesCommand,
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
    Policies {
        #[command(subcommand)]
        command: PoliciesCommand,
    },
    Debug(DebugArgs),
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
    pub verify_only: bool,
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

#[derive(Debug, Subcommand)]
pub enum EnvCommand {
    Images,
    Info,
    Doctor,
    List,
    #[command(name = "export-json")]
    ExportJson,
    #[command(name = "export-hpc")]
    ExportHpc {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    #[command(name = "ensure-images")]
    EnsureImages(EnsureImagesArgs),
    Smoke(EnvRunArgs),
    Prep(EnvRunArgs),
}

#[derive(Debug, Args, Clone)]
pub struct EnsureImagesArgs {
    #[arg(long)]
    pub domain: String,
    #[arg(long, help = "Comma-separated stage ids or short stage names")]
    pub stages: String,
    #[arg(long, default_value_t = false)]
    pub force_smoke: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    #[command(name = "init-hpc")]
    InitHpc {
        #[arg(long, default_value = "/home/bijan/bijux")]
        root: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    #[command(name = "list-tools", alias = "tools")]
    Tools {
        #[arg(long)]
        stage: Option<String>,
        #[arg(long, default_value = "all")]
        kind: String,
    },
    #[command(name = "list-stages", alias = "stages")]
    Stages,
    #[command(name = "show-tool")]
    ShowTool { id: String },
    #[command(name = "show-stage")]
    ShowStage { id: String },
    #[command(name = "show")]
    Show { id: String },
    #[command(name = "export-json")]
    ExportJson,
    #[command(name = "coverage-matrix")]
    CoverageMatrix,
    #[command(name = "verify-tool")]
    VerifyTool { id: String },
    #[command(name = "audit")]
    Audit {
        #[arg(long, default_value_t = false)]
        fix_suggestions: bool,
    },
    #[command(name = "lint")]
    Lint {
        #[arg(long, default_value_t = false)]
        hpc: bool,
        #[arg(long)]
        domain: Option<String>,
        #[arg(long, help = "Comma-separated stage ids or short stage names")]
        stages: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ToolCommand {
    Verify { id: String },
}

#[derive(Debug, Subcommand)]
pub enum EnaCommand {
    Fetch(EnaFetchArgs),
}

#[derive(Debug, Args)]
pub struct EnaFetchArgs {
    #[arg(long)]
    pub project: String,
    #[arg(long = "limit", value_delimiter = ',', default_values = ["10-se", "10-pe"])]
    pub limits: Vec<String>,
    #[arg(long, default_value = "bijux-dna-data/corpus-01/raw")]
    pub out: PathBuf,
}

#[derive(Debug, Subcommand)]
pub enum CorpusCommand {
    Validate {
        corpus: String,
    },
    List {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum DomainCommand {
    Validate {
        #[arg(long, default_value = "domain")]
        domain_dir: PathBuf,
    },
    Coverage {
        #[arg(long, default_value = "domain")]
        domain_dir: PathBuf,
    },
}

#[derive(Debug, Args)]
pub struct EnvRunArgs {
    pub runtime: String,
    pub tool: Option<String>,
    #[arg(long)]
    pub stage: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum LabCommand {
    Corpus {
        #[command(subcommand)]
        command: LabCorpusCommand,
    },
}

#[derive(Debug, Subcommand)]
pub enum LabCorpusCommand {
    #[command(name = "list-fastq")]
    ListFastq {
        #[arg(long, default_value = "canonical")]
        corpus: String,
        #[arg(long)]
        paired: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum PipelinesCommand {
    #[command(about = "List pipeline profiles.")]
    List {
        #[arg(long, value_enum)]
        domain: Option<PipelineDomainArg>,
        #[arg(long, help = "Include beta/experimental pipelines")]
        show_experimental: bool,
    },
    #[command(about = "Explain a pipeline profile.")]
    Explain {
        id: String,
        #[arg(long, default_value_t = false)]
        explain_io: bool,
    },
    #[command(about = "Explain profile defaults and invariants status.")]
    #[command(name = "explain-profile")]
    ExplainProfile {
        id: String,
    },
    #[command(about = "Validate a pipeline profile invariants and print a report.")]
    #[command(name = "validate-profile")]
    ValidateProfile {
        id: String,
    },
    #[command(about = "Diff two pipeline profiles (tools, params, invariants).")]
    #[command(name = "profile-diff")]
    ProfileDiff {
        left: String,
        right: String,
    },
    #[command(about = "Audit pipeline stages and completeness.")]
    Audit {
        #[arg(long, value_enum)]
        domain: Option<PipelineDomainArg>,
        #[arg(long, help = "Include beta/experimental pipelines")]
        show_experimental: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum PipelineDomainArg {
    Fastq,
    Bam,
    Vcf,
    Cross,
}

impl PipelineDomainArg {
    #[must_use]
    pub fn as_domain(self) -> bijux_dna_api::v1::api::plan::Domain {
        match self {
            Self::Fastq => bijux_dna_api::v1::api::plan::Domain::Fastq,
            Self::Bam => bijux_dna_api::v1::api::plan::Domain::Bam,
            Self::Vcf => bijux_dna_api::v1::api::plan::Domain::Vcf,
            Self::Cross => bijux_dna_api::v1::api::plan::Domain::Cross,
        }
    }
}

#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum BenchCommand {
    Run(BenchRunArgs),
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
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
}

#[derive(Debug, Args)]
pub struct BenchFastqValidateArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
}

#[derive(Debug, Args)]
pub struct BenchFastqFilterArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqCorrectArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqQcPostArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqUmiArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
pub struct BenchFastqScreenArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
    pub out: PathBuf,
    #[arg(long, value_delimiter = ',', default_value = "auto", help = "Tool selection: auto | all | <csv>")]
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
#[allow(clippy::struct_excessive_bools)]
pub struct BenchFastqPreprocessArgs {
    #[arg(long, alias = "sample")]
    pub sample_id: String,
    #[arg(long, help = "Pipeline profile id (default, minimal)")]
    pub pipeline_profile: Option<String>,
    #[arg(long)]
    pub r1: PathBuf,
    #[arg(long)]
    pub r2: Option<PathBuf>,
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
    #[arg(long, help = "Enable error correction stage (paired-end only)")]
    pub enable_correct: bool,
    #[arg(long, help = "Allow planned/out-of-scope stages in planning")]
    pub allow_planned: bool,
}
