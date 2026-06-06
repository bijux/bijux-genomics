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
    #[command(name = "render-vcf-stage-catalog")]
    RenderVcfStageCatalog(BenchLocalRenderVcfStageCatalogArgs),
    #[command(name = "render-vcf-stage-matrix")]
    RenderVcfStageMatrix(BenchLocalRenderVcfStageMatrixArgs),
    #[command(name = "render-vcf-smoke-root")]
    RenderVcfSmokeRoot(BenchLocalRenderVcfSmokeRootArgs),
    #[command(name = "run-vcf-call-smoke")]
    RunVcfCallSmoke(BenchLocalRunVcfCallSmokeArgs),
    #[command(name = "run-vcf-call-diploid-smoke")]
    RunVcfCallDiploidSmoke(BenchLocalRunVcfCallDiploidSmokeArgs),
    #[command(name = "run-vcf-call-gl-smoke")]
    RunVcfCallGlSmoke(BenchLocalRunVcfCallGlSmokeArgs),
    #[command(name = "run-vcf-damage-filter-smoke")]
    RunVcfDamageFilterSmoke(BenchLocalRunVcfDamageFilterSmokeArgs),
    #[command(name = "run-vcf-filter-smoke")]
    RunVcfFilterSmoke(BenchLocalRunVcfFilterSmokeArgs),
    #[command(name = "run-vcf-qc-smoke")]
    RunVcfQcSmoke(BenchLocalRunVcfQcSmokeArgs),
    #[command(name = "run-vcf-stats-smoke")]
    RunVcfStatsSmoke(BenchLocalRunVcfStatsSmokeArgs),
    #[command(name = "run-vcf-gl-propagation-smoke")]
    RunVcfGlPropagationSmoke(BenchLocalRunVcfGlPropagationSmokeArgs),
    #[command(name = "run-vcf-call-pseudohaploid-smoke")]
    RunVcfCallPseudohaploidSmoke(BenchLocalRunVcfCallPseudohaploidSmokeArgs),
    #[command(name = "run-vcf-phasing-smoke")]
    RunVcfPhasingSmoke(BenchLocalRunVcfPhasingSmokeArgs),
    #[command(name = "run-vcf-prepare-reference-panel-smoke")]
    RunVcfPrepareReferencePanelSmoke(BenchLocalRunVcfPrepareReferencePanelSmokeArgs),
    #[command(name = "validate-vcf-no-empty-output")]
    ValidateVcfNoEmptyOutput(BenchLocalValidateVcfNoEmptyOutputArgs),
    #[command(name = "validate-vcf-stage-catalog-ready")]
    ValidateVcfStageCatalogReady(BenchLocalValidateVcfStageCatalogReadyArgs),
    #[command(name = "validate-vcf-reference-compatibility")]
    ValidateVcfReferenceCompatibility(BenchLocalValidateVcfReferenceCompatibilityArgs),
    #[command(name = "validate-vcf-sample-compatibility")]
    ValidateVcfSampleCompatibility(BenchLocalValidateVcfSampleCompatibilityArgs),
    #[command(name = "validate-hpc-submission-ready")]
    ValidateHpcSubmissionReady(BenchLocalValidateHpcSubmissionReadyArgs),
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
    #[command(name = "judge-taxonomy-output")]
    JudgeTaxonomyOutput(BenchLocalJudgeTaxonomyOutputArgs),
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
pub struct BenchLocalRenderVcfStageCatalogArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRenderVcfStageMatrixArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRenderVcfSmokeRootArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfCallSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfCallDiploidSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfCallGlSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfDamageFilterSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfFilterSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfQcSmokeArgs {
    #[arg(long, default_value = "plink2")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfStatsSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfGlPropagationSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfCallPseudohaploidSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfPhasingSmokeArgs {
    #[arg(long, default_value = "shapeit5")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalRunVcfPrepareReferencePanelSmokeArgs {
    #[arg(long, default_value = "bcftools")]
    pub tool_id: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateVcfNoEmptyOutputArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub skip_refresh: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateVcfStageCatalogReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateVcfReferenceCompatibilityArgs {
    #[arg(long)]
    pub manifest: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateVcfSampleCompatibilityArgs {
    #[arg(long)]
    pub manifest: Option<std::path::PathBuf>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchLocalValidateHpcSubmissionReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
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
pub struct BenchLocalJudgeTaxonomyOutputArgs {
    #[arg(long)]
    pub manifest: std::path::PathBuf,
    #[arg(long = "report")]
    pub reports: Vec<String>,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
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
