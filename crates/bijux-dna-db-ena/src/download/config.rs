use crate::model::{EnaFileSource, EnaSourcePreference};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
            jobs: 8,
            retries: 2,
            source: EnaFileSource::FastqFtp,
            preference: EnaSourcePreference::Ftp,
            dry_run: false,
        }
    }
}
