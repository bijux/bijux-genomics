use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

use bijux_dna_db_ena::{
    download::{DEFAULT_DOWNLOAD_JOBS, DEFAULT_DOWNLOAD_RETRIES},
    EnaFileSource, EnaResultKind, EnaSourcePreference,
};

#[derive(Debug, Parser)]
#[command(name = "bijux-dna-db-ena")]
#[command(about = "Convenient ENA fetch/downloader for projects and samples")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
pub(crate) enum Command {
    Query(SharedArgs),
    Download(DownloadArgs),
}

#[derive(Debug, Clone, Parser)]
pub(crate) struct SharedArgs {
    #[arg(long = "project", value_delimiter = ',')]
    pub(crate) projects: Vec<String>,
    #[arg(long = "sample", value_delimiter = ',')]
    pub(crate) samples: Vec<String>,
    #[arg(long = "accession", value_delimiter = ',')]
    pub(crate) accessions: Vec<String>,
    #[arg(long, value_enum, default_value_t = ResultKindArg::ReadRun)]
    pub(crate) result: ResultKindArg,
    #[arg(long, value_enum, default_value_t = SourceArg::Fastq)]
    pub(crate) source: SourceArg,
    #[arg(long, value_enum, default_value_t = PreferenceArg::Ftp)]
    pub(crate) prefer: PreferenceArg,
    #[arg(long, default_value = "artifacts/ena")]
    pub(crate) output_dir: PathBuf,
    #[arg(long, default_value = "artifacts/ena/manifest.json")]
    pub(crate) manifest_out: PathBuf,
}

#[derive(Debug, Clone, Parser)]
pub(crate) struct DownloadArgs {
    #[command(flatten)]
    pub(crate) shared: SharedArgs,
    #[arg(long, default_value_t = DEFAULT_DOWNLOAD_JOBS)]
    pub(crate) jobs: usize,
    #[arg(long, default_value_t = DEFAULT_DOWNLOAD_RETRIES)]
    pub(crate) retries: usize,
    #[arg(long)]
    pub(crate) dry_run: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum ResultKindArg {
    ReadRun,
    Analysis,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum SourceArg {
    Fastq,
    Submitted,
    Sra,
    Bam,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum PreferenceArg {
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
            SourceArg::Fastq => Self::FastqFtp,
            SourceArg::Submitted => Self::SubmittedFtp,
            SourceArg::Sra => Self::SraFtp,
            SourceArg::Bam => Self::BamFtp,
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
