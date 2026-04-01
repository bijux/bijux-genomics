use anyhow::Result;

use bijux_dna_db_ena::{download::DownloadConfig, EnaRunManifest};

use super::query;
use crate::cli::args::DownloadArgs;

pub(crate) fn execute_download(args: &DownloadArgs) -> Result<(EnaRunManifest, DownloadConfig)> {
    let (manifest, mut config) = query::execute_query(&args.shared)?;
    config.output_dir.clone_from(&args.shared.output_dir);
    config.jobs = args.jobs;
    config.retries = args.retries;
    config.dry_run = args.dry_run;
    Ok((manifest, config))
}
