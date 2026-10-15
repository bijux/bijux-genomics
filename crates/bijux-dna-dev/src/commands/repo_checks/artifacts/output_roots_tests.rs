#![cfg(test)]

use std::io::Write;

use anyhow::{Context, Result};

use crate::catalog::checks::check_registry;
use crate::runtime::workspace::Workspace;

#[test]
fn output_root_permission_probe_preserves_existing_workspace_evidence() -> Result<()> {
    let fixture = bijux_dna_infra::temp_dir("bijux-output-root-probe-contract")?;
    let workspace = Workspace { root: fixture.path().to_path_buf() };
    let check = check_registry()
        .into_iter()
        .find(|check| check.id == "check-output-roots")
        .context("registered output-root check is missing")?;
    let control = super::check_output_roots(&workspace, &check)?;
    let sentinel = workspace.path(".sentinel-readonly");
    assert!(!sentinel.exists(), "a completed probe must remove its exclusively owned directory");
    writeln!(
        std::io::stdout(),
        "OUTPUT_ROOT_PROBE_CONTROL_PASS: completed probe cleaned its own output"
    )?;

    std::fs::create_dir(&sentinel)?;
    let retained_path = sentinel.join("caller-evidence.txt");
    std::fs::write(&retained_path, b"caller-owned evidence must survive")?;
    let directory_permissions = std::fs::metadata(&sentinel)?.permissions();
    let file_permissions = std::fs::metadata(&retained_path)?.permissions();
    let artifact_evidence = workspace.path("artifacts/checks/output-roots/caller-evidence.txt");
    std::fs::create_dir_all(
        artifact_evidence.parent().context("artifact evidence parent is missing")?,
    )?;
    std::fs::write(&artifact_evidence, b"retained artifact sibling")?;

    let observed = super::check_output_roots(&workspace, &check)?;
    let retained = std::fs::read(&retained_path);
    assert!(
        retained.is_ok(),
        "output-root permission probe must preserve preexisting workspace evidence: {retained:?}"
    );
    assert_eq!(retained?, b"caller-owned evidence must survive");
    assert_eq!(std::fs::metadata(&sentinel)?.permissions(), directory_permissions);
    assert_eq!(std::fs::metadata(&retained_path)?.permissions(), file_permissions);
    assert_eq!(std::fs::read(&artifact_evidence)?, b"retained artifact sibling");
    assert_eq!(
        control.status, observed.status,
        "unrelated evidence must not alter probe semantics"
    );
    for entry in std::fs::read_dir(&workspace.root)? {
        let name = entry?.file_name();
        assert!(
            name == ".sentinel-readonly" || name == "artifacts",
            "probe created an undeclared root output: {name:?}"
        );
    }
    Ok(())
}
