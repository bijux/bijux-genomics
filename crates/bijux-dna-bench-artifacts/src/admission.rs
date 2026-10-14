//! Preflight stable destinations and retain the source execution as owned evidence.
use super::model::{ArtifactOwnership, PublicationSchema, ReportOwner, EXECUTIONS};
use super::paths::{admit_ancestry, admit_relative, collect_files};
use super::prior::admit_prior;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

mod report_identity;
pub(super) use report_identity::admit_report_identity;

/// # Errors
/// Rejects symlinked execution-store ancestry.
pub fn execution_parent(destination: &Path) -> Result<PathBuf> {
    let parent = destination.join(EXECUTIONS);
    admit_ancestry(&parent)?;
    Ok(parent)
}

pub struct PublicationPlan {
    pub(super) inventory_name: String,
    pub(super) stage_result: Option<crate::contract::BenchStageResultManifestV1>,
    pub(super) execution: PathBuf,
    pub(super) destination: PathBuf,
    pub(super) report_name: String,
    pub(super) stable_files: BTreeSet<PathBuf>,
    pub(super) prior: BTreeSet<PathBuf>,
    pub(super) inventory: ArtifactOwnership,
}

impl PublicationPlan {
    /// Admit all replacements before touching any previously published file.
    ///
    /// # Errors
    /// Rejects foreign ownership, unsafe paths, malformed reports and unowned replacements.
    pub fn admit(
        repository: &Path,
        execution: &Path,
        destination: &Path,
        owner: &ReportOwner<'_>,
        schema: &PublicationSchema,
    ) -> Result<Self> {
        admit_relative(Path::new(schema.inventory_name))?;
        ensure!(
            Path::new(schema.inventory_name).components().count() == 1
                && ![EXECUTIONS, "stage-result.json", owner.report_name]
                    .contains(&schema.inventory_name)
                && !schema.schema_version.trim().is_empty(),
            "invalid benchmark publication schema"
        );
        admit_ancestry(destination)?;
        admit_ancestry(execution)?;
        ensure!(
            execution.parent() == Some(execution_parent(destination)?.as_path()),
            "benchmark execution must be a direct child of its retained execution store"
        );
        admit_relative(Path::new(owner.report_name))?;
        let files = collect_files(execution)?;
        ensure!(
            files.contains(Path::new(owner.report_name)),
            "benchmark execution has no final report"
        );
        ensure!(
            !files.contains(Path::new(schema.inventory_name))
                && !files.iter().any(|file| file.starts_with(EXECUTIONS)),
            "benchmark execution contains reserved publication paths"
        );
        admit_report_identity(&execution.join(owner.report_name), owner)?;
        let stage_result = files
            .contains(Path::new("stage-result.json"))
            .then(|| {
                super::stage_result::project_stable_stage_result(
                    &execution.join("stage-result.json"),
                    &super::PublishedArtifactPaths { repository, execution, destination },
                )
            })
            .transpose()?;
        let prior = admit_prior(destination, owner, schema, &files)?;
        for file in &files {
            let target = destination.join(file);
            admit_ancestry(&target)?;
            ensure!(
                !target.exists() || (target.is_file() && prior.contains(file)),
                "refuse to overwrite unowned benchmark artifact {}",
                target.display()
            );
        }
        for file in &prior {
            admit_relative(file)?;
            let target = destination.join(file);
            admit_ancestry(&target)?;
            ensure!(
                !target.exists() || target.is_file(),
                "prior benchmark artifact is not a regular file"
            );
        }
        let retained_prefix = execution.strip_prefix(destination)?;
        let mut claimed_files = files.clone();
        claimed_files.extend(files.iter().map(|path| retained_prefix.join(path)));
        // Downstream workflow manifests can still reference prior immutable executions.
        // Retirement of these files requires a separate admitted reference inventory.
        claimed_files.extend(prior.iter().filter(|path| path.starts_with(EXECUTIONS)).cloned());
        Ok(Self {
            inventory_name: schema.inventory_name.to_owned(),
            stage_result,
            execution: execution.to_path_buf(),
            destination: destination.to_path_buf(),
            report_name: owner.report_name.to_owned(),
            stable_files: files,
            prior,
            inventory: ArtifactOwnership {
                schema_version: schema.schema_version.to_owned(),
                report_schema: owner.schema.to_owned(),
                stage_id: owner.stage.to_owned(),
                tool_id: owner.tool.to_owned(),
                files: claimed_files,
            },
        })
    }
}
