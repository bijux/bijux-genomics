use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadReport {
    pub attempted: usize,
    pub downloaded: usize,
    pub failed: usize,
    pub failed_outputs: Vec<PathBuf>,
}
