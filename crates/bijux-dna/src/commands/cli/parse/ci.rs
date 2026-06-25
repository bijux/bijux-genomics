#[derive(Debug, Subcommand)]
pub enum CiCommand {
    Validate {
        #[arg(long, default_value = "artifacts/ci/verify_summary.json")]
        out: PathBuf,
    },
    Audit(CiAuditArgs),
    ChangedPaths(CiChangedPathsArgs),
    BudgetCheck(CiBudgetCheckArgs),
    AuditFeatures(CiAuditFeaturesArgs),
    Gate(CiGateArgs),
}

#[derive(Debug, Args)]
pub struct CiAuditArgs {
    #[arg(long, default_value = ".github/workflows/ci.yml")]
    pub workflow: PathBuf,
    #[arg(long)]
    pub no_repeated_target: Option<String>,
    #[arg(long, default_value_t = false)]
    pub slow_tier_manual_only: bool,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CiChangedPathsArgs {
    #[arg(long)]
    pub from_file: PathBuf,
}

#[derive(Debug, Args)]
pub struct CiBudgetCheckArgs {
    #[arg(long, default_value = "fast")]
    pub profile: String,
    #[arg(long)]
    pub budget_file: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CiAuditFeaturesArgs {
    #[arg(long, default_value = "default-fast")]
    pub profile: String,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

#[derive(Debug, Args)]
pub struct CiGateArgs {
    #[arg(long, default_value_t = false)]
    pub fast_no_bleeding: bool,
    #[arg(long, default_value = ".github/workflows/ci.yml")]
    pub workflow: PathBuf,
    #[arg(long)]
    pub budget_file: Option<PathBuf>,
    #[arg(long)]
    pub changed_paths_fixture: Option<PathBuf>,
    #[arg(long)]
    pub out: Option<PathBuf>,
}
