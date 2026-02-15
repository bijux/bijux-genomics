use std::path::PathBuf;

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedReference {
    pub species_id: String,
    pub build_id: String,
    pub fasta: PathBuf,
    pub contig_set_digest: Option<String>,
}

pub trait ReferenceResolver {
    fn resolve(&self, species_id: &str, build_id: &str) -> Result<ResolvedReference>;
}

#[derive(Debug, Default, Clone)]
pub struct LocalReferenceResolver {
    pub root: Option<PathBuf>,
}

impl ReferenceResolver for LocalReferenceResolver {
    fn resolve(&self, species_id: &str, build_id: &str) -> Result<ResolvedReference> {
        let root = self.root.clone().unwrap_or_else(|| {
            std::env::var("BIJUX_REFERENCE_ROOT")
                .map(PathBuf::from)
                .unwrap_or_else(|_| PathBuf::from("assets/reference"))
        });
        let fasta = root.join(species_id).join(build_id).join("reference.fa");
        if !fasta.exists() {
            return Err(anyhow!(
                "reference resolution failed: {}",
                fasta.display()
            ));
        }
        Ok(ResolvedReference {
            species_id: species_id.to_string(),
            build_id: build_id.to_string(),
            fasta,
            contig_set_digest: None,
        })
    }
}
