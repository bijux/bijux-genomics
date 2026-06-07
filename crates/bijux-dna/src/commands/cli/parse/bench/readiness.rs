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
    #[command(name = "render-essential-pipeline-corpus-assets")]
    RenderEssentialPipelineCorpusAssets(BenchReadinessRenderEssentialPipelineCorpusAssetsArgs),
    #[command(name = "render-essential-pipeline-failure-isolation")]
    RenderEssentialPipelineFailureIsolation(
        BenchReadinessRenderEssentialPipelineFailureIsolationArgs,
    ),
    #[command(name = "render-essential-pipelines-ready")]
    RenderEssentialPipelinesReady(BenchReadinessRenderEssentialPipelinesReadyArgs),
    #[command(name = "render-essential-pipeline-report-map")]
    RenderEssentialPipelineReportMap(BenchReadinessRenderEssentialPipelineReportMapArgs),
    #[command(name = "render-essential-pipeline-partial-resume")]
    RenderEssentialPipelinePartialResume(BenchReadinessRenderEssentialPipelinePartialResumeArgs),
    #[command(name = "render-essential-pipeline-commands")]
    RenderEssentialPipelineCommands(BenchReadinessRenderEssentialPipelineCommandsArgs),
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
    #[command(name = "render-all-domain-expected-benchmark-results")]
    RenderAllDomainExpectedBenchmarkResults(
        BenchReadinessRenderAllDomainExpectedBenchmarkResultsArgs,
    ),
    #[command(name = "render-all-domain-commands")]
    RenderAllDomainCommands(BenchReadinessRenderAllDomainCommandsArgs),
    #[command(name = "render-all-domain-stage-tool-table")]
    RenderAllDomainStageToolTable(BenchReadinessRenderAllDomainStageToolTableArgs),
    #[command(name = "render-vcf-comparable-metrics")]
    RenderVcfComparableMetrics(BenchReadinessRenderVcfComparableMetricsArgs),
    #[command(name = "render-vcf-expected-benchmark-results")]
    RenderVcfExpectedBenchmarkResults(BenchReadinessRenderVcfExpectedBenchmarkResultsArgs),
    #[command(name = "render-vcf-missing-result-report")]
    RenderVcfMissingResultReport(BenchReadinessRenderVcfMissingResultReportArgs),
    #[command(name = "render-vcf-report-map")]
    RenderVcfReportMap(BenchReadinessRenderVcfReportMapArgs),
    #[command(name = "render-vcf-parsers-report-ready")]
    RenderVcfParsersReportReady(BenchReadinessRenderVcfParsersReportReadyArgs),
    #[command(name = "render-vcf-parser-coverage")]
    RenderVcfParserCoverage(BenchReadinessRenderVcfParserCoverageArgs),
    #[command(name = "render-vcf-normalized-metrics-schema")]
    RenderVcfNormalizedMetricsSchema(BenchReadinessRenderVcfNormalizedMetricsSchemaArgs),
    #[command(name = "render-vcf-parser-failure-tests")]
    RenderVcfParserFailureTests(BenchReadinessRenderVcfParserFailureTestsArgs),
    #[command(name = "render-vcf-adapter-missing-input-tests")]
    RenderVcfAdapterMissingInputTests(BenchReadinessRenderVcfAdapterMissingInputTestsArgs),
    #[command(name = "render-vcf-adapters-ready")]
    RenderVcfAdaptersReady(BenchReadinessRenderVcfAdaptersReadyArgs),
    #[command(name = "render-vcf-adapter-output-coverage")]
    RenderVcfAdapterOutputCoverage(BenchReadinessRenderVcfAdapterOutputCoverageArgs),
    #[command(name = "render-vcf-commands")]
    RenderVcfCommands(BenchReadinessRenderVcfCommandsArgs),
    #[command(name = "render-vcf-angsd-adapter")]
    RenderVcfAngsdAdapter(BenchReadinessRenderVcfAngsdAdapterArgs),
    #[command(name = "render-vcf-descent-family-adapter")]
    RenderVcfDescentFamilyAdapter(BenchReadinessRenderVcfDescentFamilyAdapterArgs),
    #[command(name = "render-vcf-eigensoft-adapter")]
    RenderVcfEigensoftAdapter(BenchReadinessRenderVcfEigensoftAdapterArgs),
    #[command(name = "render-vcf-shapeit5-adapter")]
    RenderVcfShapeit5Adapter(BenchReadinessRenderVcfShapeit5AdapterArgs),
    #[command(name = "render-vcf-eagle-adapter")]
    RenderVcfEagleAdapter(BenchReadinessRenderVcfEagleAdapterArgs),
    #[command(name = "render-vcf-beagle-adapter")]
    RenderVcfBeagleAdapter(BenchReadinessRenderVcfBeagleAdapterArgs),
    #[command(name = "render-vcf-imputation-family-adapter")]
    RenderVcfImputationFamilyAdapter(BenchReadinessRenderVcfImputationFamilyAdapterArgs),
    #[command(name = "render-vcf-plink-adapter")]
    RenderVcfPlinkAdapter(BenchReadinessRenderVcfPlinkAdapterArgs),
    #[command(name = "render-vcf-plink2-adapter")]
    RenderVcfPlink2Adapter(BenchReadinessRenderVcfPlink2AdapterArgs),
    #[command(name = "render-vcf-bcftools-adapter")]
    RenderVcfBcftoolsAdapter(BenchReadinessRenderVcfBcftoolsAdapterArgs),
    #[command(name = "render-vcf-matrix-registry-consistency")]
    RenderVcfMatrixRegistryConsistency(BenchReadinessRenderVcfMatrixRegistryConsistencyArgs),
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
    #[command(name = "render-vcf-orphan-tools")]
    RenderVcfOrphanTools(BenchReadinessRenderVcfOrphanToolsArgs),
    #[command(name = "render-vcf-undercovered-stages")]
    RenderVcfUndercoveredStages(BenchReadinessRenderVcfUndercoveredStagesArgs),
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
pub struct BenchReadinessRenderVcfComparableMetricsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfExpectedBenchmarkResultsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfMissingResultReportArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfReportMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfParsersReportReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfParserCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfNormalizedMetricsSchemaArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long)]
    pub stage_dir: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfParserFailureTestsArgs {
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
pub struct BenchReadinessRenderVcfAdapterOutputCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfCommandsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfAdapterMissingInputTestsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfAdaptersReadyArgs {
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
pub struct BenchReadinessRenderVcfAngsdAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfDescentFamilyAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfEigensoftAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfShapeit5AdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfEagleAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfBeagleAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfImputationFamilyAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfPlinkAdapterArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfPlink2AdapterArgs {
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
pub struct BenchReadinessRenderEssentialPipelineCorpusAssetsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderEssentialPipelineFailureIsolationArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderEssentialPipelinesReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderEssentialPipelineReportMapArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderEssentialPipelinePartialResumeArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderEssentialPipelineCommandsArgs {
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
pub struct BenchReadinessRenderAllDomainExpectedBenchmarkResultsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainCommandsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainStageToolTableArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfBcftoolsAdapterArgs {
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
pub struct BenchReadinessRenderVcfOrphanToolsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfMatrixRegistryConsistencyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfUndercoveredStagesArgs {
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
