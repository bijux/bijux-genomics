//! File ownership and publication identities, separate from scientific report schemas.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) const INVENTORY: &str = "haplotype-artifact-ownership.json";
pub(super) const SCHEMA: &str = "bijux.bench.haplotype_artifact_ownership.v1";
pub(super) const EXECUTIONS: &str = "executions";

pub(crate) struct ReportOwner<'a> {
    pub(crate) report_name: &'a str,
    pub(crate) schema: &'a str,
    pub(crate) stage: &'a str,
    pub(crate) tool: &'a str,
}

#[derive(Serialize, Deserialize)]
pub(super) struct ArtifactOwnership {
    pub(super) schema_version: String,
    pub(super) report_schema: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) files: BTreeSet<PathBuf>,
}

pub(crate) struct PublishedArtifactPaths<'a> {
    pub(crate) repository: &'a Path,
    pub(crate) execution: &'a Path,
    pub(crate) destination: &'a Path,
}

impl PublishedArtifactPaths<'_> {
    /// Project an execution artifact onto its stable published address.
    pub(crate) fn path(&self, artifact: &Path) -> anyhow::Result<String> {
        let relative = artifact.strip_prefix(self.execution)?;
        let published = self.destination.join(relative);
        Ok(crate::stage_result::path_relative_to_repo(self.repository, &published))
    }
}
