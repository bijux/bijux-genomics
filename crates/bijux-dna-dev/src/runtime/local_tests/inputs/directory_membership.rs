//! Directory metadata is an explicit input; admitting it never admits child file bytes.
use std::collections::BTreeSet;
use std::path::{Component, Path};

use anyhow::{ensure, Context, Result};

use crate::application::ci::local_tests::{
    DirectoryEntryIdentity, DirectoryIdentity, FileIdentity, ToolIdentity,
};

fn mode(metadata: &std::fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode()
    }
    #[cfg(not(unix))]
    {
        u32::from(metadata.permissions().readonly())
    }
}

pub(super) fn source_name(name: &str) -> Result<()> {
    ensure!(
        !name.is_empty()
            && !Path::new(name).is_absolute()
            && name.split('/').all(|part| !part.is_empty() && part != "." && part != "..")
            && Path::new(name).components().all(|part| matches!(part, Component::Normal(_))),
        "lane source path is not normalized relative input"
    );
    Ok(())
}

pub(super) fn paths(files: &[FileIdentity], tools: &[ToolIdentity]) -> Result<BTreeSet<String>> {
    let mut paths = BTreeSet::from([".".to_owned()]);
    for file in files {
        source_name(&file.path)?;
        let mut parent = Path::new(&file.path).parent();
        while let Some(path) = parent {
            if path.as_os_str().is_empty() {
                break;
            }
            paths.insert(path.to_str().context("source directory must be UTF-8")?.to_owned());
            parent = path.parent();
        }
    }
    for component in tools.iter().flat_map(|tool| &tool.components) {
        let path = Path::new(&component.path);
        ensure!(
            path.is_absolute()
                && !path
                    .components()
                    .any(|part| matches!(part, Component::CurDir | Component::ParentDir)),
            "provider component path is not normalized absolute input"
        );
        let parent = path.parent().context("provider component directory absent")?;
        paths.insert(parent.to_str().context("provider directory must be UTF-8")?.to_owned());
    }
    Ok(paths)
}

pub(super) fn identity(root: &Path, name: String) -> Result<DirectoryIdentity> {
    let path = if name == "." {
        root.to_owned()
    } else if Path::new(&name).is_absolute() {
        let path = Path::new(&name);
        ensure!(
            !path.components().any(|part| matches!(part, Component::CurDir | Component::ParentDir)),
            "provider directory path is not normalized"
        );
        path.to_owned()
    } else {
        source_name(&name)?;
        root.join(&name)
    };
    let metadata = path.symlink_metadata()?;
    ensure!(
        metadata.is_dir() && !metadata.file_type().is_symlink() && path.canonicalize()? == path,
        "lane directory input is missing or redirected: {}",
        path.display()
    );
    let mut entries = Vec::new();
    for child in std::fs::read_dir(&path)? {
        let child = child?;
        let metadata = child.path().symlink_metadata()?;
        let kind = if metadata.is_file() {
            "file"
        } else if metadata.is_dir() {
            "directory"
        } else if metadata.file_type().is_symlink() {
            "symlink"
        } else {
            "other"
        };
        entries.push(DirectoryEntryIdentity {
            name: child.file_name().to_str().context("directory entry must be UTF-8")?.to_owned(),
            kind: kind.into(),
            mode: mode(&metadata),
        });
    }
    entries.sort_by(|left, right| left.name.cmp(&right.name));
    let digest = bijux_dna_infra::hash_bytes_sha256(&serde_json::to_vec(&entries)?);
    Ok(DirectoryIdentity { path: name, sha256: digest, mode: mode(&metadata), entries })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_detects_late_entries_without_promoting_child_bytes() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        std::fs::write(root.join("retained.txt"), b"original bytes")?;
        let original = identity(&root, ".".into())?;
        std::fs::write(root.join("retained.txt"), b"other bytes of different length")?;
        assert_eq!(identity(&root, ".".into())?, original);
        std::fs::write(root.join("late.txt"), b"late")?;
        assert_ne!(identity(&root, ".".into())?, original);
        Ok(())
    }

    #[test]
    fn root_entry_keeps_artifact_authority_without_hashing_generated_children() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        std::fs::create_dir(root.join("artifacts"))?;
        let original = identity(&root, ".".into())?;
        std::fs::write(root.join("artifacts/result.json"), b"owned output")?;
        assert_eq!(identity(&root, ".".into())?, original);
        assert!(original
            .entries
            .iter()
            .any(|entry| entry.name == "artifacts" && entry.kind == "directory"));
        Ok(())
    }

    #[test]
    fn membership_detects_entry_kind_and_directory_mode_changes() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        std::fs::write(root.join("owned"), b"file")?;
        let original = identity(&root, ".".into())?;
        std::fs::remove_file(root.join("owned"))?;
        std::fs::create_dir(root.join("owned"))?;
        assert_ne!(identity(&root, ".".into())?, original);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let directory = identity(&root, "owned".into())?;
            std::fs::set_permissions(
                root.join("owned"),
                std::fs::Permissions::from_mode(directory.mode ^ 0o100),
            )?;
            assert_ne!(identity(&root, "owned".into())?, directory);
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn identical_membership_at_redirected_leaf_or_parent_is_refused() -> Result<()> {
        use std::os::unix::fs::symlink;
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        std::fs::create_dir_all(root.join("original/nested"))?;
        std::fs::create_dir_all(root.join("alternate/nested"))?;
        std::fs::write(root.join("original/nested/data"), b"same")?;
        std::fs::write(root.join("alternate/nested/data"), b"same")?;
        identity(&root, "original/nested".into())?;
        std::fs::remove_file(root.join("original/nested/data"))?;
        std::fs::remove_dir(root.join("original/nested"))?;
        symlink(root.join("alternate/nested"), root.join("original/nested"))?;
        assert!(identity(&root, "original/nested".into()).is_err());
        std::fs::remove_file(root.join("original/nested"))?;
        std::fs::remove_dir(root.join("original"))?;
        symlink(root.join("alternate"), root.join("original"))?;
        assert!(identity(&root, "original/nested".into()).is_err());
        Ok(())
    }

    #[test]
    fn source_names_refuse_traversal_and_directory_namespace_is_explicit() -> Result<()> {
        for invalid in ["", ".", "../foreign", "source/../foreign", "source//nested", "/foreign"] {
            assert!(source_name(invalid).is_err());
        }
        source_name("source/nested")?;
        let owner = tempfile::tempdir()?;
        let root = owner.path().canonicalize()?;
        identity(&root, ".".into())?;
        Ok(())
    }
}
