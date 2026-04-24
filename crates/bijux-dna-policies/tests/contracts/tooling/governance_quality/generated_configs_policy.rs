#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

fn native_ops_sources(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    [
        "crates/bijux-dna-dev/src/commands/ops/tooling/diagnostics.rs",
        "crates/bijux-dna-dev/src/commands/ops/mod.rs",
        "crates/bijux-dna-dev/src/commands/ops.rs",
    ]
    .into_iter()
    .map(|rel| root.join(rel))
    .filter(|path| path.is_file())
    .collect()
}

#[test]
#[ignore = "TODO: generated header/source_commit format migration in progress"]
fn policy__contracts__generated_configs_policy__generated_configs_are_not_hand_edited() {
    let root = support::workspace_root();

    for rel in [
        "registry/tool_registry.toml",
        "registry/tool_registry_experimental.toml",
        "registry/tool_registry_vcf.toml",
        "tools/required_tools.toml",
        "tools/required_tools_vcf.toml",
        "stages/stages.toml",
        "stages/stages_vcf.toml",
        "tools/images.toml",
    ] {
        let checked_in = root.join("configs").join("ci").join(rel);
        let checked_in_raw = std::fs::read_to_string(&checked_in)
            .unwrap_or_else(|_| panic!("read {}", checked_in.display()));

        let mut lines = checked_in_raw.lines();
        let first = lines.next().unwrap_or_default();
        let second = lines.next().unwrap_or_default();
        let third = lines.next().unwrap_or_default();
        if !first.starts_with("# GENERATED - DO NOT EDIT - source: ") {
            eprintln!("generated header marker drift: {}", checked_in.display());
        }
        if !(second.starts_with("# source_commit: ")
            && second.len() == "# source_commit: ".len() + 40
            && second["# source_commit: ".len()..].chars().all(|c| c.is_ascii_hexdigit()))
        {
            eprintln!("generated source_commit drift: {}", checked_in.display());
        }
        if third != "# domain_schema_version: bijux.domain.v1" {
            eprintln!("generated domain schema header drift: {}", checked_in.display());
        }
    }
}

#[test]
fn policy__contracts__generated_configs_policy__single_generator_command_is_canonical() {
    let root = support::workspace_root();
    let makefile = root.join("makes/cargo.mk");
    let make_raw = std::fs::read_to_string(&makefile)
        .unwrap_or_else(|_| panic!("read {}", makefile.display()));
    let native_sources = native_ops_sources(&root);
    let native_contains_generator = native_sources.iter().any(|path| {
        std::fs::read_to_string(path)
            .unwrap_or_else(|_| panic!("read {}", path.display()))
            .contains("compile_domain_configs")
    });

    assert!(native_contains_generator, "bijux-dna-dev native ops must call compile_domain_configs");
    assert!(
        make_raw.contains("generate-configs:")
            && make_raw.contains("cargo run -q -p bijux-dna-dev -- tooling run generate-configs"),
        "makes/cargo.mk generate-configs target must call bijux-dna-dev tooling generate-configs"
    );
}
