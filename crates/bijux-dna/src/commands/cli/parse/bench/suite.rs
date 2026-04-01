use clap::Args;

#[derive(Debug, Args)]
pub struct BenchRunArgs {
    #[arg(long)]
    pub suite: String,
    #[arg(long, default_value_t = false)]
    pub hpc: bool,
}
