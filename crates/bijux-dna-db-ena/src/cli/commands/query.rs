use anyhow::{Context, Result};

use bijux_dna_db_ena::{
    download::DownloadConfig, EnaClient, EnaFileSource, EnaQuery, EnaRunManifest,
    EnaSourcePreference,
};

use super::super::args::SharedArgs;

pub(crate) fn execute_query(args: &SharedArgs) -> Result<(EnaRunManifest, DownloadConfig)> {
    let query = EnaQuery {
        projects: args.projects.clone(),
        samples: args.samples.clone(),
        extra_accessions: args.accessions.clone(),
        result: args.result.into(),
    };
    query.validate().context("validate ENA query selectors")?;

    let client = EnaClient::from_crate_identity().context("create ena client")?;
    let records = client.fetch_records(&query).context("fetch ENA metadata records")?;

    let source: EnaFileSource = args.source.into();
    let preference: EnaSourcePreference = args.prefer.into();

    let manifest = EnaRunManifest { query, source, preference, records };

    let mut config = DownloadConfig::from_defaults(args.output_dir.clone());
    config.source = source;
    config.preference = preference;
    config.dry_run = true;

    Ok((manifest, config))
}
