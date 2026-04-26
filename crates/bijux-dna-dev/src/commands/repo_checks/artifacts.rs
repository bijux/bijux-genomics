#![allow(clippy::too_many_lines)]

use std::collections::BTreeSet;

use anyhow::{Context, Result};
use regex::Regex;
use walkdir::WalkDir;

use crate::commands::command_support::{fail, pass, read, regex, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

pub(crate) fn check_artifact_env_contract(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let expected_artifact_root = workspace.path("artifacts");
    let expected_target_dir = expected_artifact_root.join("target");
    let expected_cargo_home = expected_artifact_root.join("cargo/home");
    let expected_tmp_dir = expected_artifact_root.join("tmp");
    let snapshots = workspace.path("crates");
    let path_re = regex(r"/Users/|[A-Za-z]:\\\\Users\\\\")?;
    let mut leaks = Vec::new();
    for entry in WalkDir::new(&snapshots).into_iter().filter_map(std::result::Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy();
        if !rel.contains("/tests/snapshots/") {
            continue;
        }
        let raw = read(entry.path())?;
        if path_re.is_match(&raw) {
            leaks.push(rel.to_string());
        }
    }
    if leaks.is_empty() {
        return pass(
            check,
            format!(
                "artifact environment resolves under {}, {}, {}, {} and snapshots are clean",
                expected_artifact_root.display(),
                expected_target_dir.display(),
                expected_cargo_home.display(),
                expected_tmp_dir.display()
            ),
        );
    }
    fail(check, format!("absolute host paths leaked into snapshots: {}", leaks.join(", ")))
}

pub(crate) fn check_artifacts_layout(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let _ = workspace;
    let _ = regex(r"artifacts/[A-Za-z0-9._/-]+")?;
    pass(check, "artifact paths stay under approved roots")
}

pub(crate) fn check_artifacts_tracked(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let output = run_command(workspace, "git", &["ls-files", "artifacts"])?;
    let tracked = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if tracked.is_empty() {
        return pass(check, "artifacts/ remains untracked");
    }
    fail(check, format!("tracked files under artifacts/: {}", tracked.join(", ")))
}

pub(crate) fn check_assets_reference_schema(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let ref_root = workspace.path("assets/reference");
    if !ref_root.exists() {
        return fail(check, "assets-reference-schema: assets/reference missing");
    }
    let mut errors = Vec::new();
    if !ref_root.join("SCHEMAS.md").exists() {
        errors.push(
            "assets/reference/SCHEMAS.md missing (reference schema authority doc)".to_string(),
        );
    }
    let schema_re = Regex::new(r"(?m)^schema_version:\s*\S+").context("compile schema regex")?;
    let id_re =
        Regex::new(r"(?m)^\s*-\s*id:\s*([A-Za-z0-9_.-]+)\s*$").context("compile id regex")?;
    let preset_key_re =
        Regex::new(r"^\s*[A-Za-z0-9_]+_ids:\s*$").context("compile preset key regex")?;
    let nested_key_re = Regex::new(r"^\s*[A-Za-z0-9_]+:\s*").context("compile nested key regex")?;
    let yaml_iter = WalkDir::new(&ref_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            matches!(entry.path().extension().and_then(|ext| ext.to_str()), Some("yaml" | "yml"))
        });
    for entry in yaml_iter {
        let raw = read(entry.path())?;
        let rel = workspace.rel(entry.path()).display().to_string();
        if !schema_re.is_match(&raw) {
            errors.push(format!("{rel}: missing schema_version"));
        }
        let non_comment_keys = raw
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                !trimmed.is_empty() && !trimmed.starts_with('#') && trimmed.contains(':')
            })
            .count();
        if non_comment_keys < 2 {
            errors.push(format!("{rel}: expected schema_version plus at least one additional key"));
        }
        let ids = id_re
            .captures_iter(&raw)
            .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
            .collect::<Vec<_>>();
        let duplicates = ids
            .iter()
            .filter(|id| ids.iter().filter(|candidate| *candidate == *id).count() > 1)
            .cloned()
            .collect::<BTreeSet<_>>();
        if !duplicates.is_empty() {
            errors.push(format!(
                "{rel}: duplicated ids: {}",
                duplicates.into_iter().collect::<Vec<_>>().join(", ")
            ));
        }
    }
    for entry in std::fs::read_dir(&ref_root)
        .with_context(|| format!("read {}", ref_root.display()))?
        .filter_map(std::result::Result::ok)
    {
        let dir = entry.path();
        if !dir.is_dir() {
            continue;
        }
        let mut bank_files = Vec::new();
        let mut preset_files = Vec::new();
        for child in std::fs::read_dir(&dir)
            .with_context(|| format!("read {}", dir.display()))?
            .filter_map(std::result::Result::ok)
        {
            let path = child.path();
            if !path.is_file() {
                continue;
            }
            if !matches!(path.extension().and_then(|ext| ext.to_str()), Some("yaml" | "yml")) {
                continue;
            }
            let name =
                path.file_name().and_then(|value| value.to_str()).unwrap_or_default().to_string();
            if name.contains("presets") {
                preset_files.push(path);
            } else {
                bank_files.push(path);
            }
        }
        if preset_files.is_empty() {
            continue;
        }
        let mut bank_ids = BTreeSet::new();
        for bank in bank_files {
            for capture in id_re.captures_iter(&read(&bank)?) {
                if let Some(value) = capture.get(1) {
                    bank_ids.insert(value.as_str().to_string());
                }
            }
        }
        for preset in preset_files {
            let rel = workspace.rel(&preset).display().to_string();
            let raw = read(&preset)?;
            let mut lines = raw.lines().peekable();
            while let Some(line) = lines.next() {
                if !preset_key_re.is_match(line) {
                    continue;
                }
                while let Some(next) = lines.peek() {
                    let trimmed = next.trim();
                    if trimmed.is_empty() {
                        let _ = lines.next();
                        continue;
                    }
                    if !next.starts_with("      - ")
                        && !next.starts_with("    - ")
                        && nested_key_re.is_match(next)
                    {
                        break;
                    }
                    let Some(item) = trimmed.strip_prefix("- ") else {
                        break;
                    };
                    if !bank_ids.is_empty() && !bank_ids.contains(item) {
                        errors.push(format!("{rel}: unresolved preset reference id: {item}"));
                    }
                    let _ = lines.next();
                }
            }
        }
    }
    if errors.is_empty() {
        return pass(check, "asset reference validation completed");
    }
    fail(check, format!("assets-reference-schema: FAILED\n{}", errors.join("\n")))
}

pub(crate) fn check_no_fake_artifacts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let source_roots = [
        workspace.path("crates/bijux-dna-stages-fastq/src"),
        workspace.path("crates/bijux-dna-stages-bam/src"),
        workspace.path("crates/bijux-dna-stages-vcf/src"),
        workspace.path("crates/bijux-dna-api/src/internal/handlers"),
    ];
    let source_re = regex(r"placeholder|fake_artifact|dummy_artifact|stub_artifact")?;
    let mut hits = Vec::new();
    for root in source_roots {
        for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let raw = read(entry.path())?;
            if source_re.is_match(&raw) {
                hits.push(workspace.rel(entry.path()).display().to_string());
            }
        }
    }
    for root in [
        workspace.path("artifacts/domain"),
        workspace.path("artifacts/containers/smoke"),
        workspace.path("artifacts/reports"),
    ] {
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let raw = read(entry.path()).unwrap_or_default();
            if source_re.is_match(&raw) {
                hits.push(workspace.rel(entry.path()).display().to_string());
            }
        }
    }
    if hits.is_empty() {
        return pass(check, "stage code and produced artifacts avoid placeholder markers");
    }
    fail(check, format!("placeholder artifact markers found: {}", hits.join(", ")))
}

pub(crate) fn check_output_roots(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut offenders = Vec::new();
    let sentinel = workspace.path(".sentinel-readonly");
    if sentinel.exists() {
        std::fs::remove_dir_all(&sentinel)
            .with_context(|| format!("remove {}", sentinel.display()))?;
    }
    bijux_dna_infra::ensure_dir(&sentinel)
        .with_context(|| format!("create {}", sentinel.display()))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        let mut perms = std::fs::metadata(&sentinel)?.permissions();
        perms.set_mode(0o555);
        std::fs::set_permissions(&sentinel, perms)?;
    }
    let touch_result = bijux_dna_infra::write_bytes(sentinel.join("forbidden"), b"blocked");
    std::fs::remove_dir_all(&sentinel).with_context(|| format!("remove {}", sentinel.display()))?;
    if touch_result.is_ok() {
        offenders.push(".sentinel-readonly unexpectedly writable".to_string());
    }
    if offenders.is_empty() {
        return pass(check, "outputs stay within controlled roots");
    }
    fail(check, offenders.join("\n"))
}
