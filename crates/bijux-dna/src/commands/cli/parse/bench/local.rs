use clap::{Args, Subcommand, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchLocalDomainArg {
    Fastq,
    Bam,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchLocalDagWatchdogScenarioArg {
    NoGlobalWait,
    FailureIsolation,
    PartialResume,
    CompletionRules,
}

#[derive(Debug, Subcommand)]
pub enum BenchLocalCommand {
    #[command(name = "list-stages")]
    ListStages(BenchLocalListStagesArgs),
    #[command(name = "simulate-dag-watchdog")]
    SimulateDagWatchdog(BenchLocalSimulateDagWatchdogArgs),
    #[command(name = "validate-pipeline-dag")]
    ValidatePipelineDag(BenchLocalValidatePipelineDagArgs),
    #[command(name = "validate-corpus-fixture")]
    ValidateCorpusFixture(BenchLocalValidateCorpusFixtureArgs),
    #[command(name = "validate-corpus-stage-compatibility")]
    ValidateCorpusStageCompatibility(BenchLocalValidateCorpusStageCompatibilityArgs),
    #[command(name = "render-corpus-skip-report")]
    RenderCorpusSkipReport(BenchLocalRenderCorpusSkipReportArgs),
    #[command(name = "validate-taxonomy-database-fixture")]
    ValidateTaxonomyDatabaseFixture(BenchLocalValidateTaxonomyDatabaseFixtureArgs),
    #[command(name = "validate-slurm-shell-syntax")]
    ValidateSlurmShellSyntax(BenchLocalValidateSlurmShellSyntaxArgs),
    #[command(name = "validate-slurm-dependencies")]
    ValidateSlurmDependencies(BenchLocalValidateSlurmDependenciesArgs),
    #[command(name = "validate-slurm-script-bodies")]
    ValidateSlurmScriptBodies(BenchLocalValidateSlurmScriptBodiesArgs),
    #[command(name = "render-slurm-submit-manifest")]
    RenderSlurmSubmitManifest(BenchLocalRenderSlurmSubmitManifestArgs),
    #[command(name = "render-benchmark-summary")]
    RenderBenchmarkSummary(BenchLocalRenderBenchmarkSummaryArgs),
    #[command(name = "check-manifest-completion")]
    CheckManifestCompletion(BenchLocalCheckManifestCompletionArgs),
    #[command(name = "check-output-completion")]
    CheckOutputCompletion(BenchLocalCheckOutputCompletionArgs),
    #[command(name = "collect-runtime-metrics")]
    CollectRuntimeMetrics(BenchLocalCollectRuntimeMetricsArgs),
    #[command(name = "render-tool-comparison-template")]
    RenderToolComparisonTemplate(BenchLocalRenderToolComparisonTemplateArgs),
    #[command(name = "validate-stage-result")]
    ValidateStageResult(BenchLocalValidateStageResultArgs),
    #[command(name = "materialize-stage")]
    MaterializeStage(BenchLocalMaterializeStageArgs),
    #[command(name = "fake-run-failures")]
    FakeRunFailures(BenchLocalFakeRunFailuresArgs),
    #[command(name = "fake-run-stages")]
    FakeRunStages(BenchLocalFakeRunStagesArgs),
    #[command(name = "render-slurm-scripts")]
    RenderSlurmScripts(BenchLocalRenderSlurmScriptsArgs),
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
pub struct BenchLocalSimulateDagWatchdogArgs {
    #[arg(long, value_enum, default_value_t = BenchLocalDagWatchdogScenarioArg::NoGlobalWait)]
    pub scenario: BenchLocalDagWatchdogScenarioArg,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidatePipelineDagArgs {
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateCorpusFixtureArgs {
    #[arg(long)]
    pub manifest: std::path::PathBuf,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateCorpusStageCompatibilityArgs {
    #[arg(long)]
    pub matrix: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRenderCorpusSkipReportArgs {
    #[arg(long)]
    pub matrix: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateTaxonomyDatabaseFixtureArgs {
    #[arg(long)]
    pub manifest: std::path::PathBuf,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateSlurmShellSyntaxArgs {
    #[arg(long)]
    pub root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateSlurmDependenciesArgs {
    #[arg(long)]
    pub root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub manifest: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateSlurmScriptBodiesArgs {
    #[arg(long)]
    pub root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRenderSlurmSubmitManifestArgs {
    #[arg(long)]
    pub root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRenderBenchmarkSummaryArgs {
    #[arg(long)]
    pub fake_run_root: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output_json: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output_markdown: Option<std::path::PathBuf>,
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
pub struct BenchLocalRenderToolComparisonTemplateArgs {
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
pub struct BenchLocalRenderSlurmScriptsArgs {
    #[arg(long, value_enum)]
    pub domain: BenchLocalDomainArg,
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
