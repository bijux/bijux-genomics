use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn command_inventory_matches_binary_tree() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
        .unwrap_or_else(|err| panic!("read Cargo.toml: {err}"));

    let binary_files = dir_entries(&root.join("src/bin"));
    let expected_files: BTreeSet<_> = [
        "compile_domain_configs.rs",
        "domain_registry_bundle.rs",
        "domain_registry_query.rs",
        "domain_validate.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(binary_files, expected_files, "binary tree changed without command review");

    for command in [
        "compile_domain_configs",
        "domain_registry_bundle",
        "domain_registry_query",
        "domain_validate",
    ] {
        assert!(
            commands_doc.contains(&format!("`{command}`")),
            "docs/COMMANDS.md must list command `{command}`"
        );
        assert!(
            commands_doc.contains(&format!("src/bin/{command}.rs")),
            "docs/COMMANDS.md must map `{command}` to its binary"
        );
    }

    for option in [
        "--domain-dir <path>",
        "--configs-dir <path>",
        "--scope <scope>",
        "--bundle <path>",
        "--write-generated",
        "--kind <domains|stages|tools|metrics|artifacts|defaults|deprecations|evidence|fixtures>",
        "--domain <id>",
        "--stage-id <id>",
        "--tool-id <id>",
    ] {
        assert!(
            commands_doc.contains(option),
            "docs/COMMANDS.md must list owned option `{option}`"
        );
    }
    assert!(
        commands_doc.contains("pre_hpc_pre_vcf"),
        "docs/COMMANDS.md must document the default compiler scope"
    );
    for forbidden in [
        "No bioinformatics tool execution.",
        "No container, scheduler, or runtime orchestration.",
        "No network clients.",
        "No writes outside declared generated config outputs.",
    ] {
        assert!(commands_doc.contains(forbidden), "docs/COMMANDS.md must document `{forbidden}`");
    }
    assert!(
        !manifest.contains("[[bin]]"),
        "command binaries should remain discovered from src/bin unless Cargo metadata needs a reviewed exception"
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect()
}
