use anyhow::{Context, Result};
use bijux_dna_db_ena::{
    client::EnaClient,
    download::{build_download_tasks, download_tasks, DownloadConfig},
    model::{EnaFileSource, EnaQuery, EnaResultKind, EnaRunManifest, EnaSourcePreference},
};
use clap::{Parser, Subcommand, ValueEnum};
use std::{fs, path::PathBuf};

#[derive(Debug, Parser)]
#[command(name = "bijux-dna-db-ena")]
#[command(about = "Convenient ENA fetch/downloader for projects and samples")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Query(SharedArgs),
    Download(DownloadArgs),
}

#[derive(Debug, Clone, Parser)]
struct SharedArgs {
    #[arg(long = "project", value_delimiter = ',')]
    projects: Vec<String>,
    #[arg(long = "sample", value_delimiter = ',')]
    samples: Vec<String>,
    #[arg(long = "accession", value_delimiter = ',')]
    accessions: Vec<String>,
    #[arg(long, value_enum, default_value_t = ResultKindArg::ReadRun)]
    result: ResultKindArg,
    #[arg(long, value_enum, default_value_t = SourceArg::FastqFtp)]
    source: SourceArg,
    #[arg(long, value_enum, default_value_t = PreferenceArg::Ftp)]
    prefer: PreferenceArg,
    #[arg(long, default_value = "artifacts/ena")]
    output_dir: PathBuf,
    #[arg(long, default_value = "artifacts/ena/manifest.json")]
    manifest_out: PathBuf,
}

#[derive(Debug, Clone, Parser)]
struct DownloadArgs {
    #[command(flatten)]
    shared: SharedArgs,
    #[arg(long, default_value_t = 8)]
    jobs: usize,
    #[arg(long, default_value_t = 2)]
    retries: usize,
    #[arg(long)]
    dry_run: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum ResultKindArg {
    ReadRun,
    Analysis,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum SourceArg {
    FastqFtp,
    SubmittedFtp,
    SraFtp,
    BamFtp,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum PreferenceArg {
    Ftp,
    Https,
}

impl From<ResultKindArg> for EnaResultKind {
    fn from(value: ResultKindArg) -> Self {
        match value {
            ResultKindArg::ReadRun => Self::ReadRun,
            ResultKindArg::Analysis => Self::Analysis,
        }
    }
}

impl From<SourceArg> for EnaFileSource {
    fn from(value: SourceArg) -> Self {
        match value {
            SourceArg::FastqFtp => Self::FastqFtp,
            SourceArg::SubmittedFtp => Self::SubmittedFtp,
            SourceArg::SraFtp => Self::SraFtp,
            SourceArg::BamFtp => Self::BamFtp,
        }
    }
}

impl From<PreferenceArg> for EnaSourcePreference {
    fn from(value: PreferenceArg) -> Self {
        match value {
            PreferenceArg::Ftp => Self::Ftp,
            PreferenceArg::Https => Self::Https,
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Command::Query(args) => {
            let (manifest, _) = execute_query(&args)?;
            write_manifest(&args.manifest_out, &manifest)?;
            println!(
                "query complete: {} records -> {}",
                manifest.records.len(),
                args.manifest_out.display()
            );
        }
        Command::Download(args) => {
            let (manifest, cfg) = execute_query(&args.shared)?;
            write_manifest(&args.shared.manifest_out, &manifest)?;

            let dl_cfg = DownloadConfig {
                output_dir: args.shared.output_dir.clone(),
                jobs: args.jobs,
                retries: args.retries,
                source: cfg.source,
                preference: cfg.preference,
                dry_run: args.dry_run,
            };
            let tasks = build_download_tasks(&manifest.records, &dl_cfg);
            let report = download_tasks(&tasks, &dl_cfg)?;
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

fn execute_query(args: &SharedArgs) -> Result<(EnaRunManifest, DownloadConfig)> {
    if args.projects.is_empty() && args.samples.is_empty() && args.accessions.is_empty() {
        anyhow::bail!("provide at least one of --project, --sample, or --accession");
    }

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

fn write_manifest(path: &PathBuf, manifest: &EnaRunManifest) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("create manifest directory {}", parent.display()))?;
    }
    let json = serde_json::to_string_pretty(manifest)?;
    fs::write(path, json).with_context(|| format!("write manifest {}", path.display()))
}
