#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use support::{crate_roots, read_to_string};

fn find_links(line: &str) -> Vec<String> {
    let mut links = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        if let Some(close) = rest[open..].find("](") {
            let start = open + close + 2;
            if let Some(end) = rest[start..].find(')') {
                let link = &rest[start..start + end];
                links.push(link.to_string());
                rest = &rest[start + end + 1..];
                continue;
            }
        }
        break;
    }
    links
}

fn resolve_link(base: &Path, link: &str) -> Option<PathBuf> {
    if link.starts_with("http") || link.starts_with('#') {
        return None;
    }
    let link = link.split('#').next().unwrap_or(link);
    if link.is_empty() {
        return None;
    }
    Some(base.join(link))
}

#[test]
fn policy__boundaries__readme_policy__readme_has_required_sections_and_links() {
    let required = [
        "## What this crate does",
        "## What it must not do (boundaries)",
        "## Effects & determinism guarantees",
        "## Public API / entrypoints",
        "## Key contracts it owns/consumes",
        "## Artifacts / Contracts",
        "## Failure modes",
        "## How to run its tests",
        "## Where the docs live",
    ];

    let mut fingerprints: HashMap<String, PathBuf> = HashMap::new();

    for crate_root in crate_roots() {
        let readme = crate_root.join("README.md");
        let content = read_to_string(&readme);
        for heading in required {
            bijux_dna_policies::policy_assert!(
                content.contains(heading),
                "README missing required heading {heading} in {}",
                readme.display()
            );
        }
        bijux_dna_policies::policy_assert!(
            content.contains("docs/INDEX.md"),
            "README must link to docs/INDEX.md in {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            content.contains("docs/TESTS.md"),
            "README must link to docs/TESTS.md in {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            content.to_lowercase().contains("owns"),
            "README must state what the crate owns: {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            content.to_lowercase().contains("must not"),
            "README must state boundaries (must not): {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            content.to_lowercase().contains("effects"),
            "README must mention effects: {}",
            readme.display()
        );
        let test_mentions = content.matches("tests/").count() + content.matches(".rs").count();
        bijux_dna_policies::policy_assert!(
            test_mentions >= 3,
            "README must mention 3+ test files: {}",
            readme.display()
        );

        let base = readme.parent().unwrap_or(&crate_root);
        for line in content.lines() {
            for link in find_links(line) {
                if let Some(target) = resolve_link(base, &link) {
                    bijux_dna_policies::policy_assert!(
                        target.exists(),
                        "README has broken link {} -> {}",
                        link,
                        target.display()
                    );
                }
            }
        }

        let mut normalized = String::new();
        for line in content.lines() {
            if line.starts_with('#') {
                continue;
            }
            if line.contains("docs/INDEX.md") || line.contains("docs/TESTS.md") {
                continue;
            }
            normalized.push_str(line.trim());
        }
        let hash = format!("{:x}", Sha256::digest(normalized.as_bytes()));
        if let Some(existing) = fingerprints.get(&hash) {
            bijux_dna_policies::policy_panic!(
                "Duplicate README bodies detected: {} and {}",
                existing.display(),
                readme.display()
            );
        }
        fingerprints.insert(hash, readme);
    }
}
