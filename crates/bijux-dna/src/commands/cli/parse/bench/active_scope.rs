use clap::{Args, Subcommand};

#[derive(Debug, Subcommand)]
pub enum BenchActiveScopeCommand {
    Validate(BenchActiveScopeValidateArgs),
}

#[derive(Debug, Args)]
pub struct BenchActiveScopeValidateArgs {
    #[arg(long, default_value_t = false)]
    pub fast: bool,
    #[arg(long)]
    pub output: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
