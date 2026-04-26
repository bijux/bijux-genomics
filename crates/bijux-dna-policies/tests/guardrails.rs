#![allow(non_snake_case)]

use anyhow::Result;
use bijux_dna_policies::GuardrailConfig;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn guardrails() {
    let _config = GuardrailConfig::for_crate("bijux-dna-policies");
}

fn test_root(test_name: &str) -> PathBuf {
    let counter = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir()
        .join(format!("bijux-dna-policies-{test_name}-{}-{counter}", std::process::id()))
}

fn write_source(crate_root: &Path, rel: &str, content: &str) {
    let path = crate_root.join("src").join(rel);
    let parent = path
        .parent()
        .unwrap_or_else(|| panic!("source path must have a parent: {}", path.display()));
    fs::create_dir_all(parent)
        .unwrap_or_else(|err| panic!("create source parent {} failed: {err}", parent.display()));
    fs::write(&path, content)
        .unwrap_or_else(|err| panic!("write source {} failed: {err}", path.display()));
}

fn create_dir(path: impl AsRef<Path>) {
    let path = path.as_ref();
    fs::create_dir_all(path)
        .unwrap_or_else(|err| panic!("create directory {} failed: {err}", path.display()));
}

fn must_pass(result: Result<()>, context: &str) {
    if let Err(err) = result {
        panic!("{context}: {err}");
    }
}

fn must_fail(result: Result<()>, context: &str) -> anyhow::Error {
    match result {
        Ok(()) => panic!("{context}"),
        Err(err) => err,
    }
}

#[test]
fn policy__root__guardrails__guardrails_smoke() {
    guardrails();
}

#[test]
fn policy__root__guardrails__empty_source_tree_is_rejected() {
    let crate_root = test_root("empty-source-tree").join("empty-crate");
    create_dir(crate_root.join("src"));

    let err =
        must_fail(bijux_dna_policies::check(&crate_root, &GuardrailConfig::default()), "empty src");

    assert!(
        err.to_string().contains("at least one Rust source"),
        "unexpected guardrail error: {err}"
    );
}

#[test]
fn policy__root__guardrails__missing_source_tree_is_rejected() {
    let crate_root = test_root("missing-source-tree").join("missing-src-crate");
    create_dir(&crate_root);

    let err =
        must_fail(bijux_dna_policies::check(&crate_root, &GuardrailConfig::default()), "no src");

    assert!(
        err.to_string().contains("No such file") || err.to_string().contains("os error"),
        "unexpected guardrail error: {err}"
    );
}

#[test]
fn policy__root__guardrails__allow_paths_match_exact_suffixes() {
    let crate_root = test_root("allow-path-suffix").join("crate");
    write_source(&crate_root, "allowed_extra.rs", "pub fn fail() { None::<u8>.expect(\"x\"); }\n");

    let config = GuardrailConfig {
        forbid_panic_expect: true,
        allow_panic_expect_paths: vec!["/src/allowed".to_string()],
        ..Default::default()
    };

    let err = must_fail(
        bijux_dna_policies::check(&crate_root, &config),
        "substring allowlist must not suppress failure",
    );

    assert!(
        err.to_string().contains("panic/expect/unwrap found"),
        "unexpected guardrail error: {err}"
    );
}

#[test]
fn policy__root__guardrails__panic_expect_scan_ignores_comment_mentions() {
    let crate_root = test_root("panic-comments").join("crate");
    write_source(
        &crate_root,
        "lib.rs",
        "// None::<u8>.expect(\"documented only\")\npub fn ok() {}\n",
    );

    let config = GuardrailConfig { forbid_panic_expect: true, ..Default::default() };

    must_pass(bijux_dna_policies::check(&crate_root, &config), "comment-only expect is allowed");
}

#[test]
fn policy__root__guardrails__panic_expect_scan_rejects_unwrap() {
    let crate_root = test_root("unwrap-scan").join("crate");
    write_source(&crate_root, "lib.rs", "pub fn fail() { Some(1).unwrap(); }\n");

    let config = GuardrailConfig { forbid_panic_expect: true, ..Default::default() };

    let err = must_fail(bijux_dna_policies::check(&crate_root, &config), "unwrap must fail");

    assert!(err.to_string().contains("unwrap found"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__stage_id_scan_ignores_comment_mentions() {
    let crate_root = test_root("stage-id-comments").join("crate");
    write_source(&crate_root, "lib.rs", "// example: \"fastq.qc\"\npub fn ok() {}\n");

    let config = GuardrailConfig { forbid_stage_id_strings: true, ..Default::default() };

    must_pass(bijux_dna_policies::check(&crate_root, &config), "comment-only stage id is allowed");
}

#[test]
fn policy__root__guardrails__stage_id_scan_rejects_raw_strings() {
    let crate_root = test_root("stage-id-raw-strings").join("crate");
    write_source(&crate_root, "lib.rs", "pub const STAGE: &str = r#\"fastq.qc\"#;\n");

    let config = GuardrailConfig { forbid_stage_id_strings: true, ..Default::default() };

    let err = must_fail(bijux_dna_policies::check(&crate_root, &config), "raw stage id must fail");

    assert!(err.to_string().contains("stage id literal"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__pub_item_budget_counts_scoped_visibility() {
    let crate_root = test_root("scoped-pub-budget").join("crate");
    write_source(&crate_root, "lib.rs", "pub(super) fn visible_to_parent() {}\n");

    let config = GuardrailConfig { max_pub_items_per_file: 0, ..Default::default() };

    let err = must_fail(bijux_dna_policies::check(&crate_root, &config), "scoped pub must count");

    assert!(err.to_string().contains("pub items"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__pub_use_budget_counts_scoped_reexports() {
    let crate_root = test_root("scoped-pub-use-budget").join("crate");
    write_source(&crate_root, "lib.rs", "mod inner {}\npub(crate) use inner as exposed;\n");

    let config = GuardrailConfig {
        forbid_pub_use_spam: true,
        max_pub_use_per_file: 0,
        ..Default::default()
    };

    let err =
        must_fail(bijux_dna_policies::check(&crate_root, &config), "scoped pub use must count");

    assert!(err.to_string().contains("pub use re-exports"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__empty_module_scan_ignores_attributes_and_scoped_mods() {
    let crate_root = test_root("empty-mod-scoped").join("crate");
    write_source(&crate_root, "mod.rs", "#![allow(dead_code)]\npub(crate) mod inner;\n");
    write_source(&crate_root, "inner.rs", "pub fn real() {}\n");

    let err = must_fail(
        bijux_dna_policies::check(&crate_root, &GuardrailConfig::default()),
        "attribute-only module shell must fail",
    );

    assert!(err.to_string().contains("empty module file"), "unexpected guardrail error: {err}");
}

#[test]
fn policy__root__guardrails__literal_scans_ignore_block_comments() {
    let crate_root = test_root("block-comment-scans").join("crate");
    write_source(
        &crate_root,
        "lib.rs",
        "/*\nNone::<u8>.expect(\"comment only\");\n\"fastq.qc\"\n*/\npub fn ok() {}\n",
    );

    let config = GuardrailConfig {
        forbid_panic_expect: true,
        forbid_stage_id_strings: true,
        ..Default::default()
    };

    must_pass(
        bijux_dna_policies::check(&crate_root, &config),
        "block-comment mentions are allowed",
    );
}
