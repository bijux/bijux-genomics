#[derive(Debug, Subcommand)]
pub enum CiCommand {
    Validate {
        #[arg(long, default_value = "artifacts/ci/verify_summary.json")]
        out: PathBuf,
    },
}
