#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const MIN_MOD_LINES: usize = 5;
const DIR_ALLOWLIST: &[(&str, &str)] = &[
    (
        "/crates/bijux-dna-core/src/public_api/metrics",
        "public API surface is intentionally namespaced for stable growth",
    ),
    (
        "/crates/bijux-dna-core/src/public_api/identity",
        "public API surface is intentionally namespaced for stable growth",
    ),
    (
        "/crates/bijux-dna-core/src/public_api/contracts",
        "public API surface is intentionally namespaced for stable growth",
    ),
    (
        "/crates/bijux-dna-core/src/public_api/catalog",
        "public API surface is intentionally namespaced for stable growth",
    ),
    (
        "/crates/bijux-dna-core/src/public_api/ergonomics",
        "public API surface is intentionally namespaced for stable growth",
    ),
];

#[test]
fn policy__boundaries__no_empty_dirs_policy__no_empty_or_placeholder_dirs() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_dir())
        {
            let dir = entry.path();
            let dir_str = dir.to_string_lossy();
            if DIR_ALLOWLIST.iter().any(|(allowed, _reason)| dir_str.contains(allowed)) {
                continue;
            }
            let mut rs_files = Vec::new();
            let mut has_any_file = false;
            let mut has_subdir = false;
            let mut file_count = 0usize;
            let mut keep_reason = None;
            if let Ok(read_dir) = std::fs::read_dir(dir) {
                for child in read_dir.flatten() {
                    let path = child.path();
                    if path.is_dir() {
                        has_subdir = true;
                    }
                    if path.is_file() {
                        has_any_file = true;
                        file_count += 1;
                        if path.file_name().and_then(|n| n.to_str()) == Some(".keep") {
                            let contents = support::read_to_string(&path);
                            if !contents.trim().is_empty() {
                                keep_reason = Some(contents.trim().to_string());
                            }
                        }
                    }
                    if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
                        rs_files.push(path);
                    }
                }
            }
            if !has_any_file && !has_subdir {
                offenders.push(dir.display().to_string());
                continue;
            }
            if dir.to_string_lossy().contains("tests/fixtures") {
                let has_only_keep = file_count == 1
                    && std::fs::read_dir(dir)
                        .ok()
                        .and_then(|mut it| it.next())
                        .and_then(Result::ok)
                        .and_then(|entry| entry.file_name().into_string().ok())
                        .is_some_and(|name| name == ".keep");
                if has_only_keep && keep_reason.is_none() {
                    offenders.push(dir.display().to_string());
                    continue;
                }
            }
            if rs_files.len() == 1
                && rs_files[0].file_name().and_then(|n| n.to_str()) == Some("mod.rs")
            {
                let lines = support::read_to_string(&rs_files[0]).lines().count();
                if lines < MIN_MOD_LINES {
                    offenders.push(dir.display().to_string());
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "empty dirs or placeholder mod.rs modules are forbidden.\n\
Fix by removing empty dirs or adding real modules.\n\
tests/fixtures must either contain real files or a .keep with a reason.\n\
See docs/40-policies/STYLE.md for tree rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__no_empty_dirs_policy__support_dirs_are_documented() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let support_dir = crate_root.join("tests").join("support");
        if !support_dir.exists() {
            continue;
        }
        let entries = std::fs::read_dir(&support_dir)
            .ok()
            .map(|it| it.flatten().collect::<Vec<_>>())
            .unwrap_or_default();
        let has_rust_helper =
            entries.iter().any(|e| e.path().extension().and_then(|ext| ext.to_str()) == Some("rs"));
        if !has_rust_helper {
            offenders.push(support_dir.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "support/ dirs must contain Rust helpers instead of placeholder docs.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
