use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchReadinessCommand {
    #[command(name = "render-fastq-tool-serving-map")]
    RenderFastqToolServingMap(BenchReadinessRenderFastqToolServingMapArgs),
    #[command(name = "render-bam-tool-serving-map")]
    RenderBamToolServingMap(BenchReadinessRenderBamToolServingMapArgs),
    #[command(name = "render-orphan-tools")]
    RenderOrphanTools(BenchReadinessRenderOrphanToolsArgs),
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
pub struct BenchReadinessRenderOrphanToolsArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
