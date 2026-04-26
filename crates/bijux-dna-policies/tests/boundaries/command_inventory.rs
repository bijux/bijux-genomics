#![allow(non_snake_case)]

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn policy__boundaries__command_inventory__documents_policy_commands_without_runtime_entrypoints() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs").join("COMMANDS.md");
    let content = fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    assert!(
        content.contains("## Runtime Commands\nNone."),
        "COMMANDS.md must make the runtime command ownership boundary explicit"
    );
    assert!(
        content.contains("## Managed Command Inventory"),
        "COMMANDS.md must provide a managed command inventory section"
    );
    assert!(
        !root.join("src").join("bin").exists(),
        "bijux-dna-policies must not define src/bin runtime command entrypoints"
    );

    assert_eq!(
        documented_commands(&content),
        entries([
            "make guardrails",
            "make policies",
            "make structure-check",
            "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --no-default-features",
            "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries --no-default-features",
            "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts --no-default-features",
            "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test determinism --no-default-features",
            "CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test guardrails --no-default-features",
        ]),
        "COMMANDS.md must stay the SSOT for policy commands this crate manages"
    );

    assert!(
        !content.contains("cargo test -p bijux-dna-policies --test boundaries`"),
        "focused policy test commands must include --no-default-features"
    );
}

fn documented_commands(content: &str) -> BTreeSet<String> {
    content
        .split('`')
        .filter(|segment| segment.starts_with("make ") || segment.starts_with("CARGO_TARGET_DIR="))
        .map(str::to_string)
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
