#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn policy__boundaries__public_api_docs__public_api_docs_match_curated_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs =
        std::fs::read_to_string(root.join("docs/PUBLIC_API.md")).expect("read docs/PUBLIC_API.md");

    assert_eq!(
        markdown_list_after_heading(&docs, "## Public Modules"),
        entries(["policy_diagnostics", "public_api"]),
        "PUBLIC_API.md must document the public modules exported from src/lib.rs"
    );
    assert_eq!(
        markdown_list_after_heading(&docs, "## Root Exports"),
        entries([
            "check",
            "GuardrailConfig",
            "policy_assert!",
            "policy_assert_eq!",
            "policy_assert_ne!",
            "policy_panic!",
        ]),
        "PUBLIC_API.md must document the stable root exports"
    );
}

#[test]
fn policy__boundaries__public_api_docs__documented_root_exports_remain_compilable() {
    use anyhow::Result;
    use bijux_dna_policies::GuardrailConfig;
    use std::path::Path;

    let _: fn(&Path, &GuardrailConfig) -> Result<()> = bijux_dna_policies::check;
    let _: Option<GuardrailConfig> = None;
    bijux_dna_policies::policy_assert!(true, "policy_assert! must be exported");
    bijux_dna_policies::policy_assert_eq!(1, 1, "policy_assert_eq! must be exported");
    bijux_dna_policies::policy_assert_ne!(1, 2, "policy_assert_ne! must be exported");
}

fn markdown_list_after_heading(markdown: &str, heading: &str) -> BTreeSet<String> {
    let mut values = BTreeSet::new();
    let mut in_section = false;

    for line in markdown.lines() {
        if line == heading {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if !in_section {
            continue;
        }
        if let Some(item) = line.strip_prefix("- `").and_then(|line| line.strip_suffix('`')) {
            values.insert(item.to_string());
        }
    }

    values
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
