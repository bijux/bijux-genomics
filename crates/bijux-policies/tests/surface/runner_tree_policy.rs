#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn policy__surface__runner_tree_policy__runner_src_layout_contract() {
    let root = workspace_root();
    let src_dir = root.join("crates/bijux-runner/src");
    let allowed = ["lib.rs", "execute.rs", "runner_core.rs", "backend"]; 
    let mut offenders = Vec::new();
    let entries = std::fs::read_dir(&src_dir).expect("read bijux-runner/src");
    for entry in entries {
        let entry = entry.expect("read entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if allowed.iter().any(|allowed| *allowed == name) {
            continue;
        }
        offenders.push(name.to_string());
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-runner/src may only contain lib.rs, execute.rs, runner_core.rs, and backend/.\n\
Unexpected entries: {:?}\n\
Fix by moving new code under backend/* or updating the policy with justification.\n\
See docs/40-policies/STYLE.md for layout rules.",
        offenders
    );
}
