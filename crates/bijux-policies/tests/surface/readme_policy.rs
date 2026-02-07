use std::path::{Path, PathBuf};

use crate::support::fs::{crate_roots, read_to_string};

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
fn readme_has_required_sections_and_links() {
    let required = [
        "## What this crate does",
        "## What it must not do (boundaries)",
        "## Effects & determinism guarantees",
        "## Where the docs live",
    ];

    for crate_root in crate_roots() {
        let readme = crate_root.join("README.md");
        let content = read_to_string(&readme);
        for heading in required {
            assert!(
                content.contains(heading),
                "README missing required heading {heading} in {}",
                readme.display()
            );
        }
        assert!(
            content.contains("docs/INDEX.md"),
            "README must link to docs/INDEX.md in {}",
            readme.display()
        );

        let base = readme.parent().unwrap_or(&crate_root);
        for line in content.lines() {
            for link in find_links(line) {
                if let Some(target) = resolve_link(base, &link) {
                    assert!(
                        target.exists(),
                        "README has broken link {} -> {}",
                        link,
                        target.display()
                    );
                }
            }
        }
    }
}
