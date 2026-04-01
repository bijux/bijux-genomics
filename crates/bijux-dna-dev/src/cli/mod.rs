mod command_dispatch;
mod execution_reporting;
mod runner;
mod schema;

pub fn run() -> anyhow::Result<()> {
    runner::run()
}
