#[derive(Debug, Subcommand)]
pub enum DevCommand {
    Ci(CiRootArgs),
    Crates(CratesRootArgs),
}

nested_root_command_args!(CratesRootArgs, CratesCommand);

#[derive(Debug, Subcommand)]
pub enum CratesCommand {
    Graph(CratesGraphArgs),
    CheckCycles(CratesCheckCyclesArgs),
    Gate(CratesGateArgs),
    MetricRegistry(CratesMetricRegistryArgs),
    ResultIdStability(CratesResultIdStabilityArgs),
    DomainNoExecution(CratesDomainNoExecutionArgs),
    ParserNoExecution(CratesParserNoExecutionArgs),
    PlannerNoParser(CratesPlannerNoParserArgs),
    RunnerOwnsProcessExecution(CratesRunnerOwnsProcessExecutionArgs),
}

#[derive(Debug, Args)]
pub struct CratesGraphArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/crate-dependency-map.json")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesCheckCyclesArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/no-crate-cycles.json")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesGateArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/CRATE_SHAPE_FOR_BENCHMARKING_READY.json")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesMetricRegistryArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/metric-registry.tsv")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesResultIdStabilityArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/result-id-stability.json")]
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

#[derive(Debug, Args)]
pub struct CratesPlannerNoParserArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/planner-no-parser.json")]
    pub output: PathBuf,
}

#[derive(Debug, Args)]
pub struct CratesRunnerOwnsProcessExecutionArgs {
    #[arg(long, default_value = "benchmarks/readiness/crates/runner-owns-process-execution.json")]
    pub output: PathBuf,
}
