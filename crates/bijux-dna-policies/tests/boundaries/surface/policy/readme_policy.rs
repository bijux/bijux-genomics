#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::HashMap;
use std::fmt::Write as _;
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
                let target = &rest[start..start + end];
                links.push(target.to_string());
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
    let required_heading_groups: [(&str, &[&str]); 5] = [
        (
            "crate purpose",
            &[
                "## What this crate does",
                "## Scope",
                "## Responsibilities",
                "## Ownership",
                "## Role",
            ],
        ),
        (
            "boundaries",
            &[
                "## What it must not do (boundaries)",
                "## Boundaries",
                "## Boundary",
                "## Scope",
                "## Responsibilities",
                "## Ownership",
                "## Role",
            ],
        ),
        (
            "public entrypoints",
            &[
                "## Public API / entrypoints",
                "## Public entrypoints",
                "## Public API",
                "## Public Surface",
                "## Public Operations",
                "## Managed Operations",
                "## Managed Commands",
                "## Command Surface",
                "## Entry Points",
            ],
        ),
        (
            "contract ownership",
            &[
                "## Key contracts it owns/consumes",
                "## Contracts and operating rules",
                "## Architecture",
                "## Internal layout",
                "## Execution lifecycle",
                "## Generated Outputs",
                "## Start In Code",
                "## Entry Points",
                "## Ownership",
                "## Public Surface",
            ],
        ),
        (
            "test execution",
            &["## How to run its tests", "## Tests", "## Verification", "## Validation"],
        ),
    ];

    let mut fingerprints: HashMap<String, PathBuf> = HashMap::new();

    for crate_root in crate_roots() {
        let readme = crate_root.join("README.md");
        let content = read_to_string(&readme);
        for (group, alternatives) in required_heading_groups {
            bijux_dna_policies::policy_assert!(
                alternatives.iter().any(|heading| content.contains(heading)),
                "README missing required heading group ({group}) in {}. Accepted headings: {:?}",
                readme.display(),
                alternatives
            );
        }
        bijux_dna_policies::policy_assert!(
            content.contains("docs/INDEX.md") || content.contains("docs/ARCHITECTURE.md"),
            "README must link to docs/INDEX.md or docs/ARCHITECTURE.md in {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            content.contains("docs/TESTS.md"),
            "README must link to docs/TESTS.md in {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            {
                let lower = content.to_lowercase();
                lower.contains("owns")
                    || lower.contains("owned")
                    || lower.contains("ownership")
                    || lower.contains("own ")
            },
            "README must state what the crate owns: {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            {
                let lower = content.to_lowercase();
                lower.contains("must not")
                    || lower.contains("does not")
                    || lower.contains("without ")
            },
            "README must state boundaries (must not/does not): {}",
            readme.display()
        );
        bijux_dna_policies::policy_assert!(
            {
                let lower = content.to_lowercase();
                lower.contains("effects") || lower.contains("effect") || lower.contains("artifact")
            },
            "README must mention effect boundaries: {}",
            readme.display()
        );
        let test_mentions = content.matches("tests/").count()
            + content.matches(".rs").count()
            + content.matches("--test").count();
        bijux_dna_policies::policy_assert!(
            test_mentions >= 1 || content.contains("docs/TESTS.md"),
            "README must mention tests or link docs/TESTS.md: {}",
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
        let hash = sha256_hex(Sha256::digest(normalized.as_bytes()));
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

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
