#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn script_files(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in WalkDir::new(root.join("scripts"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        let ext = path.extension().and_then(|s| s.to_str());
        if matches!(ext, Some("sh") | Some("py")) {
            out.push(path.to_path_buf());
        }
    }
    out.sort();
    out
}

#[test]
fn policy__contracts__scripts_layout_policy__scripts_live_in_allowed_tree() {
    let root = workspace_root();
    let allowed_prefixes = [
        "scripts/checks/",
        "scripts/containers/",
        "scripts/docs/",
        "scripts/domain/",
        "scripts/hpc/lunarc/",
        "scripts/lab/",
        "scripts/smoke/",
        "scripts/test/",
        "scripts/tooling/",
        "scripts/_lib/",
        "scripts/experimental/",
    ];

    let mut offenders = Vec::new();
    for file in script_files(&root) {
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let ok_prefix = allowed_prefixes.iter().any(|p| rel.starts_with(p));
        if !ok_prefix {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "scripts must live under the approved tree (or approved top-level files):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__scripts_have_strict_mode_and_c_locale() {
    let root = workspace_root();
    let mut strict_offenders = Vec::new();
    let mut locale_offenders = Vec::new();

    for file in script_files(&root) {
        if file.extension().and_then(|s| s.to_str()) != Some("sh") {
            continue;
        }
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        let head = std::fs::read_to_string(&file)
            .unwrap_or_default()
            .lines()
            .take(16)
            .collect::<Vec<_>>()
            .join("\n");
        if !(head.contains("set -euo pipefail") || head.contains("set -eu")) {
            strict_offenders.push(rel.clone());
        }
        if !head.contains("LC_ALL=C") {
            locale_offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        strict_offenders.is_empty(),
        "shell scripts must enable strict mode near the top:\n{}",
        strict_offenders.join("\n")
    );
    bijux_dna_policies::policy_assert!(
        locale_offenders.is_empty(),
        "shell scripts must set LC_ALL=C near the top:\n{}",
        locale_offenders.join("\n")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__supported_scripts_are_make_referenced_or_experimental()
{
    let root = workspace_root();
    let make_text = std::fs::read_to_string(root.join("Makefile")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/cargo.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/containers.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/docs.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/lab.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/lunarc.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/toy.mk")).unwrap_or_default();

    let re = Regex::new(r"scripts/[A-Za-z0-9_./-]+\\.(sh|py)").expect("regex");
    let mut supported = std::collections::BTreeSet::new();
    for m in re.find_iter(&make_text) {
        supported.insert(m.as_str().to_string());
    }

    let mut offenders = Vec::new();
    for file in script_files(&root) {
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if rel.starts_with("scripts/experimental/") || rel.starts_with("scripts/_lib/") {
            continue;
        }
        if !supported.contains(&rel) {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "scripts not referenced by Make must live under scripts/experimental/:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__ci_does_not_call_lab_scripts() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join(".github/workflows"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("yml") {
            continue;
        }
        let raw = std::fs::read_to_string(entry.path()).unwrap_or_default();
        if raw.contains("scripts/lab/") {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CI workflows must not invoke scripts/lab/* directly: {}",
        offenders.join(", ")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__arg_parsing_reuses_shared_lib() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for file in script_files(&root) {
        if file.extension().and_then(|s| s.to_str()) != Some("sh") {
            continue;
        }
        let rel = file
            .strip_prefix(&root)
            .unwrap()
            .to_string_lossy()
            .to_string();
        if rel.starts_with("scripts/_lib/") || rel.starts_with("scripts/experimental/") {
            continue;
        }
        let raw = std::fs::read_to_string(&file).unwrap_or_default();
        let does_manual_arg_parse = raw.contains("while [ \"$#\" -gt 0 ]")
            || raw.contains("case \"$1\" in")
            || raw.contains("getopts");
        let uses_shared_lib = raw.contains("scripts/_lib/common.sh")
            || raw.contains("/_lib/common.sh")
            || raw.contains("source \"$SCRIPT_DIR/../_lib/common.sh\"")
            || raw.contains(". \"$SCRIPT_DIR/../_lib/common.sh\"");
        if does_manual_arg_parse && !uses_shared_lib {
            offenders.push(rel);
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "manual arg parsing must use scripts/_lib/common.sh helpers:\\n{}",
        offenders.join("\\n")
    );
}

#[test]
fn policy__contracts__scripts_layout_policy__ci_scripts_write_under_artifacts_or_iso_root() {
    let root = workspace_root();
    let make_text = std::fs::read_to_string(root.join("Makefile")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/cargo.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/containers.mk")).unwrap_or_default()
        + &std::fs::read_to_string(root.join("makefiles/docs.mk")).unwrap_or_default();
    let re = Regex::new(r"scripts/[A-Za-z0-9_./-]+\.sh").expect("regex");
    let mut ci_scripts = std::collections::BTreeSet::new();
    for m in re.find_iter(&make_text) {
        let rel = m.as_str().to_string();
        if rel.starts_with("scripts/lab/") || rel.starts_with("scripts/hpc/") {
            continue;
        }
        ci_scripts.insert(rel);
    }

    let mut offenders = Vec::new();
    let bad_write = Regex::new(r#"(?m)^\s*(mkdir\s+-p|>\s*|>>\s*|cp\s+|mv\s+).*$"#).expect("regex");
    for rel in ci_scripts {
        let raw = std::fs::read_to_string(root.join(&rel)).unwrap_or_default();
        for line in raw.lines() {
            if !bad_write.is_match(line) {
                continue;
            }
            let mentions_allowed = line.contains("artifacts/")
                || line.contains("$ISO_ROOT")
                || line.contains("${ISO_ROOT")
                || line.contains("ARTIFACT_DIR")
                || line.contains("DOCS_ROOT")
                || line.contains("COVERAGE_BASELINE");
            if !mentions_allowed {
                offenders.push(format!("{rel}: {line}"));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "CI scripts must write outputs under artifacts/ or $ISO_ROOT:\\n{}",
        offenders.join("\\n")
    );
}
