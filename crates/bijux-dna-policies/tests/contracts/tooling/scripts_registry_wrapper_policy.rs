#![allow(non_snake_case)]
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("resolve repo root")
        .to_path_buf()
}

#[test]
fn policy__contracts__scripts_registry_wrapper_policy__registry_script_is_cli_wrapper_only() {
    let root = repo_root();
    let script = root
        .join("scripts")
        .join("containers")
        .join("registry-tools.sh");
    let content = std::fs::read_to_string(&script).expect("read scripts/containers/registry-tools.sh");

    bijux_dna_policies::policy_assert!(
        content.contains("cargo run --bin bijux-dna -- registry"),
        "scripts/containers/registry-tools.sh must delegate to CLI registry commands"
    );
    bijux_dna_policies::policy_assert!(
        !content.contains("awk "),
        "scripts/containers/registry-tools.sh must not parse generated configs directly"
    );
    bijux_dna_policies::policy_assert!(
        !content.contains("tool_registry.toml"),
        "scripts/containers/registry-tools.sh must not read configs/tool_registry.toml directly"
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
        let parses_registry = content.contains("tool_registry.toml")
            && (content.contains("awk ") || content.contains("python3"));
        if parses_registry {
            offenders.push(entry.path().display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "scripts must use CLI registry commands, not parse configs/tool_registry.toml directly:\n{}",
        offenders.join("\n")
    );
}
