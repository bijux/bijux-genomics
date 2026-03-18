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
