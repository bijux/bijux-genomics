#![allow(non_snake_case)]
#[allow(dead_code)]
#[path = "../../../support/workspace_binary.rs"]
mod workspace_binary;

use std::ffi::OsStr;
use std::path::Path;

#[test]
fn policy__contracts__workspace_binary_policy__configured_workspace_executable_uses_exact_native_path_without_cargo(
) -> std::io::Result<()> {
    let binary = std::env::current_exe()?;
    let root = Path::new("workspace with spaces");
    let command = workspace_binary::command_with_path(
        root,
        "bijux-dna",
        "bijux-dna",
        Some(binary.as_os_str()),
    )?;
    assert_eq!(command.get_program(), binary.as_os_str());
    assert_eq!(command.get_current_dir(), Some(root));
    assert_eq!(command.get_args().count(), 0);
    Ok(())
}

#[test]
fn policy__contracts__workspace_binary_policy__malformed_explicit_executable_does_not_fall_back_to_a_build(
) -> std::io::Result<()> {
    let root = std::env::current_dir()?;
    let missing = root.join("absent-workspace-executable");
    for path in [OsStr::new(""), OsStr::new("relative/bin"), missing.as_os_str(), root.as_os_str()]
    {
        assert!(workspace_binary::command_with_path(&root, "bijux-dna", "bijux-dna", Some(path))
            .is_err());
    }
    Ok(())
}

#[test]
fn policy__contracts__workspace_binary_policy__local_fallback_uses_cargo_as_the_source_freshness_authority(
) -> std::io::Result<()> {
    let command = workspace_binary::command_with_path(
        Path::new("workspace"),
        "bijux-dna-dev",
        "bijux-dna-dev",
        None,
    )?;
    assert_eq!(command.get_program(), "cargo");
    assert_eq!(
        command.get_args().collect::<Vec<_>>(),
        ["run", "--locked", "-q", "-p", "bijux-dna-dev", "--bin", "bijux-dna-dev", "--"]
    );
    Ok(())
}
