use crate::model::EnaFileSource;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadTask {
    pub project: Option<String>,
    pub sample: Option<String>,
    pub accession: String,
    pub source: EnaFileSource,
    pub url: String,
    pub output: PathBuf,
}
