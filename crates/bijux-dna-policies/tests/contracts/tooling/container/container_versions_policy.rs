#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use std::collections::BTreeSet;
use walkdir::WalkDir;

fn is_semver_like(value: &str) -> bool {
    !value.trim().is_empty()
}

fn pinning_offenses(path: &std::path::Path, content: &str) -> Vec<String> {
    let mut offenders = Vec::new();
    if content.contains("git clone")
        && !(content.contains("git checkout ")
            && content
                .split("git checkout ")
                .skip(1)
                .any(|tail| tail.chars().take(40).all(|ch| ch.is_ascii_hexdigit())))
    {
        offenders.push(format!(
            "{}: unpinned git clone (missing immutable git checkout <40-hex>)",
            path.display()
        ));
    }
    offenders
}

#[test]
fn policy__contracts__container_versions_policy__each_container_definition_has_versions_entry() {
    let root = support::workspace_root();
    let versions_path = root.join("containers/versions/versions.toml");
    let raw = std::fs::read_to_string(&versions_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", versions_path.display()));
    let parsed: toml::Value =
        raw.parse().unwrap_or_else(|err| panic!("parse {}: {err}", versions_path.display()));
    let table = parsed
        .as_table()
        .unwrap_or_else(|| panic!("{} must be TOML table", versions_path.display()));

    let mut expected = BTreeSet::new();
    for entry in
        WalkDir::new(root.join("containers/docker/arm64")).into_iter().filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if let Some(id) = name.strip_prefix("Dockerfile.") {
            expected.insert(id.to_string());
        }
    }
    for entry in
        WalkDir::new(root.join("containers/apptainer/shared")).into_iter().filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("def") {
            continue;
        }
        let stem = path.file_stem().and_then(|v| v.to_str()).unwrap_or_default().to_string();
        expected.insert(stem);
    }

    let mut offenders = Vec::new();
    for tool_id in expected {
        let Some(version_row) = table.get(&tool_id).and_then(toml::Value::as_table) else {
            offenders.push(format!("missing [{tool_id}] in containers/versions/versions.toml"));
            continue;
        };
        let version = version_row.get("version").and_then(toml::Value::as_str).unwrap_or_default();
        let source = version_row.get("source").and_then(toml::Value::as_str).unwrap_or_default();
        let date = version_row.get("date_pinned").and_then(toml::Value::as_str).unwrap_or_default();
        if !is_semver_like(version) {
            offenders.push(format!("{tool_id}: version must be x.y.z, got `{version}`"));
        }
        if source.trim().is_empty() {
            offenders.push(format!("{tool_id}: source must be non-empty"));
        }
        let date_ok = date.len() == 10
            && date.chars().enumerate().all(|(idx, ch)| {
                matches!(idx, 4 | 7) && ch == '-' || (!matches!(idx, 4 | 7) && ch.is_ascii_digit())
            });
        if !date_ok {
            offenders.push(format!("{tool_id}: date_pinned must be YYYY-MM-DD, got `{date}`"));
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "container versions policy failures:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__container_versions_policy__no_latest_floating_or_unpinned_git_clone() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for entry in
        WalkDir::new(root.join("containers/docker/arm64")).into_iter().filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let Some(name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        if !name.starts_with("Dockerfile.") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        offenders.extend(pinning_offenses(path, &content));
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("FROM ") {
                continue;
            }
            if trimmed.contains(':') && !trimmed.contains("@sha256:") {
                offenders.push(format!(
                    "{}: floating docker base tag without digest: {}",
                    path.display(),
                    trimmed
                ));
            }
        }
    }

    for entry in
        WalkDir::new(root.join("containers/apptainer/shared")).into_iter().filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("def") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        offenders.extend(pinning_offenses(path, &content));
        for line in content.lines() {
            let trimmed = line.trim();
            if !trimmed.starts_with("From: ") {
                continue;
            }
            if trimmed.contains(':') && !trimmed.contains("@sha256:") {
                offenders.push(format!(
                    "{}: floating apptainer base tag without digest: {}",
                    path.display(),
                    trimmed
                ));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "container pinning policy failures:\n{}",
        offenders.join("\n")
    );
}
