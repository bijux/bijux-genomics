#![allow(non_snake_case)]
#[path = "../../../../support/fs.rs"]
mod support;

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
            && second["# source_commit: ".len()..]
                .chars()
                .all(|c| c.is_ascii_hexdigit()))
        {
            eprintln!("generated source_commit drift: {}", checked_in.display());
        }
        if third != "# domain_schema_version: bijux.domain.v1" {
            eprintln!(
                "generated domain schema header drift: {}",
                checked_in.display()
            );
        }
    }
}

#[test]
fn policy__contracts__generated_configs_policy__single_generator_script_is_canonical() {
    let root = support::workspace_root();
    let script = root.join("scripts/tooling/generate-configs.sh");
    let makefile = root.join("makefiles/cargo.mk");
    let script_raw =
        std::fs::read_to_string(&script).unwrap_or_else(|_| panic!("read {}", script.display()));
    let make_raw = std::fs::read_to_string(&makefile)
        .unwrap_or_else(|_| panic!("read {}", makefile.display()));

    assert!(
        script_raw.contains("compile_domain_configs"),
        "scripts/tooling/generate-configs.sh must call compile_domain_configs"
    );
    assert!(
        make_raw.contains("generate-configs:")
            && make_raw.contains("./scripts/run.sh tooling generate-configs"),
        "makefiles/cargo.mk generate-configs target must call scripts/run.sh tooling generate-configs"
    );
}
