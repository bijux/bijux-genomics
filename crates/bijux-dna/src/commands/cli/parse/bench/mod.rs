use clap::Subcommand;

use super::BenchBamCommand;

mod config;
mod corpus_fastq;
mod fastq;
mod local;
mod publication;
mod readiness;
mod suite;

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
    BenchLocalDomainArg, BenchLocalFakeRunFailuresArgs, BenchLocalFakeRunStagesArgs,
    BenchLocalListStagesArgs, BenchLocalMaterializeStageArgs, BenchLocalRenderBenchmarkSummaryArgs,
    BenchLocalRenderCorpusSkipReportArgs, BenchLocalRenderSlurmScriptsArgs,
    BenchLocalRenderSlurmSubmitManifestArgs, BenchLocalRenderStageCommandsArgs,
    BenchLocalRenderToolComparisonTemplateArgs, BenchLocalSimulateDagWatchdogArgs,
    BenchLocalValidateCorpusFixtureArgs, BenchLocalValidateCorpusStageCompatibilityArgs,
    BenchLocalValidateHpcSubmissionReadyArgs, BenchLocalValidatePipelineDagArgs,
    BenchLocalValidateSlurmDependenciesArgs, BenchLocalValidateSlurmScriptBodiesArgs,
    BenchLocalValidateSlurmShellSyntaxArgs, BenchLocalValidateStageResultArgs,
    BenchLocalValidateTaxonomyDatabaseFixtureArgs,
};
pub use self::publication::{
    BenchCorpusFastqPublicationStatusArgs, BenchCorpusFastqPublishedDossiersArgs,
    BenchCorpusFastqReportArgs, BenchPublicationTargetsArgs,
};
pub use self::readiness::{
    BenchReadinessCommand, BenchReadinessRenderBamCommandAdapterCoverageArgs,
    BenchReadinessRenderBamStageDecisionTableArgs, BenchReadinessRenderBamToolServingMapArgs,
    BenchReadinessRenderFastqAdapterOutputContractArgs,
    BenchReadinessRenderFastqCommandAdapterCoverageArgs, BenchReadinessRenderFastqToolServingMapArgs,
    BenchReadinessRenderMissingBenchmarkPairsArgs, BenchReadinessRenderOrphanToolsArgs,
    BenchReadinessRenderStageRegistryExtraPairsArgs, BenchReadinessRenderToolIdNormalizationArgs,
    BenchReadinessRenderUndercoveredStagesArgs, BenchReadinessRenderUnregisteredBenchmarkPairsArgs,
    BenchReadinessValidateToolExecutionModesArgs, BenchReadinessValidateToolFamiliesArgs,
};
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
