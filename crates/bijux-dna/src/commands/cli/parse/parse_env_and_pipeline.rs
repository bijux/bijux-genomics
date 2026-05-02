#[derive(Debug, Subcommand)]
pub enum EnvCommand {
    Images,
    Info,
    Doctor,
    List,
    #[command(name = "export-json")]
    ExportJson,
    #[command(name = "export-containers")]
    ExportContainers {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    #[command(name = "export-hpc")]
    ExportHpc {
        #[arg(long, default_value_t = false)]
        json: bool,
        #[arg(long)]
        hpc_root: Option<PathBuf>,
    },
    #[command(name = "sif-inventory")]
    SifInventory {
        #[arg(long)]
        hpc_root: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    #[command(name = "ensure")]
    Ensure(EnsureStageArgs),
    #[command(name = "apptainer-qa-matrix")]
    ApptainerQaMatrix {
        #[arg(long)]
        hpc_root: Option<PathBuf>,
        #[arg(long, default_value = "docs/30-operations/APPTAINER_QA_MATRIX.md")]
        out: PathBuf,
    },
    #[command(name = "ensure-images")]
    EnsureImages(EnsureImagesArgs),
    #[command(name = "lint-apptainer-defs")]
    LintApptainerDefs,
    Smoke(EnvRunArgs),
    Prep(EnvRunArgs),
}

#[derive(Debug, Args, Clone)]
pub struct EnsureImagesArgs {
    #[arg(long)]
    pub hpc_root: Option<PathBuf>,
    #[arg(long)]
    pub domain: String,
    #[arg(long, help = "Single stage id like fastq.trim_reads", conflicts_with = "stages")]
    pub stage: Option<String>,
    #[arg(long, help = "Comma-separated stage ids or short stage names")]
    pub stages: Option<String>,
    #[arg(long, default_value_t = false)]
    pub force_smoke: bool,
    #[arg(long, default_value_t = false)]
    pub repair_mismatch: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct EnsureStageArgs {
    #[arg(long)]
    pub hpc_root: Option<PathBuf>,
    #[arg(long, help = "Stage id like fastq.trim_reads")]
    pub stage: String,
    #[arg(long, default_value_t = false)]
    pub force_smoke: bool,
    #[arg(long, default_value_t = false)]
    pub repair_mismatch: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
    #[command(name = "init-hpc")]
    InitHpc {
        #[arg(long)]
        root: Option<PathBuf>,
    },
    #[command(name = "campaign-preflight")]
    CampaignPreflight {
        #[arg(long, value_name = "PATH")]
        config: PathBuf,
        #[arg(long, value_name = "PATH")]
        env_file: Option<PathBuf>,
        #[arg(long, value_name = "PATH")]
        user_overrides: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    #[command(name = "campaign-dry-run")]
    CampaignDryRun {
        #[arg(long, value_name = "PATH")]
        config: PathBuf,
        #[arg(long, value_name = "PATH")]
        env_file: Option<PathBuf>,
        #[arg(long, value_name = "PATH")]
        user_overrides: Option<PathBuf>,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    #[command(name = "write-campaign-profiles")]
    WriteCampaignProfiles {
        #[arg(long, default_value = "configs/hpc/campaign")]
        out_dir: PathBuf,
    },
    Doctor,
}

#[derive(Debug, Subcommand)]
pub enum SlurmCommand {
    #[command(name = "submit-stage-benchmark")]
    SubmitStageBenchmark(SlurmSubmitStageArgs),
    #[command(name = "submit-domain-benchmark")]
    SubmitDomainBenchmark(SlurmSubmitDomainArgs),
    #[command(name = "submit-cross-benchmark")]
    SubmitCrossBenchmark(SlurmSubmitCrossArgs),
    #[command(name = "submit-campaign")]
    SubmitCampaign(SlurmSubmitCampaignArgs),
    #[command(name = "cancel")]
    Cancel(SlurmCancelArgs),
    #[command(name = "copy-back-manifest")]
    CopyBackManifest(SlurmCopyBackManifestArgs),
    #[command(name = "decrypt-bundle")]
    DecryptBundle(SlurmBundleDecryptArgs),
    #[command(name = "verify-bundle")]
    VerifyBundle(SlurmBundleIntegrityCheck),
    #[command(name = "rewrap-bundle")]
    RewrapBundle(SlurmBundleRewrapArgs),
    #[command(name = "import-replay")]
    ImportReplay(SlurmReplayImportArgs),
    #[command(name = "import-campaign")]
    ImportCampaign(SlurmCampaignImportArgs),
    #[command(name = "export-failure-bundle")]
    ExportFailureBundle(SlurmFailureBundleExportArgs),
    #[command(name = "share-bundle")]
    ShareBundle(SlurmShareBundleArgs),
    #[command(name = "verify-results-policy")]
    VerifyResultsPolicy(SlurmResultsPolicyCheckArgs),
}

#[derive(Debug, Args, Clone)]
pub struct SlurmSubmitStageArgs {
    #[arg(long, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub env_file: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub user_overrides: Option<PathBuf>,
    #[arg(long)]
    pub stage: String,
    #[arg(long)]
    pub tool: Option<String>,
    #[arg(long)]
    pub sample: Option<String>,
    #[arg(long, default_value_t = false)]
    pub mock_submit: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmSubmitDomainArgs {
    #[arg(long, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub env_file: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub user_overrides: Option<PathBuf>,
    #[arg(long)]
    pub domain: String,
    #[arg(long, default_value_t = false)]
    pub mock_submit: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmSubmitCrossArgs {
    #[arg(long, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub env_file: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub user_overrides: Option<PathBuf>,
    #[arg(long, help = "Comma-separated domains to include")]
    pub domains: Option<String>,
    #[arg(long, default_value_t = false)]
    pub mock_submit: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmSubmitCampaignArgs {
    #[arg(long, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub env_file: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub user_overrides: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub mock_submit: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmCancelArgs {
    #[arg(long = "job-id", value_name = "JOB_ID")]
    pub job_id: Vec<String>,
    #[arg(long, value_name = "PATH")]
    pub manifest: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub mock_cancel: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmCopyBackManifestArgs {
    #[arg(long, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub env_file: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub user_overrides: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmBundleDecryptArgs {
    #[arg(long, value_name = "PATH")]
    pub bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub sidecar: Option<PathBuf>,
    #[arg(long, value_name = "PATH", default_value = "artifacts/investigation/decrypt")]
    pub out_dir: PathBuf,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_unsafe_destination: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmBundleIntegrityCheck {
    #[arg(long, value_name = "PATH")]
    pub bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub sidecar: Option<PathBuf>,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmBundleRewrapArgs {
    #[arg(long, value_name = "PATH")]
    pub bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub sidecar: Option<PathBuf>,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long = "recipient", value_name = "RECIPIENT")]
    pub recipient: Vec<String>,
    #[arg(long, value_name = "PATH")]
    pub out_bundle: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmReplayImportArgs {
    #[arg(long, value_name = "PATH")]
    pub results_bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub results_sidecar: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub code_bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub code_sidecar: Option<PathBuf>,
    #[arg(long, value_name = "PATH", default_value = "artifacts/investigation/replay")]
    pub out_dir: PathBuf,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_unsafe_destination: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmCampaignImportArgs {
    #[arg(long, value_name = "PATH")]
    pub campaign_dir: PathBuf,
    #[arg(long, value_name = "PATH", default_value = "artifacts/investigation/campaign-import")]
    pub out_dir: PathBuf,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub allow_unsafe_destination: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmFailureBundleExportArgs {
    #[arg(long, value_name = "PATH")]
    pub config: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub env_file: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub user_overrides: Option<PathBuf>,
    #[arg(long)]
    pub stage: String,
    #[arg(long)]
    pub tool: String,
    #[arg(long)]
    pub sample: String,
    #[arg(long, value_name = "PATH")]
    pub out_dir: PathBuf,
    #[arg(long = "recipient", value_name = "RECIPIENT")]
    pub recipient: Vec<String>,
    #[arg(long, default_value = "mock-envelope-v1")]
    pub backend: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmShareBundleArgs {
    #[arg(long, value_name = "PATH")]
    pub bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub sidecar: Option<PathBuf>,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub profile: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub out_dir: PathBuf,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args, Clone)]
pub struct SlurmResultsPolicyCheckArgs {
    #[arg(long, value_name = "PATH")]
    pub results_bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub results_sidecar: Option<PathBuf>,
    #[arg(long, value_name = "PATH")]
    pub code_bundle: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub code_sidecar: Option<PathBuf>,
    #[arg(long = "identity-file", value_name = "PATH")]
    pub identity_file: Vec<PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Subcommand)]
pub enum RegistryCommand {
    #[command(name = "list-tools", alias = "tools")]
    Tools {
        #[arg(long)]
        stage: Option<String>,
        #[arg(long, requires = "stage")]
        scenario: Option<String>,
        #[arg(
            long,
            default_value = "all",
            help = "Tool kind: all | primary | optional | validation | reporting | benchmark"
        )]
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
    #[command(name = "export-containers")]
    ExportContainers {
        #[arg(long, default_value_t = false)]
        json: bool,
    },
    #[command(name = "coverage-matrix")]
    CoverageMatrix,
    #[command(name = "validate-tool")]
    ValidateTool { id: String },
    #[command(name = "audit")]
    Audit {
        #[arg(long, default_value_t = false)]
        show_binding_violations: bool,
        #[arg(long, default_value_t = false)]
        fix_suggestions: bool,
        #[arg(long, default_value_t = false)]
        fix_hints: bool,
    },
    #[command(name = "doctor")]
    Doctor {
        #[arg(long)]
        domain: Option<String>,
    },
    #[command(name = "promote")]
    Promote {
        #[arg(long = "tool")]
        tool: String,
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
    Validate { id: String },
}

#[derive(Debug, Subcommand)]
pub enum EnaCommand {
    Select(EnaSelectArgs),
    Fetch(EnaFetchArgs),
}

#[derive(Debug, Args)]
pub struct EnaSelectArgs {
    #[arg(long)]
    pub project: String,
    #[arg(long)]
    pub species: String,
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus_id: String,
    #[arg(long = "target-se", default_value_t = 10)]
    pub target_se: usize,
    #[arg(long = "target-pe", default_value_t = 10)]
    pub target_pe: usize,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct EnaFetchArgs {
    #[arg(long)]
    pub species: String,
    #[arg(long, value_name = "PATH")]
    pub snapshot: PathBuf,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum CorpusCommand {
    Materialize(CorpusMaterializeArgs),
    Normalize {
        corpus: String,
    },
    Validate {
        corpus: String,
    },
    List(CorpusListArgs),
    Diff {
        left: String,
        right: String,
        #[arg(long, default_value_t = false)]
        json: bool,
    },
}

include!("common_example_args.rs");
include!("common_root_args.rs");

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
