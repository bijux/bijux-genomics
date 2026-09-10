//! Publish copies without modifying immutable execution provenance or checksum records.
use super::admission::PublicationPlan;
use super::model::INVENTORY;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

impl PublicationPlan {
    /// The caller must retain the execution directory before this operation.
    /// Replacements are atomic individually; the full publication is not a transaction.
    pub(crate) fn publish(mut self) -> Result<()> {
        for file in
            self.stable_files.iter().filter(|file| file.as_path() != Path::new(&self.report_name))
        {
            self.publish_file(file)?;
        }
        self.publish_file(Path::new(&self.report_name))?;
        let current_files = self.inventory.files.clone();
        let retired = self.prior.difference(&current_files).cloned().collect::<Vec<_>>();
        // Retain cleanup authority if retirement is interrupted after the new report is visible.
        self.inventory.files.extend(self.prior);
        bijux_dna_infra::atomic_write_json(&self.destination.join(INVENTORY), &self.inventory)?;
        retire_files(&self.destination, &retired)?;
        self.inventory.files = current_files;
        bijux_dna_infra::atomic_write_json(&self.destination.join(INVENTORY), &self.inventory)?;
        Ok(())
    }
    fn publish_file(&self, relative: &Path) -> Result<()> {
        let source = self.execution.join(relative);
        let destination = self.destination.join(relative);
        if relative == Path::new("stage-result.json") {
            let report = self
                .stage_result
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("stage result was not admitted"))?;
            bijux_dna_infra::atomic_write_json(&destination, report)?;
            Ok(())
        } else {
            copy_atomically(&source, &destination)
        }
    }
}

fn copy_atomically(source: &Path, destination: &Path) -> Result<()> {
    bijux_dna_infra::atomic_write_with(destination, |output| {
        let mut input = fs::File::open(source)?;
        std::io::copy(&mut input, output).map(|_| ())
    })
    .with_context(|| format!("publish {}", destination.display()))?;
    Ok(())
}

fn retire_files(destination: &Path, retired: &[PathBuf]) -> Result<()> {
    let mut directories = BTreeSet::new();
    for relative in retired {
        let path = destination.join(relative);
        if path.exists() {
            bijux_dna_infra::remove_file(&path)?;
        }
        for directory in relative.ancestors().skip(1).filter(|path| !path.as_os_str().is_empty()) {
            directories.insert(destination.join(directory));
        }
    }
    // Only empty owned ancestors are removed. Unknown files keep their containing directories.
    for directory in directories.into_iter().rev() {
        if directory.is_dir() && fs::read_dir(&directory)?.next().is_none() {
            bijux_dna_infra::remove_empty_dir(&directory)?;
        }
    }
    Ok(())
}
