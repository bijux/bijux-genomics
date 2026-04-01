mod args;
mod commands;

use clap::Parser;

use self::args::{Cli, Command};
use crate::manifest_store;

pub(crate) fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Query(args) => {
            let (manifest, _) = commands::execute_query(&args)?;
            manifest_store::write_manifest(&args.manifest_out, &manifest)?;
            println!(
                "query complete: {} records -> {}",
                manifest.records.len(),
                args.manifest_out.display()
            );
        }
        Command::Download(args) => {
            let (manifest, dl_cfg) = commands::execute_download(&args)?;
            manifest_store::write_manifest(&args.shared.manifest_out, &manifest)?;
            let report = bijux_dna_db_ena::download_tasks(
                &bijux_dna_db_ena::download::build_download_tasks(&manifest.records, &dl_cfg),
                &dl_cfg,
            )?;
            println!(
                "download summary: attempted={} downloaded={} failed={} dry_run={}",
                report.attempted, report.downloaded, report.failed, dl_cfg.dry_run
            );
            if !report.failed_outputs.is_empty() {
                println!("failed outputs:");
                for path in report.failed_outputs {
                    println!("- {}", path.display());
                }
            }
        }
    }

    Ok(())
}
