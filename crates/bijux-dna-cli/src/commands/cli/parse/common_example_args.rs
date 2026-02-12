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
