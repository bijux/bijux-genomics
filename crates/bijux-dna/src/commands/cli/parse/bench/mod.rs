use clap::Subcommand;

use super::BenchBamCommand;

mod active_scope;
mod config;
mod corpus_fastq;
mod fastq;
mod local;
mod matrix;
mod paths;
mod publication;
mod readiness;
mod schema_validation;
mod suite;

pub use self::active_scope::{BenchActiveScopeCommand, BenchActiveScopeValidateArgs};
pub use self::config::{
    BenchConfigCommand, BenchConfigJsonArgs, BenchConfigValidateArgs,
    BenchNormalizeWorkspaceLayoutArgs, BenchRepoChecksArgs, BenchWorkspaceValueArgs,
    BenchWriteScreenTaxonomyDatabaseLineageArgs,
};
pub use self::corpus_fastq::BenchCorpusFastqArgs;
pub use self::fastq::BenchFastqCommand;
pub use self::fastq::{
    BenchFastqClusterOtusArgs, BenchFastqCorrectArgs, BenchFastqDepleteHostArgs,
    BenchFastqDepleteReferenceContaminantsArgs, BenchFastqDepleteRrnaArgs,
    BenchFastqDetectAdaptersArgs, BenchFastqFilterArgs, BenchFastqFilterLowComplexityArgs,
    BenchFastqIndexReferenceArgs, BenchFastqInferAsvsArgs, BenchFastqMergeArgs,
    BenchFastqNormalizeAbundanceArgs, BenchFastqNormalizePrimersArgs, BenchFastqPreprocessArgs,
    BenchFastqProfileOverrepresentedArgs, BenchFastqProfileReadLengthsArgs, BenchFastqQcPostArgs,
    BenchFastqRemoveChimerasArgs, BenchFastqRemoveDuplicatesArgs, BenchFastqScreenArgs,
    BenchFastqStatsArgs, BenchFastqTrimArgs, BenchFastqTrimPolygArgs,
    BenchFastqTrimTerminalDamageArgs, BenchFastqUmiArgs, BenchFastqValidateArgs,
};
pub use self::local::{
    BenchLocalCheckManifestCompletionArgs, BenchLocalCheckOutputCompletionArgs,
    BenchLocalCollectRuntimeMetricsArgs, BenchLocalCommand, BenchLocalDagWatchdogScenarioArg,
    BenchLocalDomainArg, BenchLocalExecuteAllDomainBenchmarkResultArgs,
    BenchLocalExecuteEssentialPipelineNodeArgs, BenchLocalFakeRunAllDomainFailuresArgs,
    BenchLocalFakeRunAllDomainsArgs, BenchLocalFakeRunEssentialPipelinesArgs,
    BenchLocalFakeRunFailuresArgs, BenchLocalFakeRunStagesArgs, BenchLocalJudgeTaxonomyOutputArgs,
    BenchLocalListStagesArgs, BenchLocalMaterializeStageArgs,
    BenchLocalRenderAllDomainSlurmScriptsArgs, BenchLocalRenderAllDomainSlurmSubmitManifestArgs,
    BenchLocalRenderBenchmarkSummaryArgs, BenchLocalRenderCorpusSkipReportArgs,
    BenchLocalRenderSlurmScriptsArgs, BenchLocalRenderSlurmSubmitManifestArgs,
    BenchLocalRenderStageCommandsArgs, BenchLocalRenderToolComparisonTemplateArgs,
    BenchLocalRenderVcfSmokeRootArgs, BenchLocalRenderVcfStageCatalogArgs,
    BenchLocalRenderVcfStageMatrixArgs, BenchLocalRunRealSmokeCoreSubsetArgs,
    BenchLocalRunVcfAdmixtureSmokeArgs, BenchLocalRunVcfCallDiploidSmokeArgs,
    BenchLocalRunVcfCallGlSmokeArgs, BenchLocalRunVcfCallPseudohaploidSmokeArgs,
    BenchLocalRunVcfCallSmokeArgs, BenchLocalRunVcfDamageFilterSmokeArgs,
    BenchLocalRunVcfDemographySmokeArgs, BenchLocalRunVcfFilterSmokeArgs,
    BenchLocalRunVcfGlPropagationSmokeArgs, BenchLocalRunVcfIbdSmokeArgs,
    BenchLocalRunVcfImputationMetricsSmokeArgs, BenchLocalRunVcfImputeSmokeArgs,
    BenchLocalRunVcfPcaSmokeArgs, BenchLocalRunVcfPhasingSmokeArgs,
    BenchLocalRunVcfPopulationStructureSmokeArgs, BenchLocalRunVcfPrepareReferencePanelSmokeArgs,
    BenchLocalRunVcfQcSmokeArgs, BenchLocalRunVcfRohSmokeArgs, BenchLocalRunVcfStatsSmokeArgs,
    BenchLocalSimulateDagWatchdogArgs, BenchLocalStageListDomainArg,
    BenchLocalValidateAllDomainSlurmResultPathsArgs,
    BenchLocalValidateAllDomainSlurmScriptBodiesArgs,
    BenchLocalValidateAllDomainSlurmShellSyntaxArgs, BenchLocalValidateCorpusFixtureArgs,
    BenchLocalValidateCorpusStageCompatibilityArgs, BenchLocalValidateHpcSubmissionReadyArgs,
    BenchLocalValidatePipelineDagArgs, BenchLocalValidateSlurmDependenciesArgs,
    BenchLocalValidateSlurmScriptBodiesArgs, BenchLocalValidateSlurmShellSyntaxArgs,
    BenchLocalValidateStageResultArgs, BenchLocalValidateTaxonomyDatabaseFixtureArgs,
    BenchLocalValidateVcfNoEmptyOutputArgs, BenchLocalValidateVcfReferenceCompatibilityArgs,
    BenchLocalValidateVcfSampleCompatibilityArgs, BenchLocalValidateVcfSmokeSuiteReadyArgs,
    BenchLocalValidateVcfStageCatalogReadyArgs,
};
pub use self::matrix::{BenchMatrixDomainArg, BenchValidateMatrixArgs};
pub use self::paths::{BenchPathsCleanupProofArgs, BenchPathsCommand, BenchPathsValidateArgs};
pub use self::publication::{
    BenchCorpusFastqPublicationStatusArgs, BenchCorpusFastqPublishedDossiersArgs,
    BenchCorpusFastqReportArgs, BenchPublicationTargetsArgs,
};
pub use self::readiness::{
    BenchReadinessCommand, BenchReadinessRenderAdapterMissingInputTestsArgs,
    BenchReadinessRenderAllDomainActiveScopeBlockersArgs,
    BenchReadinessRenderAllDomainActiveScopeCompleteArgs,
    BenchReadinessRenderAllDomainActiveStageCatalogArgs,
    BenchReadinessRenderAllDomainActiveStageToolMatrixArgs,
    BenchReadinessRenderAllDomainAdapterCoverageArgs, BenchReadinessRenderAllDomainCommandsArgs,
    BenchReadinessRenderAllDomainCompletionCheckArgs,
    BenchReadinessRenderAllDomainExpectedBenchmarkResultsArgs,
    BenchReadinessRenderAllDomainExpectedResultCoverageArgs,
    BenchReadinessRenderAllDomainFailureClassificationArgs,
    BenchReadinessRenderAllDomainHarnessReadyArgs,
    BenchReadinessRenderAllDomainLocalJobCoverageArgs,
    BenchReadinessRenderAllDomainMissingResultTestArgs,
    BenchReadinessRenderAllDomainNoDeclaredOnlyRowsArgs,
    BenchReadinessRenderAllDomainNoNotBenchmarkReadyRowsArgs,
    BenchReadinessRenderAllDomainNoPlaceholderCommandCheckArgs,
    BenchReadinessRenderAllDomainNoPlannedRowsArgs,
    BenchReadinessRenderAllDomainOutputContractCoverageArgs,
    BenchReadinessRenderAllDomainOutputDeclarationsArgs,
    BenchReadinessRenderAllDomainParserCollectorArgs,
    BenchReadinessRenderAllDomainParserFixtureCoverageArgs,
    BenchReadinessRenderAllDomainReportMapCoverageArgs,
    BenchReadinessRenderAllDomainRetainedToolsArgs,
    BenchReadinessRenderAllDomainStageToolTableArgs,
    BenchReadinessRenderBamAdapterOutputContractArgs,
    BenchReadinessRenderBamCommandAdapterCoverageArgs,
    BenchReadinessRenderBamComparableMetricsArgs, BenchReadinessRenderBamCorpusAssignmentArgs,
    BenchReadinessRenderBamNormalizedMetricsSchemaArgs, BenchReadinessRenderBamParserCoverageArgs,
    BenchReadinessRenderBamReportMapArgs, BenchReadinessRenderBamStageDecisionTableArgs,
    BenchReadinessRenderBamToolServingMapArgs, BenchReadinessRenderBenchmarkReadinessDashboardArgs,
    BenchReadinessRenderCommandArgvArgs, BenchReadinessRenderCommandsArgs,
    BenchReadinessRenderCorpusAssetCoverageGateArgs, BenchReadinessRenderCorpusCentricReportArgs,
    BenchReadinessRenderCorpusIncompatibilityArgs,
    BenchReadinessRenderEssentialPipelineCommandsArgs,
    BenchReadinessRenderEssentialPipelineCorpusAssetsArgs,
    BenchReadinessRenderEssentialPipelineFailureIsolationArgs,
    BenchReadinessRenderEssentialPipelinePartialResumeArgs,
    BenchReadinessRenderEssentialPipelineReportMapArgs,
    BenchReadinessRenderEssentialPipelinesReadyArgs,
    BenchReadinessRenderExpectedBenchmarkResultsArgs,
    BenchReadinessRenderFastqActiveStageToolMatrixArgs,
    BenchReadinessRenderFastqAdapterOutputContractArgs,
    BenchReadinessRenderFastqCommandAdapterCoverageArgs,
    BenchReadinessRenderFastqComparableMetricsArgs, BenchReadinessRenderFastqCorpusAssignmentArgs,
    BenchReadinessRenderFastqDuplicateStagesReadyArgs,
    BenchReadinessRenderFastqFilterStagesReadyArgs,
    BenchReadinessRenderFastqLocalContainerSmokeArgs,
    BenchReadinessRenderFastqNormalizedMetricsSchemaArgs,
    BenchReadinessRenderFastqParserCoverageArgs, BenchReadinessRenderFastqReportMapArgs,
    BenchReadinessRenderFastqToolServingMapArgs, BenchReadinessRenderFastqTrimStagesReadyArgs,
    BenchReadinessRenderFastqValidateReadsReadyArgs,
    BenchReadinessRenderFullBenchmarkDashboardArgs, BenchReadinessRenderFullBenchmarkReportArgs,
    BenchReadinessRenderFullBenchmarkResultCollectorArgs,
    BenchReadinessRenderMissingBenchmarkPairsArgs, BenchReadinessRenderMissingResultReportArgs,
    BenchReadinessRenderOperationalBenchmarkReadyArgs, BenchReadinessRenderOrphanToolsArgs,
    BenchReadinessRenderPairReadinessArgs, BenchReadinessRenderParserCompletenessGateArgs,
    BenchReadinessRenderParserFailureTestsArgs, BenchReadinessRenderRemovedFromScopeArgs,
    BenchReadinessRenderStageCentricReportArgs, BenchReadinessRenderStageRegistryExtraPairsArgs,
    BenchReadinessRenderStageToolAliasCheckArgs, BenchReadinessRenderStageToolAssetsArgs,
    BenchReadinessRenderStageToolBenchmarkReadyArgs, BenchReadinessRenderStageToolContainersArgs,
    BenchReadinessRenderStageToolResourcesArgs, BenchReadinessRenderToolCentricReportArgs,
    BenchReadinessRenderToolIdNormalizationArgs, BenchReadinessRenderUndercoveredStagesArgs,
    BenchReadinessRenderUnregisteredBenchmarkPairsArgs,
    BenchReadinessRenderVcfActiveStageToolMatrixArgs,
    BenchReadinessRenderVcfAdapterMissingInputTestsArgs,
    BenchReadinessRenderVcfAdapterOutputCoverageArgs, BenchReadinessRenderVcfAdaptersReadyArgs,
    BenchReadinessRenderVcfAdmixtureReadyArgs, BenchReadinessRenderVcfAllRetainedToolsCompleteArgs,
    BenchReadinessRenderVcfAngsdAdapterArgs, BenchReadinessRenderVcfBcftoolsAdapterArgs,
    BenchReadinessRenderVcfBeagleAdapterArgs, BenchReadinessRenderVcfCallDiploidReadyArgs,
    BenchReadinessRenderVcfCallGlReadyArgs, BenchReadinessRenderVcfCallPseudohaploidReadyArgs,
    BenchReadinessRenderVcfCallReadyArgs, BenchReadinessRenderVcfCommandsArgs,
    BenchReadinessRenderVcfComparableMetricsArgs, BenchReadinessRenderVcfDamageFilterReadyArgs,
    BenchReadinessRenderVcfDescentFamilyAdapterArgs, BenchReadinessRenderVcfEagleAdapterArgs,
    BenchReadinessRenderVcfEigensoftAdapterArgs,
    BenchReadinessRenderVcfExpectedBenchmarkResultsArgs, BenchReadinessRenderVcfFilterReadyArgs,
    BenchReadinessRenderVcfGlPropagationReadyArgs,
    BenchReadinessRenderVcfImputationFamilyAdapterArgs,
    BenchReadinessRenderVcfImputationMetricsReadyArgs,
    BenchReadinessRenderVcfLocalContainerSmokeArgs,
    BenchReadinessRenderVcfMatrixRegistryConsistencyArgs,
    BenchReadinessRenderVcfMissingResultReportArgs,
    BenchReadinessRenderVcfNormalizedMetricsSchemaArgs, BenchReadinessRenderVcfOrphanToolsArgs,
    BenchReadinessRenderVcfParserFailureTestsArgs,
    BenchReadinessRenderVcfParserFixtureCoverageArgs,
    BenchReadinessRenderVcfParsersReportReadyArgs, BenchReadinessRenderVcfPcaReadyArgs,
    BenchReadinessRenderVcfPlink2AdapterArgs, BenchReadinessRenderVcfPlinkAdapterArgs,
    BenchReadinessRenderVcfPopulationStructureReadyArgs,
    BenchReadinessRenderVcfPrepareReferencePanelReadyArgs, BenchReadinessRenderVcfQcReadyArgs,
    BenchReadinessRenderVcfReportMapArgs, BenchReadinessRenderVcfShapeit5AdapterArgs,
    BenchReadinessRenderVcfStatsReadyArgs, BenchReadinessRenderVcfToolServingMapArgs,
    BenchReadinessRenderVcfUndercoveredStagesArgs, BenchReadinessValidateToolExecutionModesArgs,
    BenchReadinessValidateToolFamiliesArgs,
};
pub use self::schema_validation::{BenchSchemaDomainArg, BenchValidateSchemasArgs};
pub use self::suite::BenchRunArgs;

#[derive(Debug, Subcommand)]
pub enum BenchCommand {
    Config {
        #[command(subcommand)]
        command: BenchConfigCommand,
    },
    Run(BenchRunArgs),
    Status,
    #[command(name = "workspace-value")]
    WorkspaceValue(BenchWorkspaceValueArgs),
    #[command(name = "config-json")]
    ConfigJson(BenchConfigJsonArgs),
    #[command(name = "repo-checks")]
    RepoChecks(BenchRepoChecksArgs),
    #[command(name = "write-screen-taxonomy-database-lineage")]
    WriteScreenTaxonomyDatabaseLineage(BenchWriteScreenTaxonomyDatabaseLineageArgs),
    #[command(name = "validate-matrix")]
    ValidateMatrix(BenchValidateMatrixArgs),
    #[command(name = "validate-schemas")]
    ValidateSchemas(BenchValidateSchemasArgs),
    #[command(name = "active-scope")]
    ActiveScope {
        #[command(subcommand)]
        command: BenchActiveScopeCommand,
    },
    Paths {
        #[command(subcommand)]
        command: BenchPathsCommand,
    },
    #[command(name = "publication-targets")]
    PublicationTargets(BenchPublicationTargetsArgs),
    #[command(name = "corpus-fastq")]
    CorpusFastq(BenchCorpusFastqArgs),
    #[command(name = "normalize-workspace-layout")]
    NormalizeWorkspaceLayout(BenchNormalizeWorkspaceLayoutArgs),
    #[command(name = "corpus-fastq-report")]
    CorpusFastqReport(BenchCorpusFastqReportArgs),
    #[command(name = "corpus-fastq-publication-status")]
    CorpusFastqPublicationStatus(BenchCorpusFastqPublicationStatusArgs),
    #[command(name = "corpus-fastq-published-dossiers")]
    CorpusFastqPublishedDossiers(BenchCorpusFastqPublishedDossiersArgs),
    Readiness {
        #[command(subcommand)]
        command: BenchReadinessCommand,
    },
    Local {
        #[command(subcommand)]
        command: BenchLocalCommand,
    },
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
