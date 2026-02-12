#[derive(Debug, Subcommand)]
pub enum CiCommand {
    Verify {
        #[arg(long, default_value = "artifacts/ci/verify_summary.json")]
        out: PathBuf,
    },
}
