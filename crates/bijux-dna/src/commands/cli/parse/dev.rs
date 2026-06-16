#[derive(Debug, Subcommand)]
pub enum DevCommand {
    Ci(CiRootArgs),
    Crates(CratesRootArgs),
}

nested_root_command_args!(CratesRootArgs, CratesCommand);

#[derive(Debug, Subcommand)]
pub enum CratesCommand {
    Graph(CratesGraphArgs),
}

#[derive(Debug, Args)]
pub struct CratesGraphArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/crate-dependency-map.json")]
    pub output: PathBuf,
}
