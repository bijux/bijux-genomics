use anyhow::{anyhow, Context, Result};
use std::{
    collections::BTreeMap,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

pub(super) fn read_fixture(path: &Path) -> Result<Vec<u8>> {
    let meta = fs::symlink_metadata(path)
        .with_context(|| format!("inspect fixture {}", path.display()))?;
    anyhow::ensure!(
        meta.file_type().is_file(),
        "fixture must be a regular file: {}",
        path.display()
    );
    fs::read(path).with_context(|| format!("read fixture {}", path.display()))
}
pub(super) fn insert_copy(
    copies: &mut BTreeMap<PathBuf, Vec<u8>>,
    source: &Path,
    destination: PathBuf,
) -> Result<()> {
    let payload = read_fixture(source)?;
    if let Some(prior) = copies.get(&destination) {
        anyhow::ensure!(
            prior == &payload,
            "QC fixture sources conflict at {}",
            destination.display()
        );
    }
    copies.insert(destination, payload);
    Ok(())
}
pub(super) fn collect_directory(
    source: &Path,
    destination: &Path,
    copies: &mut BTreeMap<PathBuf, Vec<u8>>,
) -> Result<()> {
    let meta = fs::symlink_metadata(source)?;
    anyhow::ensure!(
        meta.is_dir() && !meta.file_type().is_symlink(),
        "fixture directory must be a real directory: {}",
        source.display()
    );
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let target = destination.join(entry.file_name());
        if entry.file_type()?.is_dir() {
            collect_directory(&entry.path(), &target, copies)?;
        } else {
            insert_copy(copies, &entry.path(), target)?;
        }
    }
    Ok(())
}
pub(super) fn fixture_source(root: &Path, relative: &Path) -> Result<PathBuf> {
    anyhow::ensure!(
        !relative.is_absolute()
            && relative.components().all(|c| matches!(c, std::path::Component::Normal(_))),
        "QC contributor must be relative to its fixture root: {}",
        relative.display()
    );
    let path = root.join(relative);
    let canonical = fs::canonicalize(&path)?;
    anyhow::ensure!(
        canonical.starts_with(fs::canonicalize(root)?),
        "QC fixture escapes its declared root: {}",
        path.display()
    );
    Ok(path)
}
pub(super) fn admit_output(path: &Path) -> Result<()> {
    for (index, ancestor) in path.ancestors().filter(|p| !p.as_os_str().is_empty()).enumerate() {
        match fs::symlink_metadata(ancestor) {
            Ok(meta)
                if meta.file_type().is_symlink()
                    || (index == 0 && !meta.is_file())
                    || (index > 0 && !meta.is_dir()) =>
            {
                return Err(anyhow!(
                    "QC output has conflicting kind or symlink: {}",
                    ancestor.display()
                ))
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("inspect {}", ancestor.display()))
            }
        }
    }
    Ok(())
}
