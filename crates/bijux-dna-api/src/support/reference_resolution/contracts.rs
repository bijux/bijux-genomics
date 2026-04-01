use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct ResolvedReference {
    pub species_id: String,
    pub build_id: String,
    pub fasta: PathBuf,
    pub contig_set_digest: Option<String>,
}

pub(crate) trait ReferenceResolver {
    fn resolve(&self, species_id: &str, build_id: &str) -> Result<ResolvedReference>;
}
