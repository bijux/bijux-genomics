use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchReadinessCommand {
    #[command(name = "render-fastq-tool-serving-map")]
    RenderFastqToolServingMap(BenchReadinessRenderFastqToolServingMapArgs),
    #[command(name = "render-bam-tool-serving-map")]
    RenderBamToolServingMap(BenchReadinessRenderBamToolServingMapArgs),
    #[command(name = "render-missing-benchmark-pairs")]
    RenderMissingBenchmarkPairs(BenchReadinessRenderMissingBenchmarkPairsArgs),
    #[command(name = "render-stage-registry-extra-pairs")]
    RenderStageRegistryExtraPairs(BenchReadinessRenderStageRegistryExtraPairsArgs),
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
