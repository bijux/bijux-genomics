#[derive(Debug, Args)]
pub struct ExampleRunArgs {
    pub id: String,
    #[arg(long, default_value_t = false)]
    pub hpc: bool,
}

#[derive(Debug, Args)]
pub struct ExampleValidateArgs {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct ExamplePlanArgs {
    pub id: String,
}

#[derive(Debug, Args)]
pub struct ExampleListArgs {
    #[arg(long, default_value_t = false)]
    pub json: bool,
}

#[derive(Debug, Args)]
pub struct DevExamplesScaffoldArgs {
    #[arg(long, default_value = "1xx")]
    pub series: String,
    #[arg(long)]
    pub count: usize,
}
