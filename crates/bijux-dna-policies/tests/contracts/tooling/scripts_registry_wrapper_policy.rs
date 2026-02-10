#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

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
    let script = root.join("scripts").join("registry-tools.sh");
    let content = std::fs::read_to_string(&script).expect("read scripts/registry-tools.sh");

    bijux_dna_policies::policy_assert!(
        content.contains("cargo run --bin bijux-dna -- registry"),
        "scripts/registry-tools.sh must delegate to CLI registry commands"
    );
    bijux_dna_policies::policy_assert!(
        !content.contains("awk "),
        "scripts/registry-tools.sh must not parse generated configs directly"
    );
    bijux_dna_policies::policy_assert!(
        !content.contains("tool_registry.toml"),
        "scripts/registry-tools.sh must not read configs/tool_registry.toml directly"
    );
}
