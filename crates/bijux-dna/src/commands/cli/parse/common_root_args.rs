#[derive(Debug, Args)]
pub struct PlanArgs {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct CorpusListArgs {
    #[arg(long, default_value_t = false)]
    pub json: bool,
    pub corpus: Option<String>,
}

#[derive(Debug, Args)]
pub struct CorpusMaterializeArgs {
    #[arg(
        long,
        value_name = "PATH",
        default_value = "configs/runtime/corpora/corpus-01.toml"
    )]
    pub spec: PathBuf,
    #[arg(long, value_name = "PATH")]
    pub root: Option<PathBuf>,
    #[arg(long, default_value_t = 4)]
    pub jobs: usize,
    #[arg(long, default_value_t = 2)]
    pub retries: usize,
    #[arg(long, default_value_t = false)]
    pub dry_run: bool,
}
