use clap::{Args, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum BenchMatrixDomainArg {
    Vcf,
}

#[derive(Debug, Args)]
pub struct BenchValidateMatrixArgs {
    #[arg(long, value_enum)]
    pub domain: BenchMatrixDomainArg,
    #[arg(long)]
    pub matrix: Option<std::path::PathBuf>,
    #[arg(long, default_value_t = false)]
    pub strict: bool,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
