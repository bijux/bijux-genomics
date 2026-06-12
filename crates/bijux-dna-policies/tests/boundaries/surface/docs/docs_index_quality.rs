#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::{crate_roots, read_to_string};

#[test]
fn policy__boundaries__docs_index_quality__docs_index_has_required_sections() {
    for crate_root in crate_roots() {
        let index = crate_root.join("docs").join("INDEX.md");
        if !index.exists() {
            continue;
        }
        let content = read_to_string(&index);
        let has_h1 = content.lines().any(|line| line.starts_with("# "));
        let has_section = content.lines().any(|line| line.starts_with("## "));
        if !has_h1 || !has_section {
            bijux_dna_policies::policy_panic!(
                "docs/INDEX.md must have an H1 and at least one section in {}",
                index.display()
            );
        }
    }
}

#[test]
fn policy__boundaries__docs_index_quality__docs_index_links_are_valid() {
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        for entry in WalkDir::new(&docs) {
            let entry = entry.expect("walk docs");
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().file_name().and_then(|n| n.to_str()) != Some("INDEX.md") {
                continue;
            }
            let content = read_to_string(entry.path());
            let base = entry.path().parent().unwrap_or(&docs);
            for line in content.lines() {
                let mut rest = line;
                while let Some(open) = rest.find('[') {
                    if let Some(close) = rest[open..].find("](") {
                        let start = open + close + 2;
                        if let Some(end) = rest[start..].find(')') {
                            let link = &rest[start..start + end];
                            if link.starts_with("http") || link.starts_with('#') {
                                rest = &rest[start + end + 1..];
                                continue;
                            }
                            let target = base.join(link);
                            if !target.exists() {
                                bijux_dna_policies::policy_panic!(
                                    "broken link in {}: {} -> {}",
                                    entry.path().display(),
                                    link,
                                    target.display()
                                );
                            }
                            rest = &rest[start + end + 1..];
                            continue;
                        }
                    }
                    break;
                }
            }
        }
    }
}
