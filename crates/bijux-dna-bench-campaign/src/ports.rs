//! Explicit execution and workspace-layout dependencies of corpus campaigns.
use anyhow::Result;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One external invocation. Argument boundaries, current directory and environment remain explicit.
#[derive(Debug, Clone, Copy)]
pub struct CampaignCommand<'a> {
    pub program: &'a str,
    pub arguments: &'a [String],
    pub cwd: Option<&'a Path>,
    pub environment: Option<&'a BTreeMap<String, String>>,
}

/// Execute a command and report the observed process exit code.
/// Implementations must preserve argument boundaries and the declared execution context.
pub trait CampaignExecution: Sync {
    /// # Errors
    /// Return transport/setup failures without converting them into a successful sample result.
    fn execute(&self, command: CampaignCommand<'_>) -> Result<i32>;
}

/// Resolve campaign paths through the caller's canonical workspace configuration.
pub trait CampaignPaths {
    /// # Errors
    /// Reject unsupported stages or incomplete workspace path declarations.
    fn stage_out_root(&self, corpus_id: &str, stage_id: &str) -> Result<PathBuf>;
    /// Configured input bind root for an Apptainer index warmup, when available.
    fn apptainer_bind_root(&self) -> Option<PathBuf>;
}

/// Supply the publication instant after sample execution finishes.
pub trait CampaignClock {
    fn now(&self) -> std::time::SystemTime;
}
