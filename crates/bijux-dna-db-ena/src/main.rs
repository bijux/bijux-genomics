mod cli_entrypoint;

fn main() -> anyhow::Result<()> {
    cli_entrypoint::run()
}
