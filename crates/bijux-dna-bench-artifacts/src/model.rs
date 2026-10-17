//! File ownership and publication identities, separate from scientific report schemas.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(super) const EXECUTIONS: &str = "executions";

pub struct ReportOwner<'a> {
    pub report_name: &'a str,
    pub schema: &'a str,
    pub stage: &'a str,
    pub tool: &'a str,
}

#[derive(Serialize, Deserialize)]
pub(super) struct ArtifactOwnership {
    pub(super) schema_version: String,
    pub(super) report_schema: String,
    pub(super) stage_id: String,
    pub(super) tool_id: String,
    pub(super) files: BTreeSet<PathBuf>,
}

pub struct PublishedArtifactPaths<'a> {
    pub repository: &'a Path,
    pub execution: &'a Path,
    pub destination: &'a Path,
}

impl PublishedArtifactPaths<'_> {
    /// Project an execution artifact onto its stable published address.
    ///
    /// # Errors
    /// Rejects foreign or escaping execution paths.
    pub fn path(&self, artifact: &Path) -> anyhow::Result<String> {
        let relative = artifact.strip_prefix(self.execution)?;
        if !relative.as_os_str().is_empty() {
            super::paths::admit_relative(relative)?;
        }
        let published = self.destination.join(relative);
        Ok(crate::contract::path_relative_to_repo(self.repository, &published))
    }
}

/// Persisted ownership protocol selected by a benchmark capability.
#[derive(Clone, Copy)]
pub struct PublicationSchema {
    pub inventory_name: &'static str,
    pub schema_version: &'static str,
}

impl PublicationSchema {
    pub(super) fn admit(&self, owner: &ReportOwner<'_>) -> anyhow::Result<()> {
        super::paths::admit_relative(Path::new(self.inventory_name))?;
        anyhow::ensure!(
            Path::new(self.inventory_name).components().count() == 1
                && ![EXECUTIONS, "stage-result.json", owner.report_name]
                    .contains(&self.inventory_name)
                && !self.schema_version.trim().is_empty(),
            "invalid benchmark publication schema"
        );
        Ok(())
    }
}
