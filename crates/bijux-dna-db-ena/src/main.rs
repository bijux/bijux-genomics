mod cli_entrypoint;
mod manifest_store;

fn main() -> anyhow::Result<()> {
    cli_entrypoint::run()
}
