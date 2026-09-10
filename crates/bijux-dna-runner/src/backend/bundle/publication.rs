//! Owner: recoverable ciphertext/sidecar publication under a non-waiting pair lease.
use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{anyhow, ensure, Context, Result};
use bijux_dna_infra::FileLock;
use serde::{Deserialize, Serialize};

use super::storage::{digest_file_sha256, sidecar_path_for, temp_output_path};

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct PublicationIntent {
    version: u32,
    previous_ciphertext: Option<String>,
    previous_sidecar: Option<String>,
    ciphertext: String,
    sidecar: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum PublicationPoint {
    Prepared,
    Ciphertext,
    Sidecar,
    Committed,
}

pub(super) struct BundlePair {
    output: PathBuf,
    sidecar: PathBuf,
    journal: PathBuf,
    _lease: FileLock,
}

impl BundlePair {
    pub(super) fn acquire(output: &Path) -> Result<Self> {
        let (lock, journal) = locations(output)?;
        regular_file(&lock)?;
        let lease =
            FileLock::try_acquire(&lock).context("bundle pair is busy or cannot be leased")?;
        Self::from_lease(output, journal, lease)
    }

    pub(super) fn acquire_reader(output: &Path) -> Result<Option<Self>> {
        let (lock, journal) = locations(output)?;
        regular_file(&lock)?;
        let Some(lease) = FileLock::try_acquire_existing(&lock)? else {
            ensure!(!journal.try_exists()?, "publication journal has no coordination lease");
            // Legacy transferred bundles need no write access beside their input files.
            return Ok(None);
        };
        Self::from_lease(output, journal, lease).map(Some)
    }

    fn from_lease(output: &Path, journal: PathBuf, lease: FileLock) -> Result<Self> {
        let pair = Self {
            output: output.to_path_buf(),
            sidecar: sidecar_path_for(output),
            journal,
            _lease: lease,
        };
        pair.recover()?;
        regular_file(&pair.output)?;
        regular_file(&pair.sidecar)?;
        Ok(pair)
    }

    pub(super) fn publish<T>(
        &self,
        prepare: impl FnOnce(&Path) -> Result<(T, Vec<u8>)>,
        mut observe: impl FnMut(PublicationPoint) -> Result<()>,
    ) -> Result<T> {
        let parent = self.journal.parent().context("publication parent is absent")?;
        let staged = bijux_dna_infra::temp_dir_in(parent, ".bundle-publication-")?;
        let ciphertext = staged.path().join("new-ciphertext");
        let sidecar = staged.path().join("new-sidecar");
        let (value, bytes) = prepare(&ciphertext)?;
        bijux_dna_infra::write_bytes(&sidecar, bytes)?;
        let intent = PublicationIntent {
            version: 1,
            previous_ciphertext: backup(&self.output, &staged.path().join("old-ciphertext"))?,
            previous_sidecar: backup(&self.sidecar, &staged.path().join("old-sidecar"))?,
            ciphertext: digest_file_sha256(&ciphertext)?,
            sidecar: digest_file_sha256(&sidecar)?,
        };
        bijux_dna_infra::write_bytes(
            staged.path().join("intent.json"),
            serde_json::to_vec(&intent)?,
        )?;
        for name in ["new-ciphertext", "new-sidecar", "intent.json"] {
            fs::File::open(staged.path().join(name))?.sync_all()?;
        }
        ensure!(!self.journal.try_exists()?, "bundle publication journal already exists");
        bijux_dna_infra::rename(staged.path(), &self.journal)
            .context("admit prepared bundle publication")?;
        let publication = (|| {
            observe(PublicationPoint::Prepared)?;
            bijux_dna_infra::rename(&self.journal.join("new-ciphertext"), &self.output)?;
            observe(PublicationPoint::Ciphertext)?;
            bijux_dna_infra::rename(&self.journal.join("new-sidecar"), &self.sidecar)?;
            observe(PublicationPoint::Sidecar)?;
            bijux_dna_infra::atomic_write_bytes(
                &self.journal.join("committed"),
                b"bundle pair committed v1\n",
            )?;
            observe(PublicationPoint::Committed)?;
            self.recover()
        })();
        match publication {
            Ok(()) => Ok(value),
            Err(error) => match self.recover() {
                Ok(()) => Err(error),
                Err(recovery) => {
                    Err(anyhow!("{error:#}; bundle pair recovery also failed: {recovery:#}"))
                }
            },
        }
    }

    fn recover(&self) -> Result<()> {
        match self.journal.symlink_metadata() {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
            Ok(metadata) => ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "bundle publication journal must be an ordinary directory"
            ),
        }
        let files = journal_files(&self.journal)?;
        let intent: PublicationIntent =
            serde_json::from_slice(&fs::read(self.journal.join("intent.json"))?)
                .context("admit bundle publication intent")?;
        ensure!(intent.version == 1, "unsupported bundle publication intent");
        for digest in [&intent.ciphertext, &intent.sidecar]
            .into_iter()
            .chain(intent.previous_ciphertext.iter())
            .chain(intent.previous_sidecar.iter())
        {
            ensure!(
                digest.len() == 64
                    && digest.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
                "invalid publication content identity"
            );
        }
        if files.iter().any(|path| path.file_name().is_some_and(|name| name == "committed")) {
            ensure!(
                fs::read(self.journal.join("committed"))? == b"bundle pair committed v1\n",
                "invalid bundle publication completion marker"
            );
            ensure!(
                content_identity(&self.output)?.as_ref() == Some(&intent.ciphertext)
                    && content_identity(&self.sidecar)?.as_ref() == Some(&intent.sidecar),
                "committed bundle pair no longer matches its publication intent"
            );
        } else {
            // Admit both restorations before changing either visible member.
            admit_restoration(
                &self.output,
                &self.journal.join("old-ciphertext"),
                intent.previous_ciphertext.as_deref(),
                &intent.ciphertext,
            )?;
            admit_restoration(
                &self.sidecar,
                &self.journal.join("old-sidecar"),
                intent.previous_sidecar.as_deref(),
                &intent.sidecar,
            )?;
            restore(
                &self.output,
                &self.journal.join("old-ciphertext"),
                intent.previous_ciphertext.as_deref(),
                &self.journal.join("restore-ciphertext"),
            )?;
            restore(
                &self.sidecar,
                &self.journal.join("old-sidecar"),
                intent.previous_sidecar.as_deref(),
                &self.journal.join("restore-sidecar"),
            )?;
        }
        // Retire the live intent atomically before removing its owned members.
        // Interruption during retirement can retain diagnostic files, never a live mixed pair.
        let retired = temp_output_path(&self.journal);
        ensure!(!retired.try_exists()?, "publication retirement path already exists");
        bijux_dna_infra::rename(&self.journal, &retired)?;
        for path in journal_files(&retired)? {
            bijux_dna_infra::remove_file(&path)?;
        }
        bijux_dna_infra::remove_empty_dir(&retired)?;
        Ok(())
    }
}

fn regular_file(path: &Path) -> Result<bool> {
    match path.symlink_metadata() {
        Ok(metadata) => {
            ensure!(
                metadata.is_file() && !metadata.file_type().is_symlink(),
                "bundle publication member must be an ordinary file: {}",
                path.display()
            );
            Ok(true)
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}
fn content_identity(path: &Path) -> Result<Option<String>> {
    if regular_file(path)? {
        Ok(Some(digest_file_sha256(path)?))
    } else {
        Ok(None)
    }
}
fn copy_synced(source: &Path, destination: &Path) -> Result<()> {
    regular_file(source)?;
    regular_file(destination)?;
    bijux_dna_infra::copy_file(source, destination)?;
    fs::File::open(destination)?.sync_all()?;
    Ok(())
}
fn backup(path: &Path, destination: &Path) -> Result<Option<String>> {
    if !regular_file(path)? {
        return Ok(None);
    }
    copy_synced(path, destination)?;
    Ok(Some(digest_file_sha256(destination)?))
}
fn admit_restoration(
    output: &Path,
    backup: &Path,
    previous: Option<&str>,
    next: &str,
) -> Result<()> {
    let current = content_identity(output)?;
    ensure!(
        current.is_none() || current.as_deref() == previous || current.as_deref() == Some(next),
        "bundle output changed outside its publication: {}",
        output.display()
    );
    if let Some(previous) = previous {
        ensure!(
            content_identity(backup)?.as_deref() == Some(previous),
            "previous bundle publication member is missing or corrupted"
        );
    }
    Ok(())
}
fn restore(output: &Path, backup: &Path, previous: Option<&str>, staging: &Path) -> Result<()> {
    if content_identity(output)?.as_deref() == previous {
        return Ok(());
    }
    if previous.is_some() {
        copy_synced(backup, staging)?;
        bijux_dna_infra::rename(staging, output)?;
    } else if regular_file(output)? {
        bijux_dna_infra::remove_file(output)?;
    }
    Ok(())
}
fn journal_files(root: &Path) -> Result<Vec<PathBuf>> {
    fs::read_dir(root)?
        .map(|entry| {
            let entry = entry?;
            ensure!(
                [
                    "intent.json",
                    "new-ciphertext",
                    "new-sidecar",
                    "old-ciphertext",
                    "old-sidecar",
                    "committed",
                    "restore-ciphertext",
                    "restore-sidecar"
                ]
                .iter()
                .any(|name| entry.file_name() == *name),
                "unknown file inside bundle publication journal"
            );
            ensure!(regular_file(&entry.path())?, "publication member disappeared");
            Ok(entry.path())
        })
        .collect()
}

fn locations(output: &Path) -> Result<(PathBuf, PathBuf)> {
    let parent =
        output.parent().filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    ensure!(parent.is_dir(), "bundle output parent does not exist");
    let name = output
        .file_name()
        .context("bundle output has no filename")?
        .to_str()
        .context("bundle output filename is not UTF-8")?;
    Ok((
        parent.join(format!(".{name}.publication.lock")),
        parent.join(format!(".{name}.publication")),
    ))
}

#[cfg(test)]
#[path = "../../../tests/contracts/bundle/publication.rs"]
mod tests;
