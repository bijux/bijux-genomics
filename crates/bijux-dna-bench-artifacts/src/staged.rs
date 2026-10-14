//! Consume disposable staged artifacts into stable publication addresses.
use super::admission::admit_report_identity;
use super::model::{ArtifactOwnership, PublicationSchema, ReportOwner};
use super::paths::{admit_ancestry, admit_relative, collect_files};
use super::prior::admit_prior;
use anyhow::{ensure, Context, Result};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// Admitted publication whose regular source files are consumed, without a retained store.
pub struct StagedPublicationPlan {
    staged: PathBuf,
    destination: PathBuf,
    report_name: String,
    inventory_name: String,
    files: BTreeSet<PathBuf>,
    prior: BTreeSet<PathBuf>,
    inventory: ArtifactOwnership,
}

impl StagedPublicationPlan {
    /// Admit ownership and every stable replacement before consuming any source.
    /// Stage-result addresses are already stable and their bytes are not projected.
    ///
    /// # Errors
    /// Rejects malformed report identity, unsafe paths, foreign ownership and unowned targets.
    pub fn admit(
        staged: &Path,
        destination: &Path,
        owner: &ReportOwner<'_>,
        schema: &PublicationSchema,
    ) -> Result<Self> {
        schema.admit(owner)?;
        let current = (!staged.is_absolute() || !destination.is_absolute())
            .then(std::env::current_dir)
            .transpose()?;
        let absolute = |path: &Path| {
            current.as_ref().map_or_else(|| path.to_path_buf(), |root| root.join(path))
        };
        let source_root = absolute(staged);
        let destination_root = absolute(destination);
        let staged = source_root.as_path();
        let destination = destination_root.as_path();
        admit_ancestry(destination)?;
        admit_ancestry(staged)?;
        admit_relative(Path::new(owner.report_name))?;
        ensure!(
            !source_root.starts_with(&destination_root)
                && !destination_root.starts_with(&source_root),
            "staged publication source and destination roots overlap"
        );
        let files = collect_files(staged)?;
        ensure!(
            files.contains(Path::new(owner.report_name))
                && !files.contains(Path::new(schema.inventory_name)),
            "benchmark staging must contain its report and no ownership receipt"
        );
        admit_report_identity(&staged.join(owner.report_name), owner)?;
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
        let inventory = ArtifactOwnership {
            schema_version: schema.schema_version.to_owned(),
            report_schema: owner.schema.to_owned(),
            stage_id: owner.stage.to_owned(),
            tool_id: owner.tool.to_owned(),
            files: files.clone(),
        };
        Ok(Self {
            staged: staged.to_path_buf(),
            destination: destination.to_path_buf(),
            report_name: owner.report_name.to_owned(),
            inventory_name: schema.inventory_name.to_owned(),
            files,
            prior,
            inventory,
        })
    }

    /// Consume source files with report-last, individually atomic replacements.
    /// Callers own both roots exclusively between admission and publication.
    /// The complete set is not a filesystem transaction; empty staged and retired
    /// directories remain available to the caller's lifecycle owner.
    ///
    /// # Errors
    /// Returns replacement, retirement or inventory publication failures.
    pub fn publish(self) -> Result<()> {
        bijux_dna_infra::ensure_dir(&self.destination)?;
        for file in self.files.iter().filter(|file| file.as_path() != Path::new(&self.report_name))
        {
            self.publish_file(file)?;
        }
        self.publish_file(Path::new(&self.report_name))?;
        for retired in self.prior.difference(&self.files) {
            let path = self.destination.join(retired);
            if path.exists() {
                bijux_dna_infra::remove_file(&path)?;
            }
        }
        bijux_dna_infra::atomic_write_json(
            &self.destination.join(&self.inventory_name),
            &self.inventory,
        )?;
        Ok(())
    }

    fn publish_file(&self, relative: &Path) -> Result<()> {
        let target = self.destination.join(relative);
        bijux_dna_infra::rename(&self.staged.join(relative), &target)
            .with_context(|| format!("publish {}", target.display()))
    }
}
