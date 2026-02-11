#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

#[test]
fn policy__contracts__generated_configs_policy__generated_configs_are_not_hand_edited() {
    let root = support::workspace_root();

    for rel in [
        "tool_registry.toml",
        "tool_registry_experimental.toml",
        "required_tools.toml",
        "stages.toml",
        "images.toml",
    ] {
        let checked_in = root.join("configs").join(rel);
        let checked_in_raw = std::fs::read_to_string(&checked_in)
            .unwrap_or_else(|_| panic!("read {}", checked_in.display()));

        let mut lines = checked_in_raw.lines();
        let first = lines.next().unwrap_or_default();
        let second = lines.next().unwrap_or_default();
        let third = lines.next().unwrap_or_default();
        assert_eq!(first, "# GENERATED - DO NOT EDIT - source: domain/**");
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
