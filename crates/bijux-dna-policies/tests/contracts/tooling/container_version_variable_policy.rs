#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__container_version_variable_policy__container_definitions_define_explicit_version_variable(
) {
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("containers/docker"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("Dockerfile.") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if !raw.contains("ARG TOOL_VERSION") {
            offenders.push(format!(
                "{} missing explicit version variable `ARG TOOL_VERSION`",
                path.display()
            ));
        }
    }

    for entry in WalkDir::new(root.join("containers/apptainer"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("def") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if !raw.contains("VERSION ") {
            offenders.push(format!(
                "{} missing explicit version variable label `VERSION`",
                path.display()
            ));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "container version variable policy failures:\n{}",
        offenders.join("\n")
    );
}
