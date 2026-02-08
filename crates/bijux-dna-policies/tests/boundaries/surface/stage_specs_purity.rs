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
fn policy__boundaries__stage_specs_purity__stage_specs_are_declarative_only() {
    let root = repo_root();
    let specs = [
        root.join("crates/bijux-dna-stages-fastq/src/stage_specs.rs"),
        root.join("crates/bijux-dna-stages-bam/src/stage_specs.rs"),
    ];
    let forbidden = [
        "CommandSpec",
        "ContainerImageRef",
        "command_template",
        "argv",
        "docker",
        "container",
    ];
    let mut offenders = Vec::new();
    for spec in specs {
        let content = std::fs::read_to_string(&spec).expect("read stage_specs");
        for token in &forbidden {
            if content.contains(token) {
                offenders.push(format!(
                    "{} contains forbidden token `{}`",
                    spec.display(),
                    token
                ));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stage_specs must be declarative only (no command/tool wiring).\n\
Fix by moving execution details into planners or runner.\n\
See docs/40-policies/STYLE.md for contract purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
