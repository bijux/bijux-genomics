//! Owner: benchmark local campaign planning.
//! Local campaign planning and preflight; scheduler execution remains outside this service.
use std::collections::BTreeMap;
use std::path::PathBuf;
pub(crate) mod configuration;
pub(crate) mod planning;
pub(crate) mod policies;
pub(crate) mod preflight;
pub(crate) mod resources;
pub(crate) mod templates;
/// Invocation observations and resolved frontend defaults for local campaign services.
#[derive(Debug, Clone)]
pub struct CampaignContext {
    pub workspace_root: PathBuf,
    pub campaign_root: PathBuf,
    pub process_environment: BTreeMap<String, String>,
    pub executable_search_path: Option<std::ffi::OsString>,
    pub timestamp: String,
}

pub use planning::campaign_dry_run;
pub use preflight::campaign_preflight;
#[cfg(test)]
#[path = "../../../tests/contracts/campaign/owner.rs"]
mod tests;
