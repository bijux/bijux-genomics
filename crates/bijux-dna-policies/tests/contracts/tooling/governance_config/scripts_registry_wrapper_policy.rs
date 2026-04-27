#![allow(non_snake_case)]
use std::path::PathBuf;
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__scripts_registry_wrapper_policy__container_shell_wrappers_are_removed() {
    let root = repo_root();
    let wrappers = WalkDir::new(root.join("bijux-dna-dev").join("containers"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("sh"))
        .map(|entry| entry.path().display().to_string())
        .collect::<Vec<_>>();

    bijux_dna_policies::policy_assert!(
        wrappers.is_empty(),
        "bijux-dna-dev/containers must not contain shell wrappers after migration:\n{}",
        wrappers.join("\n")
    );
}

#[test]
fn policy__contracts__scripts_registry_wrapper_policy__scripts_do_not_parse_tool_registry_directly()
{
    let root = repo_root();
    let scripts_dir = root.join("scripts");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&scripts_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("sh") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read script");
        let parses_registry = content.contains("tool_registry.toml") && content.contains("awk ");
        if parses_registry {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "scripts must use CLI registry commands, not parse configs/ci/registry/tool_registry.toml directly:\n{}",
        offenders.join("\n")
    );
}
