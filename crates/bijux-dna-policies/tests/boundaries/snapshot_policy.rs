#![allow(non_snake_case)]
#[path = "../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const SNAPSHOT_BUCKETS: &[&str] =
    &["contracts", "schemas", "semantics", "determinism", "boundaries", "fixtures", "snapshots"];
const SNAPSHOT_EXTENSIONS: &[&str] = &["snap", "json", "txt", "jsonl", "html"];

fn crate_name_for(root: &std::path::Path) -> String {
    let manifest = root.join("Cargo.toml");
    let Ok(content) = std::fs::read_to_string(&manifest) else {
        return root.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string();
    };
    let mut in_package = false;
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_package = line == "[package]";
            continue;
        }
        if !in_package || line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((name, _)) = line.split_once('=') {
            if name.trim() == "name" {
                return line
                    .split_once('=')
                    .map(|(_, val)| val.trim().trim_matches('"').to_string())
                    .unwrap_or_default();
            }
        }
    }
    root.file_name().and_then(|name| name.to_str()).unwrap_or("").to_string()
}

#[test]
fn policy__boundaries__snapshot_policy__snapshot_files_live_in_snapshots_dir() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SNAPSHOT_EXTENSIONS.iter().any(|ext| name.ends_with(&format!(".{ext}"))) {
                continue;
            }
            if path.to_string_lossy().contains("tests/snapshots/") {
                continue;
            }
            if path.to_string_lossy().contains("tests/contracts/snapshots/") {
                continue;
            }
            if path.to_string_lossy().contains("tests/fixtures/") {
                continue;
            }
            if std::path::Path::new(name).extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("json") || ext.eq_ignore_ascii_case("txt")
            }) {
                continue; // allow non-snapshot artifacts outside snapshots dir
            }
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "snapshot files must live under tests/snapshots.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__snapshot_policy__snapshot_names_are_normalized() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        let crate_name = crate_name_for(&crate_root);
        let snapshots_dir = crate_root.join("tests").join("snapshots");
        if !snapshots_dir.exists() {
            continue;
        }
        for entry in WalkDir::new(&snapshots_dir)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
        {
            let path = entry.path();
            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !SNAPSHOT_EXTENSIONS.contains(&extension) {
                continue;
            }
            if !file_name.starts_with(&format!("{crate_name}__")) {
                offenders.push(path.display().to_string());
                continue;
            }
            let parts: Vec<&str> = file_name.splitn(4, "__").collect();
            if parts.len() < 3 || !SNAPSHOT_BUCKETS.contains(&parts[1]) {
                offenders.push(path.display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "snapshot files must be named `<crate>__<bucket>__<name>.<ext>`.\n\
Allowed buckets: {}.\n\
Offenders:\n{}",
        SNAPSHOT_BUCKETS.join(", "),
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__snapshot_policy__tests_document_snapshot_intent() {
    let mut offenders = Vec::new();
    for crate_root in support::crate_roots() {
        for entry in WalkDir::new(&crate_root)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| entry.path().extension().and_then(|e| e.to_str()) == Some("rs"))
        {
            let content = support::read_to_string(entry.path());
            if (content.contains("assert_snapshot!") || content.contains("assert_json_snapshot!"))
                && !content.contains("///")
            {
                offenders.push(entry.path().display().to_string());
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "snapshot tests must include a doc comment explaining intent.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
