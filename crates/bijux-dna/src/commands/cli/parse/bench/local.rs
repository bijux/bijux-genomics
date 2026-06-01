use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchLocalDomainArg {
    Fastq,
    Bam,
}

#[derive(Debug, Subcommand)]
pub enum BenchLocalCommand {
    #[command(name = "list-stages")]
    ListStages(BenchLocalListStagesArgs),
    #[command(name = "check-manifest-completion")]
    CheckManifestCompletion(BenchLocalCheckManifestCompletionArgs),
    #[command(name = "check-output-completion")]
    CheckOutputCompletion(BenchLocalCheckOutputCompletionArgs),
    #[command(name = "collect-runtime-metrics")]
    CollectRuntimeMetrics(BenchLocalCollectRuntimeMetricsArgs),
    #[command(name = "validate-stage-result")]
    ValidateStageResult(BenchLocalValidateStageResultArgs),
    #[command(name = "materialize-stage")]
    MaterializeStage(BenchLocalMaterializeStageArgs),
    #[command(name = "fake-run-failures")]
    FakeRunFailures(BenchLocalFakeRunFailuresArgs),
    #[command(name = "fake-run-stages")]
    FakeRunStages(BenchLocalFakeRunStagesArgs),
    #[command(name = "render-stage-commands")]
    RenderStageCommands(BenchLocalRenderStageCommandsArgs),
}

#[derive(Debug, Args)]
pub struct BenchLocalListStagesArgs {
    #[arg(long, value_enum)]
    pub domain: BenchLocalDomainArg,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalCheckOutputCompletionArgs {
    #[arg(long)]
    pub fake_run_root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalCheckManifestCompletionArgs {
    #[arg(long)]
    pub fake_run_root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalCollectRuntimeMetricsArgs {
    #[arg(long)]
    pub fake_run_root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalMaterializeStageArgs {
    #[arg(long)]
    pub stage_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateStageResultArgs {
    #[arg(long)]
    pub manifest: std::path::PathBuf,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalFakeRunStagesArgs {
    #[arg(long)]
    pub output_root: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalFakeRunFailuresArgs {
    #[arg(long)]
    pub output_root: Option<std::path::PathBuf>,
    #[arg(long = "stage-id")]
    pub stage_ids: Vec<String>,
    #[arg(long, default_value_t = 1)]
    pub exit_code: i32,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRenderStageCommandsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
