#[derive(Debug, Subcommand)]
pub enum FixturesCommand {
    Validate(FixturesValidateArgs),
}

#[derive(Debug, Args)]
pub struct FixturesValidateArgs {
    #[arg(long, value_name = "CORPUS_ID")]
    pub corpus: String,
    #[arg(long, default_value_t = false)]
    pub json: bool,
}
