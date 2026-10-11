use super::cases::{VcfParserFailureCase, VcfParserFailureReason};
use anyhow::{ensure, Context, Result};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) struct FixtureSnapshot {
    files: Vec<(PathBuf, Vec<u8>)>,
}

pub(super) fn admit_fixture_set(
    repo: &Path,
    cases: &[VcfParserFailureCase],
) -> Result<Vec<FixtureSnapshot>> {
    cases
        .iter()
        .map(|case| {
            let root = repo.join(case.fixture_dir);
            admit_ancestry(repo, &root)?;
            let snapshot = snapshot_directory(&root)?;
            ensure!(
                snapshot.files.iter().any(|(path, _)| path == Path::new(case.probe_artifact)),
                "VCF failure fixture lacks governed probe {}",
                case.probe_artifact
            );
            let published = repo
                .join("artifacts/bench-readiness/vcf-parser-failure-tests")
                .join(case.case_id)
                .join("fixture");
            for (path, _) in &snapshot.files {
                admit_target(repo, &published.join(path))?;
            }
            Ok(snapshot)
        })
        .collect()
}

fn snapshot_directory(root: &Path) -> Result<FixtureSnapshot> {
    let mut files = Vec::new();
    let mut directories = vec![root.to_owned()];
    while let Some(directory) = directories.pop() {
        let kind = fs::symlink_metadata(&directory)
            .with_context(|| format!("inspect {}", directory.display()))?
            .file_type();
        ensure!(kind.is_dir(), "VCF failure fixtures require directories without symbolic links");
        for entry in
            fs::read_dir(&directory).with_context(|| format!("read {}", directory.display()))?
        {
            let entry = entry?;
            let kind = entry.file_type()?;
            if kind.is_dir() {
                directories.push(entry.path());
            } else {
                ensure!(
                    kind.is_file(),
                    "VCF failure fixtures refuse symbolic links and special files"
                );
                let path = entry.path();
                files.push((
                    path.strip_prefix(root)?.to_owned(),
                    fs::read(&path).with_context(|| format!("read {}", path.display()))?,
                ));
            }
        }
    }
    files.sort_by(|left, right| left.0.cmp(&right.0));
    Ok(FixtureSnapshot { files })
}

fn admit_ancestry(repo: &Path, directory: &Path) -> Result<()> {
    ensure!(directory.starts_with(repo), "VCF failure fixture path escapes repository");
    for ancestor in directory.ancestors() {
        match fs::symlink_metadata(ancestor) {
            Ok(metadata) => ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "VCF failure fixture ancestry must contain directories without symbolic links: {}",
                ancestor.display()
            ),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        if ancestor == repo {
            break;
        }
    }
    Ok(())
}

fn admit_target(repo: &Path, path: &Path) -> Result<()> {
    admit_ancestry(repo, path.parent().context("VCF failure probe parent")?)?;
    match fs::symlink_metadata(path) {
        Ok(metadata) => ensure!(
            metadata.is_file() && !metadata.file_type().is_symlink(),
            "VCF failure probe refuses nonregular destination {}",
            path.display()
        ),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(())
}

pub(super) fn materialize_fixture(snapshot: &FixtureSnapshot, root: &Path) -> Result<()> {
    for (relative, bytes) in &snapshot.files {
        bijux_dna_infra::atomic_write_bytes(&root.join(relative), bytes)?;
    }
    Ok(())
}

pub(super) fn publish_probe(
    repo: &Path,
    working: &Path,
    published: &Path,
    case: &VcfParserFailureCase,
) -> Result<()> {
    let snapshot = snapshot_directory(working)?;
    for (relative, _) in &snapshot.files {
        admit_target(repo, &published.join(relative))?;
    }
    let absent = published.join(case.probe_artifact);
    if case.failure_reason == VcfParserFailureReason::MissingIndex {
        admit_target(repo, &absent)?;
    }
    materialize_fixture(&snapshot, published)?;
    if case.failure_reason == VcfParserFailureReason::MissingIndex {
        bijux_dna_infra::remove_file_if_exists(&absent)
            .with_context(|| format!("remove governed absent index {}", absent.display()))?;
    }
    Ok(())
}
