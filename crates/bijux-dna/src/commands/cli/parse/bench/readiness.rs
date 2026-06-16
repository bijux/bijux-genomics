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
    #[command(name = "render-bam-commands")]
    RenderBamCommands(BenchReadinessRenderBamCommandsArgs),
    #[command(name = "render-bam-local-container-smoke")]
    RenderBamLocalContainerSmoke(BenchReadinessRenderBamLocalContainerSmokeArgs),
    #[command(name = "render-bam-corpus-assignment")]
    RenderBamCorpusAssignment(BenchReadinessRenderBamCorpusAssignmentArgs),
    #[command(name = "render-bam-contamination-sex-haplogroups-ready")]
    RenderBamContaminationSexHaplogroupsReady(
        BenchReadinessRenderBamContaminationSexHaplogroupsReadyArgs,
    ),
    #[command(name = "render-bam-kinship-ready")]
    RenderBamKinshipReady(BenchReadinessRenderBamKinshipReadyArgs),
    #[command(name = "render-bam-recalibration-genotyping-ready")]
    RenderBamRecalibrationGenotypingReady(BenchReadinessRenderBamRecalibrationGenotypingReadyArgs),
    #[command(name = "render-bam-damage-authenticity-ready")]
    RenderBamDamageAuthenticityReady(BenchReadinessRenderBamDamageAuthenticityReadyArgs),
    #[command(name = "render-bam-insert-size-gc-bias-ready")]
    RenderBamInsertSizeGcBiasReady(BenchReadinessRenderBamInsertSizeGcBiasReadyArgs),
    #[command(name = "render-bam-overlap-endogenous-ready")]
    RenderBamOverlapEndogenousReady(BenchReadinessRenderBamOverlapEndogenousReadyArgs),
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
    #[command(name = "render-bam-parser-fixture-coverage", alias = "render-bam-parser-coverage")]
    RenderBamParserFixtureCoverage(BenchReadinessRenderBamParserFixtureCoverageArgs),
    #[command(name = "render-bam-report-map")]
    RenderBamReportMap(BenchReadinessRenderBamReportMapArgs),
    #[command(name = "render-bam-all-retained-tools-complete")]
    RenderBamAllRetainedToolsComplete(BenchReadinessRenderBamAllRetainedToolsCompleteArgs),
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
    #[command(name = "render-fastq-parser-fixture-coverage")]
    RenderFastqParserFixtureCoverage(BenchReadinessRenderFastqParserFixtureCoverageArgs),
    #[command(name = "render-fastq-commands")]
    RenderFastqCommands(BenchReadinessRenderFastqCommandsArgs),
    #[command(name = "render-fastq-report-map")]
    RenderFastqReportMap(BenchReadinessRenderFastqReportMapArgs),
    #[command(name = "render-fastq-active-stage-tool-matrix")]
    RenderFastqActiveStageToolMatrix(BenchReadinessRenderFastqActiveStageToolMatrixArgs),
    #[command(name = "render-fastq-local-container-smoke")]
    RenderFastqLocalContainerSmoke(BenchReadinessRenderFastqLocalContainerSmokeArgs),
    #[command(name = "render-fastq-duplicate-stages-ready")]
    RenderFastqDuplicateStagesReady(BenchReadinessRenderFastqDuplicateStagesReadyArgs),
    #[command(name = "render-fastq-filter-stages-ready")]
    RenderFastqFilterStagesReady(BenchReadinessRenderFastqFilterStagesReadyArgs),
    #[command(name = "render-fastq-trim-stages-ready")]
    RenderFastqTrimStagesReady(BenchReadinessRenderFastqTrimStagesReadyArgs),
    #[command(name = "render-fastq-validate-reads-ready")]
    RenderFastqValidateReadsReady(BenchReadinessRenderFastqValidateReadsReadyArgs),
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
    #[command(name = "render-all-domain-expected-result-coverage")]
    RenderAllDomainExpectedResultCoverage(BenchReadinessRenderAllDomainExpectedResultCoverageArgs),
    #[command(name = "render-all-domain-harness-ready")]
    RenderAllDomainHarnessReady(BenchReadinessRenderAllDomainHarnessReadyArgs),
    #[command(name = "render-all-domain-local-job-coverage")]
    RenderAllDomainLocalJobCoverage(BenchReadinessRenderAllDomainLocalJobCoverageArgs),
    #[command(name = "render-all-domain-no-placeholder-command-check")]
    RenderAllDomainNoPlaceholderCommandCheck(
        BenchReadinessRenderAllDomainNoPlaceholderCommandCheckArgs,
    ),
    #[command(name = "render-all-domain-failure-classification")]
    RenderAllDomainFailureClassification(BenchReadinessRenderAllDomainFailureClassificationArgs),
    #[command(name = "render-all-domain-completion-check")]
    RenderAllDomainCompletionCheck(BenchReadinessRenderAllDomainCompletionCheckArgs),
    #[command(name = "render-all-domain-missing-result-test")]
    RenderAllDomainMissingResultTest(BenchReadinessRenderAllDomainMissingResultTestArgs),
    #[command(name = "render-all-domain-parser-collector")]
    RenderAllDomainParserCollector(BenchReadinessRenderAllDomainParserCollectorArgs),
    #[command(name = "render-full-benchmark-result-collector")]
    RenderFullBenchmarkResultCollector(BenchReadinessRenderFullBenchmarkResultCollectorArgs),
    #[command(name = "render-full-benchmark-dashboard")]
    RenderFullBenchmarkDashboard(BenchReadinessRenderFullBenchmarkDashboardArgs),
    #[command(name = "render-full-benchmark-report")]
    RenderFullBenchmarkReport(BenchReadinessRenderFullBenchmarkReportArgs),
    #[command(name = "render-operational-benchmark-ready")]
    RenderOperationalBenchmarkReady(BenchReadinessRenderOperationalBenchmarkReadyArgs),
    #[command(name = "render-all-domain-output-declarations")]
    RenderAllDomainOutputDeclarations(BenchReadinessRenderAllDomainOutputDeclarationsArgs),
    #[command(name = "render-all-domain-commands")]
    RenderAllDomainCommands(BenchReadinessRenderAllDomainCommandsArgs),
    #[command(name = "render-all-domain-active-stage-catalog")]
    RenderAllDomainActiveStageCatalog(BenchReadinessRenderAllDomainActiveStageCatalogArgs),
    #[command(name = "render-all-domain-active-scope-blockers")]
    RenderAllDomainActiveScopeBlockers(BenchReadinessRenderAllDomainActiveScopeBlockersArgs),
    #[command(name = "render-all-domain-active-scope-complete")]
    RenderAllDomainActiveScopeComplete(BenchReadinessRenderAllDomainActiveScopeCompleteArgs),
    #[command(name = "render-all-domain-adapter-coverage")]
    RenderAllDomainAdapterCoverage(BenchReadinessRenderAllDomainAdapterCoverageArgs),
    #[command(name = "render-all-domain-output-contract-coverage")]
    RenderAllDomainOutputContractCoverage(BenchReadinessRenderAllDomainOutputContractCoverageArgs),
    #[command(name = "render-all-domain-parser-fixture-coverage")]
    RenderAllDomainParserFixtureCoverage(BenchReadinessRenderAllDomainParserFixtureCoverageArgs),
    #[command(name = "render-all-domain-report-map-coverage")]
    RenderAllDomainReportMapCoverage(BenchReadinessRenderAllDomainReportMapCoverageArgs),
    #[command(name = "render-all-domain-active-stage-tool-matrix")]
    RenderAllDomainActiveStageToolMatrix(BenchReadinessRenderAllDomainActiveStageToolMatrixArgs),
    #[command(name = "render-all-domain-no-declared-only-rows")]
    RenderAllDomainNoDeclaredOnlyRows(BenchReadinessRenderAllDomainNoDeclaredOnlyRowsArgs),
    #[command(name = "render-all-domain-no-not-benchmark-ready-rows")]
    RenderAllDomainNoNotBenchmarkReadyRows(
        BenchReadinessRenderAllDomainNoNotBenchmarkReadyRowsArgs,
    ),
    #[command(name = "render-all-domain-no-planned-rows")]
    RenderAllDomainNoPlannedRows(BenchReadinessRenderAllDomainNoPlannedRowsArgs),
    #[command(name = "render-all-domain-retained-tools")]
    RenderAllDomainRetainedTools(BenchReadinessRenderAllDomainRetainedToolsArgs),
    #[command(name = "render-stage-tool-alias-check")]
    RenderStageToolAliasCheck(BenchReadinessRenderStageToolAliasCheckArgs),
    #[command(name = "render-removed-from-scope")]
    RenderRemovedFromScope(BenchReadinessRenderRemovedFromScopeArgs),
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
    #[command(name = "render-vcf-parser-fixture-coverage")]
    RenderVcfParserFixtureCoverage(BenchReadinessRenderVcfParserFixtureCoverageArgs),
    #[command(name = "render-vcf-normalized-metrics-schema")]
    RenderVcfNormalizedMetricsSchema(BenchReadinessRenderVcfNormalizedMetricsSchemaArgs),
    #[command(name = "render-vcf-parser-failure-tests")]
    RenderVcfParserFailureTests(BenchReadinessRenderVcfParserFailureTestsArgs),
    #[command(name = "render-vcf-adapter-missing-input-tests")]
    RenderVcfAdapterMissingInputTests(BenchReadinessRenderVcfAdapterMissingInputTestsArgs),
    #[command(name = "render-vcf-adapters-ready")]
    RenderVcfAdaptersReady(BenchReadinessRenderVcfAdaptersReadyArgs),
    #[command(name = "render-vcf-active-stage-tool-matrix")]
    RenderVcfActiveStageToolMatrix(BenchReadinessRenderVcfActiveStageToolMatrixArgs),
    #[command(name = "render-vcf-local-container-smoke")]
    RenderVcfLocalContainerSmoke(BenchReadinessRenderVcfLocalContainerSmokeArgs),
    #[command(name = "render-vcf-damage-filter-ready")]
    RenderVcfDamageFilterReady(BenchReadinessRenderVcfDamageFilterReadyArgs),
    #[command(name = "render-vcf-filter-ready")]
    RenderVcfFilterReady(BenchReadinessRenderVcfFilterReadyArgs),
    #[command(name = "render-vcf-gl-propagation-ready")]
    RenderVcfGlPropagationReady(BenchReadinessRenderVcfGlPropagationReadyArgs),
    #[command(name = "render-vcf-call-gl-ready")]
    RenderVcfCallGlReady(BenchReadinessRenderVcfCallGlReadyArgs),
    #[command(name = "render-vcf-call-diploid-ready")]
    RenderVcfCallDiploidReady(BenchReadinessRenderVcfCallDiploidReadyArgs),
    #[command(name = "render-vcf-call-pseudohaploid-ready")]
    RenderVcfCallPseudohaploidReady(BenchReadinessRenderVcfCallPseudohaploidReadyArgs),
    #[command(name = "render-vcf-admixture-ready")]
    RenderVcfAdmixtureReady(BenchReadinessRenderVcfAdmixtureReadyArgs),
    #[command(name = "render-vcf-population-structure-ready")]
    RenderVcfPopulationStructureReady(BenchReadinessRenderVcfPopulationStructureReadyArgs),
    #[command(name = "render-vcf-all-retained-tools-complete")]
    RenderVcfAllRetainedToolsComplete(BenchReadinessRenderVcfAllRetainedToolsCompleteArgs),
    #[command(name = "render-vcf-pca-ready")]
    RenderVcfPcaReady(BenchReadinessRenderVcfPcaReadyArgs),
    #[command(name = "render-vcf-imputation-metrics-ready")]
    RenderVcfImputationMetricsReady(BenchReadinessRenderVcfImputationMetricsReadyArgs),
    #[command(name = "render-vcf-stats-ready")]
    RenderVcfStatsReady(BenchReadinessRenderVcfStatsReadyArgs),
    #[command(name = "render-vcf-qc-ready")]
    RenderVcfQcReady(BenchReadinessRenderVcfQcReadyArgs),
    #[command(name = "render-vcf-prepare-reference-panel-ready")]
    RenderVcfPrepareReferencePanelReady(BenchReadinessRenderVcfPrepareReferencePanelReadyArgs),
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
    #[command(name = "render-vcf-call-ready")]
    RenderVcfCallReady(BenchReadinessRenderVcfCallReadyArgs),
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
pub struct BenchReadinessRenderBamCommandsArgs {
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
pub struct BenchReadinessRenderBamContaminationSexHaplogroupsReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamKinshipReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamRecalibrationGenotypingReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamDamageAuthenticityReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamInsertSizeGcBiasReadyArgs {
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
pub struct BenchReadinessRenderVcfParserFixtureCoverageArgs {
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
pub struct BenchReadinessRenderBamParserFixtureCoverageArgs {
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
pub struct BenchReadinessRenderBamAllRetainedToolsCompleteArgs {
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
pub struct BenchReadinessRenderVcfActiveStageToolMatrixArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfLocalContainerSmokeArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamLocalContainerSmokeArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfDamageFilterReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfFilterReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfGlPropagationReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfCallGlReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfCallReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfCallDiploidReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfCallPseudohaploidReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfAdmixtureReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfPopulationStructureReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfAllRetainedToolsCompleteArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderBamOverlapEndogenousReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfPcaReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfImputationMetricsReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfStatsReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfQcReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderVcfPrepareReferencePanelReadyArgs {
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
pub struct BenchReadinessRenderFastqParserFixtureCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqCommandsArgs {
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
pub struct BenchReadinessRenderFastqActiveStageToolMatrixArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqLocalContainerSmokeArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqDuplicateStagesReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqFilterStagesReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqTrimStagesReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFastqValidateReadsReadyArgs {
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
pub struct BenchReadinessRenderAllDomainExpectedResultCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainHarnessReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainLocalJobCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainFailureClassificationArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainCompletionCheckArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainMissingResultTestArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainParserCollectorArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFullBenchmarkResultCollectorArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFullBenchmarkDashboardArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderFullBenchmarkReportArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderOperationalBenchmarkReadyArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainOutputDeclarationsArgs {
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
pub struct BenchReadinessRenderAllDomainActiveStageCatalogArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainAdapterCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainActiveScopeBlockersArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainActiveScopeCompleteArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainOutputContractCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainParserFixtureCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainReportMapCoverageArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainActiveStageToolMatrixArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainNoDeclaredOnlyRowsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainNoNotBenchmarkReadyRowsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainNoPlaceholderCommandCheckArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainNoPlannedRowsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderAllDomainRetainedToolsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderStageToolAliasCheckArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct BenchReadinessRenderRemovedFromScopeArgs {
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
