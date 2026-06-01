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
    #[command(name = "materialize-stage")]
    MaterializeStage(BenchLocalMaterializeStageArgs),
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
pub struct BenchLocalMaterializeStageArgs {
    #[arg(long)]
    pub stage_id: String,
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
