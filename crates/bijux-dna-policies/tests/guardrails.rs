#![allow(non_snake_case)]

use anyhow::Result;
use bijux_dna_policies::GuardrailConfig;
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

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

    must_pass(
        bijux_dna_policies::check(&crate_root, &GuardrailConfig::default()),
        "module router with sourced scoped module is allowed",
    );
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

fn write_equivalent_public_alias_surface(crate_root: &Path, grouped: bool, count: usize) {
    write_source(crate_root, "lib.rs", "mod definitions;\npub mod exports;\n");
    write_source(crate_root, "definitions.rs", "pub struct Value;\n");
    let aliases = (0..count).map(|index| format!("Public{index}")).collect::<Vec<_>>();
    let exports = if grouped {
        format!(
            "pub use crate::definitions::{{\n{}\n}};\n",
            aliases
                .iter()
                .map(|name| format!("    Value as {name},"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    } else {
        aliases.iter().fold(String::new(), |mut output, name| {
            writeln!(output, "pub use crate::definitions::Value as {name};")
                .unwrap_or_else(|error| panic!("write public alias: {error}"));
            output
        })
    };
    write_source(crate_root, "exports.rs", &exports);
    let consumer = format!(
        "pub fn same_types(value: &owned_surface::exports::Public0) {{\n{}\n}}\n",
        aliases
            .iter()
            .map(|name| format!("    let _: &owned_surface::exports::{name} = value;"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    fs::write(crate_root.join("consumer.rs"), consumer)
        .unwrap_or_else(|error| panic!("write actual public consumer: {error}"));
}

#[test]
fn equivalent_public_alias_grouping_preserves_guardrail_verdict() {
    let root = test_root("equivalent-public-aliases");
    let grouped = root.join("grouped");
    let separated = root.join("separated");
    let config = GuardrailConfig::for_crate("bijux-dna-api");
    let count = config
        .max_pub_items_per_file
        .checked_add(1)
        .unwrap_or_else(|| panic!("bounded counterexample size"));
    write_equivalent_public_alias_surface(&grouped, true, count);
    write_equivalent_public_alias_surface(&separated, false, count);
    must_pass(
        bijux_dna_policies::check(&grouped, &config),
        "same public alias set grouped control",
    );
    assert_eq!(
        fs::read(grouped.join("src/definitions.rs"))
            .unwrap_or_else(|error| panic!("grouped definition: {error}")),
        fs::read(separated.join("src/definitions.rs"))
            .unwrap_or_else(|error| panic!("separated definition: {error}"))
    );
    assert_eq!(
        fs::read(grouped.join("consumer.rs"))
            .unwrap_or_else(|error| panic!("grouped consumer: {error}")),
        fs::read(separated.join("consumer.rs"))
            .unwrap_or_else(|error| panic!("separated consumer: {error}"))
    );
    eprintln!(
        "equivalent_public_surface group={} separated={} alias_count={count}",
        grouped.display(),
        separated.display()
    );
    must_pass(
        bijux_dna_policies::check(&separated, &config),
        "identical public identities cannot be rejected solely by reexport statement grouping",
    );
}

#[test]
fn public_surface_layout_keeps_real_namespace_refusals() {
    let root = test_root("public-surface-namespace-refusal");
    let config = GuardrailConfig::for_crate("bijux-dna-api");
    write_source(&root, "lib.rs", "pub fn retained_capability() {}\n");
    must_pass(bijux_dna_policies::check(&root, &config), "ordinary owned capability");
    write_source(&root, "unresolved/mod.rs", "pub mod absent;\n");
    let error = must_fail(
        bijux_dna_policies::check(&root, &config),
        "an unresolved namespace must retain its real structural refusal",
    );
    assert!(
        error.to_string().contains("module directory contains only mod.rs"),
        "unexpected namespace refusal: {error}"
    );
}
