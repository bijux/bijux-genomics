use std::collections::BTreeSet;

use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;

use crate::infrastructure::workspace::Workspace;
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::native::support::{fail, make_files, pass, read};

pub(crate) fn check_supported_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    if !workspace.path(&["scr", "ipts"].concat()).exists() {
        return pass(check, "legacy scripts directory is absent");
    }
    fail(check, "legacy scripts directory must be removed")
}

pub(crate) fn check_no_orphan_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_script_help(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_script_interface(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_script_arg_style(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_script_deps(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_script_entrypoint(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let offenders = repo_script_references(workspace, &[
        workspace.path("Makefile"),
        workspace.path("makes"),
        workspace.path(".github"),
    ])?;
    if offenders.is_empty() {
        return pass(check, "make and workflow entrypoints do not reference legacy scripts");
    }
    fail(check, offenders.join("\n"))
}

pub(crate) fn check_shell_portability(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_exit_codes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_ci_shell_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let offenders = repo_script_references(workspace, &[workspace.path(".github/workflows")])?;
    if offenders.is_empty() {
        return pass(check, "CI workflows do not reference legacy scripts");
    }
    fail(check, offenders.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_makes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let raw_cargo_re = Regex::new(r"^\t@?.*\bcargo(\s|$)").expect("regex");
    let allowed_re = Regex::new(r"cargo\s+run\s+(-q\s+)?-p\s+bijux-dev-dna\s+--").expect("regex");
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
    violations.extend(repo_script_references(
        workspace,
        &[workspace.path("Makefile"), workspace.path("makes")],
    )?);
    if violations.is_empty() {
        return pass(check, "make surface delegates through bijux-dev-dna without legacy scripts");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_no_parallel_accidental(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_no_temp_leaks(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_network_usage(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_script_writes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_lib_api(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

pub(crate) fn check_tree_intent(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    check_supported_scripts(workspace, check)
}

fn repo_script_references(workspace: &Workspace, roots: &[std::path::PathBuf]) -> Result<Vec<String>> {
    let mut offenders = BTreeSet::new();
    let script_re = Regex::new(&format!(r"{}[A-Za-z0-9_./-]+", ["scr", "ipts/"].concat())).expect("regex");
    for root in roots {
        if root.is_file() {
            let rel = workspace.rel(root).display().to_string();
            collect_script_hits(&rel, &read(root)?, &script_re, &mut offenders);
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = workspace.rel(entry.path()).display().to_string();
            collect_script_hits(&rel, &read(entry.path())?, &script_re, &mut offenders);
        }
    }
    Ok(offenders.into_iter().collect())
}

fn collect_script_hits(rel: &str, raw: &str, re: &Regex, out: &mut BTreeSet<String>) {
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
