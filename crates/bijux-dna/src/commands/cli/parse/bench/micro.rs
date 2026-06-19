use clap::Args;

#[derive(Debug, Args)]
pub struct BenchRunMicroArgs {
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
