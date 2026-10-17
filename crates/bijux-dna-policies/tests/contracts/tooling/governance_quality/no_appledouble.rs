#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

const EXCLUDE_DIRS: &[&str] = &[".git", "target", "artifacts", "site", "node_modules"];

#[test]
fn slow__policy__contracts__no_appledouble__no_appledouble_artifacts() -> anyhow::Result<()> {
    let root = workspace_root();
    let offenders = inspect_appledouble(&root)?.offenders;

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "AppleDouble files are forbidden in the repo:\n{}",
        offenders.join("\n")
    );
    Ok(())
}
#[derive(Debug)]
struct AppleDoubleInspection {
    offenders: Vec<String>,
    entries_visited: usize,
    entered_root_outputs: usize,
}

fn is_root_output_path(root: &std::path::Path, path: &std::path::Path) -> bool {
    path.strip_prefix(root).ok().and_then(|relative| relative.components().next()).is_some_and(
        |component| component.as_os_str().to_str().is_some_and(|name| EXCLUDE_DIRS.contains(&name)),
    )
}

fn inspect_appledouble(root: &std::path::Path) -> anyhow::Result<AppleDoubleInspection> {
    anyhow::ensure!(
        root.is_dir(),
        "governed repository root is missing or not a directory: {}",
        root.display()
    );
    let root = root.to_path_buf();
    let mut entries_visited = 0usize;
    let mut entered_root_outputs = 0usize;
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root).into_iter().filter_entry(|entry| {
        !entry.file_type().is_dir() || !is_root_output_path(&root, entry.path())
    }) {
        let entry = entry?;
        entries_visited += 1;
        let path = entry.path();
        if is_root_output_path(&root, path) {
            entered_root_outputs += 1;
        }
        if !entry.file_type().is_file() {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if name.starts_with("._") {
            offenders.push(path.display().to_string());
        }
    }

    Ok(AppleDoubleInspection { offenders, entries_visited, entered_root_outputs })
}

#[test]
fn policy__contracts__no_appledouble__governed_source_survives_artifact_ancestors_and_output_pruning(
) -> anyhow::Result<()> {
    use anyhow::{ensure, Context};
    use std::collections::BTreeSet;

    let sandbox = bijux_dna_testkit::temp::tempdir_for("appledouble-source-authority");
    let root = sandbox.path().join("artifacts/verification/source");
    let governed = [
        "crates/member/src/._module",
        "configs/domain/._schema",
        "docs/._guide",
        "assets/._fixture",
        "crates/member/src/artifacts/._typed_artifact",
    ];
    for relative in governed {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().context("governed file parent")?)?;
        std::fs::write(path, b"forbidden resource-fork metadata")?;
    }
    for excluded in EXCLUDE_DIRS {
        let path = root.join(excluded).join("nested/._generated_metadata");
        std::fs::create_dir_all(path.parent().context("generated output parent")?)?;
        std::fs::write(path, b"excluded generated output")?;
    }
    let inspected = inspect_appledouble(&root)?;
    let actual = inspected.offenders.into_iter().collect::<BTreeSet<_>>();
    let expected = governed
        .map(|relative| root.join(relative).display().to_string())
        .into_iter()
        .collect::<BTreeSet<_>>();
    ensure!(actual == expected, "governed source violations were hidden: {actual:?}");
    ensure!(inspected.entries_visited > governed.len(), "source directories were not traversed");
    ensure!(inspected.entered_root_outputs == 0, "generated output trees were traversed");
    ensure!(
        inspect_appledouble(&root.join("missing")).is_err(),
        "missing source root was admitted"
    );
    let file_root = root.join("source-file");
    std::fs::write(&file_root, b"not a source directory")?;
    ensure!(inspect_appledouble(&file_root).is_err(), "file source root was admitted");
    Ok(())
}
