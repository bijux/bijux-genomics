use std::path::PathBuf;

/// Discovered normalized read paths; content admission remains a separate operation.
#[derive(Debug, Clone)]
pub struct NormalizedFastqSample {
    pub sample_id: String,
    pub r1: PathBuf,
    pub r2: Option<PathBuf>,
    pub layout: String,
}
