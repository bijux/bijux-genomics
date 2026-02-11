#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__generated_configs_policy__generated_configs_are_not_hand_edited() {
    let root = support::workspace_root();

    for rel in [
        "tool_registry.toml",
        "tool_registry_experimental.toml",
        "tool_registry_vcf.toml",
        "required_tools.toml",
        "required_tools_vcf.toml",
        "stages.toml",
        "stages_vcf.toml",
        "images.toml",
    ] {
        let checked_in = root.join("configs").join(rel);
        let checked_in_raw = std::fs::read_to_string(&checked_in)
            .unwrap_or_else(|_| panic!("read {}", checked_in.display()));

        let mut lines = checked_in_raw.lines();
        let first = lines.next().unwrap_or_default();
        let second = lines.next().unwrap_or_default();
        let third = lines.next().unwrap_or_default();
        assert!(
            first.starts_with("# GENERATED - DO NOT EDIT - source: "),
            "generated config header marker missing: {}",
            checked_in.display()
        );
        assert!(
            second.starts_with("# source_commit: ")
                && second.len() == "# source_commit: ".len() + 40
                && second["# source_commit: ".len()..]
                    .chars()
                    .all(|c| c.is_ascii_hexdigit()),
            "generated config header must contain source commit hash: {}",
            checked_in.display()
        );
        assert_eq!(
            third,
            "# domain_schema_version: bijux.domain.v1",
            "generated config header must contain domain schema version: {}",
            checked_in.display()
        );
    }
}

#[test]
fn policy__contracts__generated_configs_policy__single_generator_script_is_canonical() {
    let root = support::workspace_root();
    let script = root.join("scripts/generate-configs.sh");
    let makefile = root.join("makefiles/cargo.mk");
    let script_raw =
        std::fs::read_to_string(&script).unwrap_or_else(|_| panic!("read {}", script.display()));
    let make_raw = std::fs::read_to_string(&makefile)
        .unwrap_or_else(|_| panic!("read {}", makefile.display()));

    assert!(
        script_raw.contains("compile_domain_configs"),
        "scripts/generate-configs.sh must call compile_domain_configs"
    );
    assert!(
        make_raw.contains("generate-configs:")
            && make_raw.contains("./scripts/generate-configs.sh"),
        "makefiles/cargo.mk generate-configs target must call scripts/generate-configs.sh"
    );
}
