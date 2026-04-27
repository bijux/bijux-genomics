use crate::model::{EnaFileSource, EnaSourcePreference};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub const DEFAULT_DOWNLOAD_JOBS: usize = 8;
pub const DEFAULT_DOWNLOAD_RETRIES: usize = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadConfig {
    pub output_dir: PathBuf,
    pub jobs: usize,
    pub retries: usize,
    pub source: EnaFileSource,
    pub preference: EnaSourcePreference,
    pub dry_run: bool,
}

impl DownloadConfig {
    #[must_use]
    pub fn from_defaults(output_dir: PathBuf) -> Self {
        Self {
            output_dir,
            jobs: DEFAULT_DOWNLOAD_JOBS,
            retries: DEFAULT_DOWNLOAD_RETRIES,
            source: EnaFileSource::FastqFtp,
            preference: EnaSourcePreference::Ftp,
            dry_run: false,
        }
    }

    /// # Errors
    /// Returns an error when the configured output directory is empty or when
    /// the configured job count is zero.
    pub fn validate(&self) -> Result<()> {
        if self.output_dir.as_os_str().is_empty() {
            anyhow::bail!("output_dir must not be empty");
        }
        if self.jobs == 0 {
            anyhow::bail!("jobs must be greater than zero");
        }
        Ok(())
    }
}
