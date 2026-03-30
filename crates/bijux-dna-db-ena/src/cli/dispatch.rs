use anyhow::{Context, Result};

use bijux_dna_db_ena::{
    download::DownloadConfig, EnaClient, EnaFileSource, EnaQuery, EnaRunManifest,
    EnaSourcePreference,
};

use super::args::{DownloadArgs, SharedArgs};

pub(crate) fn execute_query(args: &SharedArgs) -> Result<(EnaRunManifest, DownloadConfig)> {
    let query = EnaQuery {
        projects: args.projects.clone(),
        samples: args.samples.clone(),
        extra_accessions: args.accessions.clone(),
        result: args.result.into(),
    };
    let client = EnaClient::new("bijux-dna-db-ena/0.1").context("create ena client")?;
    let records = client
        .fetch_records(&query)
        .context("fetch ENA metadata records")?;

    let source: EnaFileSource = args.source.into();
    let preference: EnaSourcePreference = args.prefer.into();

    let manifest = EnaRunManifest {
        query,
        source,
        preference,
        records,
    };

    Ok((
        manifest,
        DownloadConfig {
            output_dir: args.output_dir.clone(),
            jobs: 8,
            retries: 2,
            source,
            preference,
            dry_run: true,
        },
    ))
}

pub(crate) fn execute_download(args: &DownloadArgs) -> Result<(EnaRunManifest, DownloadConfig)> {
    let (manifest, mut config) = execute_query(&args.shared)?;
    config.output_dir = args.shared.output_dir.clone();
    config.jobs = args.jobs;
    config.retries = args.retries;
    config.dry_run = args.dry_run;
    Ok((manifest, config))
}
