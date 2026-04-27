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

        let header_lines = checked_in_raw.lines().take(12).collect::<Vec<_>>();
        let generated_line = header_lines
            .iter()
            .find(|line| line.starts_with("# GENERATED - DO NOT EDIT - source: "))
            .copied();
        let source_commit_line =
            header_lines.iter().find(|line| line.starts_with("# source_commit: ")).copied();
        let domain_schema_line = header_lines
            .iter()
            .find(|line| **line == "# domain_schema_version: bijux.domain.v1")
            .copied();

        bijux_dna_policies::policy_assert!(
            generated_line.is_some(),
            "generated header marker drift: {}",
            checked_in.display()
        );
        bijux_dna_policies::policy_assert!(
            domain_schema_line.is_some(),
            "generated domain schema header drift: {}",
            checked_in.display()
        );

        let requires_source_commit = generated_line.is_some_and(|line| {
            line.contains("source: domain/**") || line.contains("source: domain/vcf/**")
        });
        if requires_source_commit {
            let valid_source_commit = source_commit_line.is_some_and(|line| {
                line.len() == "# source_commit: ".len() + 40
                    && line["# source_commit: ".len()..].chars().all(|c| c.is_ascii_hexdigit())
            });
            bijux_dna_policies::policy_assert!(
                valid_source_commit,
                "generated source_commit drift: {}",
                checked_in.display()
            );
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
