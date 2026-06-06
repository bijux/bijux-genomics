#[derive(Debug, Subcommand)]
pub enum FixturesCommand {
    Validate(FixturesValidateArgs),
    #[command(name = "validate-expected")]
    ValidateExpected(FixturesValidateExpectedArgs),
}

#[derive(Debug, Args)]
pub struct FixturesValidateArgs {
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct FixturesValidateExpectedArgs {
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
