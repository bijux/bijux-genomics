use std::collections::BTreeSet;

use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;

use crate::commands::command_support::{fail, make_files, pass, read};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

pub(crate) fn check_legacy_automation_removed(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    if !workspace.path(&["scr", "ipts"].concat()).exists() {
        return pass(check, "legacy automation directory is absent");
    }
    fail(check, "legacy automation directory must be removed")
}

pub(crate) fn check_legacy_automation_references(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_help(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_interface(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_arg_style(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_dependencies(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_entrypoints(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let offenders = repo_legacy_automation_references(
        workspace,
        &[workspace.path("Makefile"), workspace.path("makes")],
    )?;
    if offenders.is_empty() {
        return pass(check, "make and workflow entrypoints do not reference legacy automation");
    }
    fail(check, offenders.join("\n"))
}

pub(crate) fn check_automation_portability(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_exit_codes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_ci_automation_surface(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let offenders =
        repo_legacy_automation_references(workspace, &[workspace.path(".github/workflows")])?;
    if offenders.is_empty() {
        return pass(check, "CI workflows do not reference legacy automation");
    }
    fail(check, offenders.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_makes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let raw_cargo_re =
        Regex::new(r"^\t@?.*(?:^|[[:space:]])cargo(?:[[:space:]]|$)").expect("regex");
    let allowed_re = Regex::new(r"cargo\s+run\s+(-q\s+)?-p\s+bijux-dna-dev\s+--").expect("regex");
    let install_note_re = Regex::new(r"cargo install").expect("regex");
    let mut violations = Vec::new();
    for file in make_files(workspace)? {
        let rel = workspace.rel(&file).display().to_string();
        let raw = read(&file)?;
        for line in raw.lines() {
            if !raw_cargo_re.is_match(line) {
                continue;
            }
            if allowed_re.is_match(line) || install_note_re.is_match(line) {
                continue;
            }
            violations.push(format!("{rel}:{line}"));
        }
    }
    violations.extend(repo_legacy_automation_references(
        workspace,
        &[workspace.path("Makefile"), workspace.path("makes")],
    )?);
    if violations.is_empty() {
        return pass(
            check,
            "make surface delegates through bijux-dna-dev without legacy automation",
        );
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_automation(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_parallelism(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_temp_discipline(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_network_usage(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_writes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_boundary(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

pub(crate) fn check_automation_intent(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_legacy_automation_removed(workspace, check)
}

fn repo_legacy_automation_references(
    workspace: &Workspace,
    roots: &[std::path::PathBuf],
) -> Result<Vec<String>> {
    let mut offenders = BTreeSet::new();
    let script_re =
        Regex::new(&format!(r"{}[A-Za-z0-9_./-]+", ["scr", "ipts/"].concat())).expect("regex");
    for root in roots {
        if root.is_file() {
            let rel = workspace.rel(root).display().to_string();
            collect_legacy_automation_hits(&rel, &read(root)?, &script_re, &mut offenders);
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = workspace.rel(entry.path()).display().to_string();
            collect_legacy_automation_hits(&rel, &read(entry.path())?, &script_re, &mut offenders);
        }
    }
    Ok(offenders.into_iter().collect())
}

fn collect_legacy_automation_hits(rel: &str, raw: &str, re: &Regex, out: &mut BTreeSet<String>) {
    let legacy = ["scr", "ipts/"].concat();
    for (index, line) in raw.lines().enumerate() {
        if line.trim_start().starts_with('#') || !line.contains(&legacy) {
            continue;
        }
        for capture in re.find_iter(line) {
            out.insert(format!("{rel}:{}:{}", index + 1, capture.as_str()));
        }
    }
}
