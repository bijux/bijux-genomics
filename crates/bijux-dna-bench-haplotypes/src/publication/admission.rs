//! Preflight stable destinations and retain the source execution as owned evidence.
use super::model::{ArtifactOwnership, ReportOwner, EXECUTIONS, INVENTORY, SCHEMA};
use super::paths::{admit_ancestry, admit_relative, collect_files};
use super::prior::admit_prior;
use anyhow::{ensure, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub(crate) fn execution_parent(destination: &Path) -> Result<PathBuf> {
    let parent = destination.join(EXECUTIONS);
    admit_ancestry(&parent)?;
    Ok(parent)
}

pub(crate) struct PublicationPlan {
    pub(super) stage_result: Option<crate::stage_result::BenchStageResultManifestV1>,
    pub(super) execution: PathBuf,
    pub(super) destination: PathBuf,
    pub(super) report_name: String,
    pub(super) stable_files: BTreeSet<PathBuf>,
    pub(super) prior: BTreeSet<PathBuf>,
    pub(super) inventory: ArtifactOwnership,
}

impl PublicationPlan {
    /// Admit all replacements before touching any previously published file.
    pub(crate) fn admit(
        repository: &Path,
        execution: &Path,
        destination: &Path,
        owner: &ReportOwner<'_>,
    ) -> Result<Self> {
        admit_ancestry(destination)?;
        admit_ancestry(execution)?;
        ensure!(
            execution.parent() == Some(execution_parent(destination)?.as_path()),
            "haplotype execution must be a direct child of its retained execution store"
        );
        admit_relative(Path::new(owner.report_name))?;
        let files = collect_files(execution)?;
        ensure!(
            files.contains(Path::new(owner.report_name)),
            "haplotype execution has no final report"
        );
        ensure!(
            !files.contains(Path::new(INVENTORY))
                && !files.iter().any(|file| file.starts_with(EXECUTIONS)),
            "haplotype execution contains reserved publication paths"
        );
        let stage_result = files
            .contains(Path::new("stage-result.json"))
            .then(|| {
                super::stage_result::project_stable_stage_result(
                    &execution.join("stage-result.json"),
                    &super::PublishedArtifactPaths { repository, execution, destination },
                )
            })
            .transpose()?;
        let prior = admit_prior(destination, owner, &files)?;
        for file in &files {
            let target = destination.join(file);
            admit_ancestry(&target)?;
            ensure!(
                !target.exists() || (target.is_file() && prior.contains(file)),
                "refuse to overwrite unowned haplotype artifact {}",
                target.display()
            );
        }
        for file in &prior {
            admit_relative(file)?;
            let target = destination.join(file);
            admit_ancestry(&target)?;
            ensure!(
                !target.exists() || target.is_file(),
                "prior haplotype artifact is not a regular file"
            );
        }
        let retained_prefix = execution.strip_prefix(destination)?;
        let mut claimed_files = files.clone();
        claimed_files.extend(files.iter().map(|path| retained_prefix.join(path)));
        // Downstream workflow manifests can still reference prior immutable executions.
        // Retirement of these files requires a separate admitted reference inventory.
        claimed_files.extend(prior.iter().filter(|path| path.starts_with(EXECUTIONS)).cloned());
        Ok(Self {
            stage_result,
            execution: execution.to_path_buf(),
            destination: destination.to_path_buf(),
            report_name: owner.report_name.to_owned(),
            stable_files: files,
            prior,
            inventory: ArtifactOwnership {
                schema_version: SCHEMA.to_owned(),
                report_schema: owner.schema.to_owned(),
                stage_id: owner.stage.to_owned(),
                tool_id: owner.tool.to_owned(),
                files: claimed_files,
            },
        })
    }
}
