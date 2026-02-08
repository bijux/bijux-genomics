#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::path::PathBuf;

use walkdir::WalkDir;

use support::{crate_roots, read_to_string};

#[test]
fn policy__surface__docs_index_quality__docs_index_has_required_sections() {
    let required = [
        "## Scope",
        "## Effects",
        "## Boundaries",
        "## Extension Points",
        "## How to Test",
    ];

    for crate_root in crate_roots() {
        let index = crate_root.join("docs").join("INDEX.md");
        if !index.exists() {
            bijux_policies::policy_panic!("missing docs/INDEX.md in {}", crate_root.display());
        }
        let content = read_to_string(&index);
        for heading in required {
            if !content.contains(heading) {
                bijux_policies::policy_panic!(
                    "docs/INDEX.md missing required section {heading} in {}",
                    index.display()
                );
            }
        }
    }
}

#[test]
fn policy__surface__docs_index_quality__docs_index_links_are_valid() {
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
                                bijux_policies::policy_panic!(
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
