mod runner;
mod schema;

pub fn run() -> anyhow::Result<()> {
    runner::run()
}
