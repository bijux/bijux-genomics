#![allow(non_snake_case)]

use bijux_dna_policies::GuardrailConfig;
use bijux_dna_testkit::TestPaths;
use std::fs;
use std::path::Path;

pub fn guardrails() {
    let _config = GuardrailConfig::for_crate("bijux-dna-policies");
}

fn write_source(crate_root: &Path, rel: &str, content: &str) {
    let path = crate_root.join("src").join(rel);
    fs::create_dir_all(path.parent().expect("source parent")).expect("create source parent");
    fs::write(path, content).expect("write source");
}

#[test]
fn policy__root__guardrails__guardrails_smoke() {
    guardrails();
}

#[test]
fn policy__root__guardrails__empty_source_tree_is_rejected() {
    let paths = TestPaths::new("policies-empty-source-tree");
    let crate_root = paths.child("empty-crate");
    fs::create_dir_all(crate_root.join("src")).expect("create src");

    let err =
        bijux_dna_policies::check(&crate_root, &GuardrailConfig::default()).expect_err("empty src");

    assert!(
        err.to_string().contains("at least one Rust source"),
        "unexpected guardrail error: {err}"
    );
}

#[test]
fn policy__root__guardrails__allow_paths_match_exact_suffixes() {
    let paths = TestPaths::new("policies-allow-path-suffix");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "allowed_extra.rs", "pub fn fail() { None::<u8>.expect(\"x\"); }\n");

    let mut config = GuardrailConfig::default();
    config.forbid_panic_expect = true;
    config.allow_panic_expect_paths = vec!["/src/allowed".to_string()];

    let err = bijux_dna_policies::check(&crate_root, &config)
        .expect_err("substring allowlist must not suppress failure");

    assert!(err.to_string().contains("panic/expect found"), "unexpected guardrail error: {err}");
}
