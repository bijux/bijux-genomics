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
fn policy__root__guardrails__missing_source_tree_is_rejected() {
    let paths = TestPaths::new("policies-missing-source-tree");
    let crate_root = paths.child("missing-src-crate");
    fs::create_dir_all(&crate_root).expect("create crate root");

    let err =
        bijux_dna_policies::check(&crate_root, &GuardrailConfig::default()).expect_err("no src");

    assert!(
        err.to_string().contains("No such file") || err.to_string().contains("os error"),
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

#[test]
fn policy__root__guardrails__panic_expect_scan_ignores_comment_mentions() {
    let paths = TestPaths::new("policies-panic-comments");
    let crate_root = paths.child("crate");
    write_source(
        &crate_root,
        "lib.rs",
        "// None::<u8>.expect(\"documented only\")\npub fn ok() {}\n",
    );

    let mut config = GuardrailConfig::default();
    config.forbid_panic_expect = true;

    bijux_dna_policies::check(&crate_root, &config).expect("comment-only expect is allowed");
}

#[test]
fn policy__root__guardrails__panic_expect_scan_rejects_unwrap() {
    let paths = TestPaths::new("policies-unwrap-scan");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "lib.rs", "pub fn fail() { Some(1).unwrap(); }\n");

    let mut config = GuardrailConfig::default();
    config.forbid_panic_expect = true;

    let err = bijux_dna_policies::check(&crate_root, &config).expect_err("unwrap must fail");

    assert!(err.to_string().contains("unwrap found"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__stage_id_scan_ignores_comment_mentions() {
    let paths = TestPaths::new("policies-stage-id-comments");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "lib.rs", "// example: \"fastq.qc\"\npub fn ok() {}\n");

    let mut config = GuardrailConfig::default();
    config.forbid_stage_id_strings = true;

    bijux_dna_policies::check(&crate_root, &config).expect("comment-only stage id is allowed");
}

#[test]
fn policy__root__guardrails__stage_id_scan_rejects_raw_strings() {
    let paths = TestPaths::new("policies-stage-id-raw-strings");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "lib.rs", "pub const STAGE: &str = r#\"fastq.qc\"#;\n");

    let mut config = GuardrailConfig::default();
    config.forbid_stage_id_strings = true;

    let err = bijux_dna_policies::check(&crate_root, &config).expect_err("raw stage id must fail");

    assert!(err.to_string().contains("stage id literal"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__pub_item_budget_counts_scoped_visibility() {
    let paths = TestPaths::new("policies-scoped-pub-budget");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "lib.rs", "pub(super) fn visible_to_parent() {}\n");

    let mut config = GuardrailConfig::default();
    config.max_pub_items_per_file = 0;

    let err = bijux_dna_policies::check(&crate_root, &config).expect_err("scoped pub must count");

    assert!(err.to_string().contains("pub items"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__pub_use_budget_counts_scoped_reexports() {
    let paths = TestPaths::new("policies-scoped-pub-use-budget");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "lib.rs", "mod inner {}\npub(crate) use inner as exposed;\n");

    let mut config = GuardrailConfig::default();
    config.forbid_pub_use_spam = true;
    config.max_pub_use_per_file = 0;

    let err =
        bijux_dna_policies::check(&crate_root, &config).expect_err("scoped pub use must count");

    assert!(err.to_string().contains("pub use re-exports"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__empty_module_scan_ignores_attributes_and_scoped_mods() {
    let paths = TestPaths::new("policies-empty-mod-scoped");
    let crate_root = paths.child("crate");
    write_source(&crate_root, "mod.rs", "#![allow(dead_code)]\npub(crate) mod inner;\n");
    write_source(&crate_root, "inner.rs", "pub fn real() {}\n");

    let err = bijux_dna_policies::check(&crate_root, &GuardrailConfig::default())
        .expect_err("attribute-only module shell must fail");

    assert!(err.to_string().contains("empty module file"), "unexpected guardrail error: {err}");
}
