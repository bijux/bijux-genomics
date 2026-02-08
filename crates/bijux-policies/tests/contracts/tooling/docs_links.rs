#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use support::{read_to_string, workspace_root};

fn docs_root() -> PathBuf {
    workspace_root().join("docs")
}

fn resolve_link(base: &Path, link: &str) -> Option<PathBuf> {
    if link.starts_with("http://")
        || link.starts_with("https://")
        || link.starts_with("mailto:")
        || link.starts_with('#')
    {
        return None;
    }
    let link = link.split('#').next().unwrap_or(link);
    if link.is_empty() {
        return None;
    }
    let target = base.join(link);
    Some(target)
}

#[test]
fn policy__tooling__docs_links__docs_links_are_resolvable() {
    let root = docs_root();
    for entry in WalkDir::new(&root) {
        let entry = entry.expect("walk docs");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
            continue;
        }
        let content = read_to_string(entry.path());
        let base = entry.path().parent().unwrap_or(&root);
        for line in content.lines() {
            let mut rest = line;
            while let Some(open) = rest.find('[') {
                if let Some(close) = rest[open..].find("](") {
                    let start = open + close + 2;
                    if let Some(end) = rest[start..].find(')') {
                        let link = &rest[start..start + end];
                        if let Some(target) = resolve_link(base, link) {
                            if !target.exists() {
                                bijux_policies::policy_panic!(
                                    "broken link in {}: {} -> {}",
                                    entry.path().display(),
                                    link,
                                    target.display()
                                );
                            }
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
