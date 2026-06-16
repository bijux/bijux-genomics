#[derive(Debug, Subcommand)]
pub enum DevCommand {
    Ci(CiRootArgs),
    Crates(CratesRootArgs),
}

nested_root_command_args!(CratesRootArgs, CratesCommand);

#[derive(Debug, Subcommand)]
pub enum CratesCommand {
    Graph(CratesGraphArgs),
    DomainNoExecution(CratesDomainNoExecutionArgs),
    ParserNoExecution(CratesParserNoExecutionArgs),
}

#[derive(Debug, Args)]
pub struct CratesGraphArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/crate-dependency-map.json")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesDomainNoExecutionArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/domain-no-execution.json")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesParserNoExecutionArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/parser-no-execution.json")]
    pub output: PathBuf,
}
