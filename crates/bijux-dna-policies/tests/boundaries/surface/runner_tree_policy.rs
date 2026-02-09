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
fn policy__boundaries__runner_tree_policy__runner_src_layout_contract() {
    let root = repo_root();
    let src_dir = root.join("crates/bijux-dna-runner/src");
    let allowed = ["lib.rs", "execute.rs", "runner_core.rs", "backend"];
    let mut offenders = Vec::new();
    let entries = std::fs::read_dir(&src_dir).expect("read bijux-dna-runner/src");
    for entry in entries {
        let entry = entry.expect("read entry");
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if allowed.iter().any(|allowed| *allowed == name) {
            continue;
        }
        offenders.push(name.to_string());
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-runner/src may only contain lib.rs, execute.rs, runner_core.rs, and backend/.\n\
Unexpected entries: {:?}\n\
Fix by moving new code under backend/* or updating the policy with justification.\n\
See docs/40-policies/STYLE.md for layout rules.",
        offenders
    );
}

#[test]
fn policy__boundaries__runner_tree_policy__runner_scope_is_docker_only() {
    let root = repo_root();
    let lib_rs = root.join("crates/bijux-dna-runner/src/lib.rs");
    let content = std::fs::read_to_string(&lib_rs).expect("read runner lib.rs");
    bijux_dna_policies::policy_assert!(
        !content.contains("LocalRunner"),
        "bijux-dna-runner scope is docker-only in this workspace; LocalRunner must not be exposed."
    );
}

#[test]
fn policy__boundaries__runner_tree_policy__runner_primitives_are_not_reexported() {
    let root = repo_root();
    let lib_rs = root.join("crates/bijux-dna-runner/src/lib.rs");
    let content = std::fs::read_to_string(&lib_rs).expect("read runner lib.rs");
    bijux_dna_policies::policy_assert!(
        !content.contains("pub mod primitives"),
        "runner internals must not expose a public primitives module"
    );
    let reexport_line = content
        .lines()
        .find(|line| line.trim_start().starts_with("pub use") && line.contains("primitives"));
    bijux_dna_policies::policy_assert!(
        reexport_line.is_none(),
        "runner internals must not re-export primitives"
    );
}
