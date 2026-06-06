use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchReadinessCommand {
    #[command(name = "render-adapter-missing-input-tests")]
    RenderAdapterMissingInputTests(BenchReadinessRenderAdapterMissingInputTestsArgs),
    #[command(name = "render-commands")]
    RenderCommands(BenchReadinessRenderCommandsArgs),
    #[command(name = "render-command-argv")]
    RenderCommandArgv(BenchReadinessRenderCommandArgvArgs),
    #[command(name = "render-stage-tool-containers")]
    RenderStageToolContainers(BenchReadinessRenderStageToolContainersArgs),
    #[command(name = "render-stage-tool-assets")]
    RenderStageToolAssets(BenchReadinessRenderStageToolAssetsArgs),
    #[command(name = "render-stage-tool-resources")]
    RenderStageToolResources(BenchReadinessRenderStageToolResourcesArgs),
    #[command(name = "render-bam-adapter-output-contract")]
    RenderBamAdapterOutputContract(BenchReadinessRenderBamAdapterOutputContractArgs),
    #[command(name = "render-bam-stage-decision-table")]
    RenderBamStageDecisionTable(BenchReadinessRenderBamStageDecisionTableArgs),
    #[command(name = "render-bam-command-adapter-coverage")]
    RenderBamCommandAdapterCoverage(BenchReadinessRenderBamCommandAdapterCoverageArgs),
    #[command(name = "render-bam-corpus-assignment")]
    RenderBamCorpusAssignment(BenchReadinessRenderBamCorpusAssignmentArgs),
    #[command(name = "render-corpus-incompatibility")]
    RenderCorpusIncompatibility(BenchReadinessRenderCorpusIncompatibilityArgs),
    #[command(name = "render-corpus-centric-report")]
    RenderCorpusCentricReport(BenchReadinessRenderCorpusCentricReportArgs),
    #[command(name = "render-benchmark-readiness-dashboard")]
    RenderBenchmarkReadinessDashboard(BenchReadinessRenderBenchmarkReadinessDashboardArgs),
    #[command(name = "render-stage-tool-benchmark-ready")]
    RenderStageToolBenchmarkReady(BenchReadinessRenderStageToolBenchmarkReadyArgs),
    #[command(name = "render-bam-comparable-metrics")]
    RenderBamComparableMetrics(BenchReadinessRenderBamComparableMetricsArgs),
    #[command(name = "render-bam-normalized-metrics-schema")]
    RenderBamNormalizedMetricsSchema(BenchReadinessRenderBamNormalizedMetricsSchemaArgs),
    #[command(name = "render-bam-parser-coverage")]
    RenderBamParserCoverage(BenchReadinessRenderBamParserCoverageArgs),
    #[command(name = "render-bam-report-map")]
    RenderBamReportMap(BenchReadinessRenderBamReportMapArgs),
    #[command(name = "render-expected-benchmark-results")]
    RenderExpectedBenchmarkResults(BenchReadinessRenderExpectedBenchmarkResultsArgs),
    #[command(name = "render-missing-result-report")]
    RenderMissingResultReport(BenchReadinessRenderMissingResultReportArgs),
    #[command(name = "render-pair-readiness")]
    RenderPairReadiness(BenchReadinessRenderPairReadinessArgs),
    #[command(name = "render-stage-centric-report")]
    RenderStageCentricReport(BenchReadinessRenderStageCentricReportArgs),
    #[command(name = "render-tool-centric-report")]
    RenderToolCentricReport(BenchReadinessRenderToolCentricReportArgs),
    #[command(name = "render-parser-completeness-gate")]
    RenderParserCompletenessGate(BenchReadinessRenderParserCompletenessGateArgs),
    #[command(name = "render-corpus-asset-coverage-gate")]
    RenderCorpusAssetCoverageGate(BenchReadinessRenderCorpusAssetCoverageGateArgs),
    #[command(name = "render-parser-failure-tests")]
    RenderParserFailureTests(BenchReadinessRenderParserFailureTestsArgs),
    #[command(name = "render-fastq-adapter-output-contract")]
    RenderFastqAdapterOutputContract(BenchReadinessRenderFastqAdapterOutputContractArgs),
    #[command(name = "render-fastq-command-adapter-coverage")]
    RenderFastqCommandAdapterCoverage(BenchReadinessRenderFastqCommandAdapterCoverageArgs),
    #[command(name = "render-fastq-comparable-metrics")]
    RenderFastqComparableMetrics(BenchReadinessRenderFastqComparableMetricsArgs),
    #[command(name = "render-fastq-corpus-assignment")]
    RenderFastqCorpusAssignment(BenchReadinessRenderFastqCorpusAssignmentArgs),
    #[command(name = "render-fastq-normalized-metrics-schema")]
    RenderFastqNormalizedMetricsSchema(BenchReadinessRenderFastqNormalizedMetricsSchemaArgs),
    #[command(name = "render-fastq-parser-coverage")]
    RenderFastqParserCoverage(BenchReadinessRenderFastqParserCoverageArgs),
    #[command(name = "render-fastq-report-map")]
    RenderFastqReportMap(BenchReadinessRenderFastqReportMapArgs),
    #[command(name = "render-fastq-tool-serving-map")]
    RenderFastqToolServingMap(BenchReadinessRenderFastqToolServingMapArgs),
    #[command(name = "render-bam-tool-serving-map")]
    RenderBamToolServingMap(BenchReadinessRenderBamToolServingMapArgs),
    #[command(name = "render-vcf-tool-serving-map")]
    RenderVcfToolServingMap(BenchReadinessRenderVcfToolServingMapArgs),
    #[command(name = "render-missing-benchmark-pairs")]
    RenderMissingBenchmarkPairs(BenchReadinessRenderMissingBenchmarkPairsArgs),
    #[command(name = "render-stage-registry-extra-pairs")]
    RenderStageRegistryExtraPairs(BenchReadinessRenderStageRegistryExtraPairsArgs),
    #[command(name = "validate-tool-execution-modes")]
    ValidateToolExecutionModes(BenchReadinessValidateToolExecutionModesArgs),
    #[command(name = "render-tool-id-normalization")]
    RenderToolIdNormalization(BenchReadinessRenderToolIdNormalizationArgs),
    #[command(name = "validate-tool-families")]
    ValidateToolFamilies(BenchReadinessValidateToolFamiliesArgs),
    #[command(name = "render-unregistered-benchmark-pairs")]
    RenderUnregisteredBenchmarkPairs(BenchReadinessRenderUnregisteredBenchmarkPairsArgs),
    #[command(name = "render-orphan-tools")]
    RenderOrphanTools(BenchReadinessRenderOrphanToolsArgs),
    #[command(name = "render-undercovered-stages")]
    RenderUndercoveredStages(BenchReadinessRenderUndercoveredStagesArgs),
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAdapterMissingInputTestsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderCommandsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderCommandArgvArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageToolContainersArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageToolAssetsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageToolResourcesArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamAdapterOutputContractArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamStageDecisionTableArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamCommandAdapterCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamCorpusAssignmentArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderCorpusIncompatibilityArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderCorpusCentricReportArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBenchmarkReadinessDashboardArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageToolBenchmarkReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamComparableMetricsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamNormalizedMetricsSchemaArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamParserCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamReportMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderExpectedBenchmarkResultsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderMissingResultReportArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderPairReadinessArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageCentricReportArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderToolCentricReportArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderParserCompletenessGateArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderCorpusAssetCoverageGateArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderParserFailureTestsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqAdapterOutputContractArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqCommandAdapterCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqComparableMetricsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqCorpusAssignmentArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqNormalizedMetricsSchemaArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqParserCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqReportMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqToolServingMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamToolServingMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfToolServingMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderMissingBenchmarkPairsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageRegistryExtraPairsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessValidateToolExecutionModesArgs {
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderToolIdNormalizationArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessValidateToolFamiliesArgs {
    #[arg(long)]
    pub config: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderUnregisteredBenchmarkPairsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderOrphanToolsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderUndercoveredStagesArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
