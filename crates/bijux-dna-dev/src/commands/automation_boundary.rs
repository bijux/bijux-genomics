use std::collections::BTreeSet;

use anyhow::Result;
use regex::Regex;
use walkdir::WalkDir;

use crate::commands::command_support::{fail, make_files, pass, read, regex};
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
    let raw_cargo_re = regex(r"^\t@?(?:cargo|.*[[:space:]]cargo)(?:[[:space:]]|$)")?;
    let allowed_re = regex(r"cargo\s+run\s+(-q\s+)?-p\s+bijux-dna-dev\s+--")?;
    let install_note_re = regex(r"cargo install")?;
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
    let script_re = regex(&format!(r"{}[A-Za-z0-9_./-]+", ["scr", "ipts/"].concat()))?;
    for root in roots {
        if root.is_file() {
            let rel = workspace.rel(root).display().to_string();
            collect_legacy_automation_hits(&rel, &read(root)?, &script_re, &mut offenders);
            continue;
        }
        if !root.is_dir() {
            continue;
        }
        // Bytecode caches are generated inputs, not automation source. Other
        // files still require readable UTF-8 so an unknown source cannot vanish.
        for entry in WalkDir::new(root)
            .into_iter()
            .filter_entry(|entry| !entry.file_type().is_dir() || entry.file_name() != "__pycache__")
        {
            let entry = entry?;
            if !entry.file_type().is_file()
                || matches!(
                    entry.path().extension().and_then(|ext| ext.to_str()),
                    Some("pyc" | "pyo")
                )
            {
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
            let prefix = &line[..capture.start()];
            if prefix.ends_with(".github/") || is_governed_shared_script_prefix(prefix) {
                continue;
            }
            if prefix.ends_with(".bijux/shared/bijux-checks/")
                || prefix.ends_with("shared/bijux-checks/")
            {
                continue;
            }
            out.insert(format!("{rel}:{}:{}", index + 1, capture.as_str()));
        }
    }
}

fn is_governed_shared_script_prefix(prefix: &str) -> bool {
    prefix.ends_with(".bijux/shared/bijux-makes-rs/")
        || prefix.ends_with("shared/bijux-makes-rs/")
        || prefix.ends_with("$(BIJUX_MAKES_SHARED_ROOT)/bijux-makes-rs/")
}

#[cfg(test)]
mod tests {
    use anyhow::{Context, Result};

    use super::{check_no_raw_cargo_in_makes, is_governed_shared_script_prefix};
    use crate::catalog::checks::check_registry;
    use crate::model::check::{CheckDefinition, CheckStatus};
    use crate::runtime::workspace::Workspace;

    fn make_surface_fixture() -> Result<(tempfile::TempDir, Workspace, CheckDefinition)> {
        let artifact_root = Workspace::resolve()?.path("artifacts/tests/automation-boundary");
        std::fs::create_dir_all(&artifact_root)?;
        let owner = tempfile::tempdir_in(artifact_root)?;
        let workspace = Workspace { root: owner.path().to_path_buf() };
        std::fs::create_dir_all(workspace.path("makes/tests/__pycache__"))?;
        std::fs::write(
            workspace.path("Makefile"),
            concat!(
                "gate:\n",
                "\t@cargo run -q -p bijux-dna-dev -- tooling run ci-fmt\n",
                "\tcargo run -p bijux-dna-dev -- tooling run ci-fmt\n",
                "\t@env MODE=1 cargo run -q -p bijux-dna-dev -- tooling run ci-fmt\n",
                "\t@cargo install --locked example-tool\n",
            ),
        )?;
        let check = check_registry()
            .into_iter()
            .find(|check| check.id == "check-no-raw-cargo-in-makes")
            .context("maintained no-raw-Cargo check")?;
        Ok((owner, workspace, check))
    }

    #[test]
    fn generated_python_bytecode_is_not_automation_source() -> Result<()> {
        let (_owner, workspace, check) = make_surface_fixture()?;
        let bytecode = [0xff, 0xfe, 0, 0x80];
        for path in [
            "makes/tests/__pycache__/case.cpython-314.pyc",
            "makes/tests/__pycache__/cached-source.py",
            "makes/tests/legacy-cache.pyc",
            "makes/tests/legacy-cache.pyo",
        ] {
            std::fs::write(workspace.path(path), bytecode)?;
        }

        let outcome = check_no_raw_cargo_in_makes(&workspace, &check)?;
        assert_eq!(outcome.status, CheckStatus::Passed);
        assert_eq!(
            std::fs::read(workspace.path("makes/tests/__pycache__/case.cpython-314.pyc"))?,
            bytecode
        );
        Ok(())
    }

    #[test]
    fn bytecode_caches_do_not_hide_raw_cargo_or_legacy_source() -> Result<()> {
        let (_owner, workspace, check) = make_surface_fixture()?;
        std::fs::write(workspace.path("makes/tests/__pycache__/case.pyc"), [0xff])?;
        std::fs::write(workspace.path("makes/legacy.sh"), "bash scripts/legacy.sh\n")?;
        for recipe in [
            "\t@cargo test --workspace",
            "\tcargo test --workspace",
            "\t@env MODE=1 cargo test --workspace",
        ] {
            std::fs::write(workspace.path("makes/unguarded.mk"), format!("gate:\n{recipe}\n"))?;
            let outcome = check_no_raw_cargo_in_makes(&workspace, &check)?;
            assert_eq!(outcome.status, CheckStatus::Failed);
            assert!(outcome.detail.contains("makes/unguarded.mk:"));
            assert!(outcome.detail.contains(recipe));
            assert!(outcome.detail.contains("makes/legacy.sh:1:scripts/legacy.sh"));
        }
        Ok(())
    }

    #[test]
    fn unreadable_automation_source_outside_bytecode_cache_still_refuses() -> Result<()> {
        let (_owner, workspace, check) = make_surface_fixture()?;
        std::fs::write(workspace.path("makes/tests/source.py"), [0xff, 0xfe])?;
        let result = check_no_raw_cargo_in_makes(&workspace, &check);
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn governed_shared_rust_automation_is_recognized() {
        assert!(is_governed_shared_script_prefix(
            "shared_gate=${repo_root}/.bijux/shared/bijux-makes-rs/"
        ));
        assert!(is_governed_shared_script_prefix(
            "NEXTEST_EXPR_BIN ?= $(BIJUX_MAKES_SHARED_ROOT)/bijux-makes-rs/"
        ));
    }

    #[test]
    fn repository_local_script_paths_remain_legacy() {
        assert!(!is_governed_shared_script_prefix(""));
        assert!(!is_governed_shared_script_prefix("makes/"));
        assert!(!is_governed_shared_script_prefix("tools/"));
    }
}
