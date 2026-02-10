#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use bijux_dna_domain_compiler::{compile_domain_configs, CompileOptions};

#[test]
fn policy__contracts__generated_configs_policy__generated_configs_are_not_hand_edited() {
    let root = support::workspace_root();
    let temp = tempfile::tempdir().expect("create temp dir");
    let generated = temp.path().join("configs");
    compile_domain_configs(&CompileOptions {
        domain_dir: root.join("domain"),
        configs_dir: generated.clone(),
        scope: "pre_hpc_pre_vcf".to_string(),
    })
    .expect("compile generated configs");

    for rel in ["tool_registry.toml", "stages.toml", "images.toml"] {
        let checked_in = root.join("configs").join(rel);
        let expected = generated.join(rel);
        let checked_in_raw = std::fs::read_to_string(&checked_in)
            .unwrap_or_else(|_| panic!("read {}", checked_in.display()));
        let expected_raw = std::fs::read_to_string(&expected)
            .unwrap_or_else(|_| panic!("read {}", expected.display()));

        let normalize_source_commit = |text: &str| {
            text.lines()
                .map(|line| {
                    if line.starts_with("# source_commit: ") {
                        "# source_commit: <normalized>".to_string()
                    } else {
                        line.to_string()
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert!(
            normalize_source_commit(&checked_in_raw) == normalize_source_commit(&expected_raw),
            "generated config drift or hand-edit detected: {}",
            checked_in.display()
        );

        let mut lines = checked_in_raw.lines();
        let first = lines.next().unwrap_or_default();
        let second = lines.next().unwrap_or_default();
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
    }
}
