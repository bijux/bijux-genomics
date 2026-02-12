#![allow(non_snake_case)]

use std::path::Path;

#[test]
fn policy__contracts__cli_app_prefix_policy__root_cli_requires_app_prefix() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root");
    let parse_file = root.join("crates/bijux-dna-cli/src/commands/cli/parse/common.rs");
    let src = std::fs::read_to_string(&parse_file).expect("read parse common");

    let root_enum = src
        .split("pub enum RootCommand")
        .nth(1)
        .and_then(|s| s.split("}\n\n#[derive").next())
        .unwrap_or("");

    let has_non_dna_root_variant = root_enum
        .lines()
        .map(str::trim)
        .any(|line| line.ends_with('{') && !line.starts_with("Dna "));

    bijux_dna_policies::policy_assert!(
        !has_non_dna_root_variant,
        "root CLI must only expose app-prefixed commands (`bijux <app> ...`); found non-dna root variant in {}",
        parse_file.display()
    );
}
