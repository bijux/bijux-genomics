use std::collections::BTreeSet;

use anyhow::{Context, Result};
use regex::Regex;

use crate::infrastructure::workspace::Workspace;
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::native::support::{
    fail, load_supported_scripts, make_files, pass, read, run_command, runnable_script_paths,
    shell_script_paths,
};

pub(crate) fn check_supported_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let listed = load_supported_scripts(workspace)?
        .into_iter()
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>();

    let mut missing_files = Vec::new();
    for path in &listed {
        if !workspace.path(path).is_file() {
            missing_files.push(path.clone());
        }
    }

    let referenced = referenced_scripts_from_make(workspace)?;
    let referenced_missing = referenced
        .difference(&listed)
        .cloned()
        .collect::<Vec<_>>();

    let runnable = runnable_script_paths(workspace)?
        .into_iter()
        .map(|path| workspace.rel(&path).to_string_lossy().to_string())
        .collect::<BTreeSet<_>>();
    let unlisted = runnable.difference(&listed).cloned().collect::<Vec<_>>();

    if missing_files.is_empty() && referenced_missing.is_empty() && unlisted.is_empty() {
        return pass(check, "supported scripts catalog is synchronized");
    }

    let mut lines = Vec::new();
    if !missing_files.is_empty() {
        lines.push(format!("listed script files missing: {}", missing_files.join(", ")));
    }
    if !referenced_missing.is_empty() {
        lines.push(format!(
            "scripts referenced by make but not listed: {}",
            referenced_missing.join(", ")
        ));
    }
    if !unlisted.is_empty() {
        lines.push(format!(
            "runnable scripts missing from scripts/SUPPORTED.toml: {}",
            unlisted.join(", ")
        ));
    }
    fail(check, lines.join("\n"))
}

pub(crate) fn check_no_orphan_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let listed = load_supported_scripts(workspace)?
        .into_iter()
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>();
    let runnable = runnable_script_paths(workspace)?
        .into_iter()
        .map(|path| workspace.rel(&path).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    let orphans = runnable
        .into_iter()
        .filter(|path| !listed.contains(path))
        .collect::<Vec<_>>();
    if orphans.is_empty() {
        return pass(check, "all runnable scripts are registered");
    }
    fail(
        check,
        format!(
            "scripts not listed in scripts/SUPPORTED.toml: {}",
            orphans.join(", ")
        ),
    )
}

pub(crate) fn check_script_help(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    for path in runnable_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let script = path.to_string_lossy().to_string();
        let output = run_command(workspace, "timeout", &["8s", &script, "--help"])?;
        let combined = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        if !output.status.success() {
            violations.push(format!("{rel}: --help failed"));
            continue;
        }
        if !combined.contains("Usage:") {
            violations.push(format!("{rel}: --help output missing Usage:"));
        }
    }
    if violations.is_empty() {
        return pass(check, "all scripts respond to --help with Usage output");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_script_interface(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    let pollution = workspace.path("--__bijux_invalid_flag__");

    if pollution.exists() {
        std::fs::remove_file(&pollution)
            .with_context(|| format!("remove stale {}", pollution.display()))?;
    }

    for path in runnable_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let script = path.to_string_lossy().to_string();
        let help_output = run_command(workspace, "timeout", &["8s", &script, "--help"])?;
        let help_text = format!(
            "{}{}",
            String::from_utf8_lossy(&help_output.stdout),
            String::from_utf8_lossy(&help_output.stderr)
        );
        if !help_output.status.success() {
            violations.push(format!("{rel}: --help failed"));
        }
        if !help_text.contains("Usage:") {
            violations.push(format!("{rel}: help output missing Usage:"));
        }
        if help_text.contains("--verbose") {
            let verbose =
                run_command(workspace, "timeout", &["8s", &script, "--verbose", "--help"])?;
            if !verbose.status.success() {
                violations.push(format!("{rel}: --verbose --help failed"));
            }
        }
        if help_text.contains("--dry-run") {
            let dry_run =
                run_command(workspace, "timeout", &["8s", &script, "--dry-run", "--help"])?;
            if !dry_run.status.success() {
                violations.push(format!("{rel}: --dry-run --help failed"));
            }
        }
        let probe = run_command(
            workspace,
            "timeout",
            &["1s", &script, "--__bijux_invalid_flag__"],
        )?;
        if pollution.exists() {
            violations.push(format!(
                "{rel}: invalid flag probe wrote {}",
                workspace.rel(&pollution).display()
            ));
            std::fs::remove_file(&pollution)
                .with_context(|| format!("remove {}", pollution.display()))?;
        }
        if probe.status.code().is_some_and(|code| code == 0) {
            violations.push(format!("{rel}: invalid flag unexpectedly succeeded"));
        }
    }

    if violations.is_empty() {
        return pass(check, "script interface contract is consistent");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_script_arg_style(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let allowlist = [
        "scripts/_lib/common.sh",
        "scripts/run.sh",
        "scripts/smoke/run.sh",
        "scripts/tooling/benchmarks.sh",
        "scripts/tooling/cargo-targets.sh",
        "scripts/containers/make.sh",
        "scripts/assets/make.sh",
        "scripts/docs/make.sh",
        "scripts/domain/make.sh",
        "scripts/examples/make.sh",
        "scripts/hpc/make.sh",
        "scripts/lab/make.sh",
        "scripts/smoke/make.sh",
        "scripts/test/make.sh",
        "scripts/tooling/make.sh",
        "scripts/containers/lint.sh",
        "scripts/containers/smoke-docker-amd64.sh",
        "scripts/test/toy_runs.sh",
        "scripts/tooling/coverage_summary.sh",
    ];
    let positional_re = Regex::new(r#""\$[2-9]"|\$\{[2-9]\}|"\\$@"|"\\$#""#).expect("regex");
    let flag_re = Regex::new(r"--[A-Za-z0-9_-]+").expect("regex");

    let mut violations = Vec::new();
    for path in runnable_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        if allowlist.contains(&rel.as_str()) {
            continue;
        }
        let raw = read(&path)?;
        if positional_re.is_match(&raw) && !flag_re.is_match(&raw) {
            violations.push(format!(
                "{rel}: uses multi-argument positional style without named flags"
            ));
        }
    }
    if violations.is_empty() {
        return pass(check, "scripts use durable flag-oriented argument styles");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_script_deps(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    for path in runnable_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let readme = path
            .parent()
            .map(|dir| dir.join("README.md"))
            .context("script path has no parent")?;
        if !readme.is_file() {
            violations.push(format!("{rel}: missing {}/README.md", readme.parent().unwrap().display()));
            continue;
        }
        let raw = read(&readme)?;
        if !raw.contains("Requires:") {
            violations.push(format!(
                "{}: missing 'Requires:' declaration",
                workspace.rel(&readme).display()
            ));
        }
    }
    if violations.is_empty() {
        return pass(check, "script subtree READMEs declare required tools");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_script_entrypoint(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let direct_script_re =
        Regex::new(r"scripts/[A-Za-z0-9_./-]+\.sh([^A-Za-z0-9_./-]|$)").expect("regex");
    let mut violations = Vec::new();
    for file in make_files(workspace)? {
        let rel = workspace.rel(&file).to_string_lossy().to_string();
        let raw = read(&file)?;
        for line in raw.lines() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') || !trimmed.contains("scripts/") {
                continue;
            }
            if trimmed.contains("scripts/run.sh") {
                continue;
            }
            if let Some(hit) = direct_script_re.find(trimmed) {
                violations.push(format!("{rel}: {}", &trimmed[hit.start()..hit.end()]));
            }
        }
    }
    if violations.is_empty() {
        return pass(check, "make entrypoints use scripts/run.sh");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_shell_portability(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let forbidden = [
        ("function ", "prefer POSIX-style name() declarations"),
        ("readarray", "readarray is not portable"),
        ("mapfile", "mapfile is not portable"),
    ];
    let mut violations = Vec::new();
    for path in shell_script_paths(workspace)? {
        if path.extension().and_then(|ext| ext.to_str()) != Some("sh") {
            continue;
        }
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let raw = read(&path)?;
        for (needle, reason) in forbidden {
            if raw.contains(needle) {
                violations.push(format!("{rel}: {reason}"));
            }
        }
    }
    if violations.is_empty() {
        return pass(check, "shell surface avoids non-portable constructs");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_exit_codes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    for path in runnable_script_paths(workspace)? {
        if path.extension().and_then(|ext| ext.to_str()) != Some("sh") {
            continue;
        }
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let raw = read(&path)?;
        if raw.contains("exit 3")
            || raw.contains("exit 4")
            || raw.contains("exit 5")
            || raw.contains("exit 126")
        {
            violations.push(format!("{rel}: uses non-contract exit code"));
        }
    }
    if violations.is_empty() {
        return pass(check, "scripts keep to the shared exit-code contract");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_ci_shell_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let workflows = read(&workspace.path(".github/workflows/ci.yml")).unwrap_or_default();
    let mut violations = Vec::new();
    for path in runnable_script_paths(workspace)? {
        if path.extension().and_then(|ext| ext.to_str()) != Some("sh") {
            continue;
        }
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        if !workflows.contains(&rel) {
            continue;
        }
        let head = read(&path)?
            .lines()
            .take(16)
            .collect::<Vec<_>>()
            .join("\n");
        if !head.contains("set -euo pipefail") && !head.contains("set -Eeuo pipefail") {
            violations.push(format!("{rel}: missing strict mode header"));
        }
        if !head.contains("LC_ALL=C") {
            violations.push(format!("{rel}: missing LC_ALL=C"));
        }
    }
    if violations.is_empty() {
        return pass(check, "CI shell scripts keep strict mode and locale headers");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_makes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let allowed = load_supported_scripts(workspace)?
        .into_iter()
        .filter(|entry| entry.ci_allowed)
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>();
    let mut violations = Vec::new();
    for file in make_files(workspace)? {
        let rel = workspace.rel(&file).to_string_lossy().to_string();
        let raw = read(&file)?;
        for line in raw.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('#') {
                continue;
            }
            if trimmed.contains("cargo ")
                && !trimmed.contains("scripts/run.sh")
                && !trimmed.contains("./scripts/run.sh")
            {
                violations.push(format!("{rel}: direct cargo usage `{trimmed}`"));
            }
            if let Some(script) = direct_script_path(trimmed) {
                if !allowed.contains(&script) {
                    violations.push(format!(
                        "{rel}: non-ci script referenced from make `{script}`"
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        return pass(check, "make surface delegates through approved CI scripts");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_scripts(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let allowlist = [
        "scripts/containers/registry-tools.sh",
        "scripts/containers/smoke-apptainer.sh",
        "scripts/domain/validate.sh",
        "scripts/lab/run_bench.sh",
        "scripts/lab/run_pipelines.sh",
        "scripts/tooling/bijux.sh",
        "scripts/tooling/generate-configs.sh",
        "scripts/tooling/image-qa.sh",
    ];
    let mut violations = Vec::new();
    for path in shell_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        if allowlist.contains(&rel.as_str()) {
            continue;
        }
        let raw = read(&path)?;
        if raw.contains("cargo ") || raw.contains("docker ") || raw.contains("apptainer ") {
            violations.push(format!("{rel}: direct tool invocation detected"));
        }
    }
    if violations.is_empty() {
        return pass(check, "scripts avoid raw cargo/container invocations");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_no_parallel_accidental(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let scripts = load_supported_scripts(workspace)?;
    let mut seen = BTreeSet::new();
    let mut duplicates = Vec::new();
    let write_re = Regex::new(r"artifacts/[A-Za-z0-9._/-]+").expect("regex");
    for entry in scripts {
        let path = workspace.path(&entry.path);
        if !path.is_file() {
            continue;
        }
        let raw = read(&path)?;
        for capture in write_re.find_iter(&raw) {
            let key = capture.as_str().to_string();
            if !seen.insert(key.clone()) {
                duplicates.push(format!("{}: {key}", entry.path));
            }
        }
    }
    if duplicates.is_empty() {
        return pass(check, "supported scripts do not share accidental artifact roots");
    }
    fail(check, duplicates.join("\n"))
}

pub(crate) fn check_no_temp_leaks(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    for path in shell_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        if rel.ends_with("check-no-temp-leaks.sh")
            || rel.ends_with("check-runtime-execution-kernel-config.sh")
        {
            continue;
        }
        let raw = read(&path)?;
        if raw.contains("mktemp") && !raw.contains("trap ") {
            violations.push(format!("{rel}: mktemp without cleanup trap"));
        }
    }
    if violations.is_empty() {
        return pass(check, "temporary files are cleaned up explicitly");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_network_usage(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let allowlist = [
        "scripts/tooling/acquire-maps.sh",
        "scripts/tooling/acquire-panels.sh",
        "scripts/tooling/acquire-reference.sh",
    ];
    let mut violations = Vec::new();
    for path in shell_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        if allowlist.contains(&rel.as_str()) {
            continue;
        }
        let raw = read(&path)?;
        if raw.contains("curl ") || raw.contains("wget ") || raw.contains("git clone ") {
            violations.push(format!("{rel}: network primitive detected"));
        }
    }
    if violations.is_empty() {
        return pass(check, "network access is isolated to acquisition scripts");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_script_writes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    let absolute_write_re = Regex::new(r"(>|>>|cp |mv |mkdir -p|rm -rf)\s*/(tmp|var|opt|usr|etc|home|Users)\b")
        .expect("regex");
    for entry in load_supported_scripts(workspace)? {
        if entry.outputs.is_empty() {
            violations.push(format!("{}: empty outputs contract", entry.path));
        }
        if !entry.outputs.iter().any(|output| output == "artifacts/") {
            violations.push(format!("{}: outputs must include artifacts/", entry.path));
        }
        if !entry.outputs.iter().any(|output| output == "$ISO_ROOT/") {
            violations.push(format!("{}: outputs must include $ISO_ROOT/", entry.path));
        }
        if let Ok(raw) = read(&workspace.path(&entry.path)) {
            if absolute_write_re.is_match(&raw) {
                violations.push(format!("{}: forbidden absolute write pattern", entry.path));
            }
        }
    }
    if violations.is_empty() {
        return pass(check, "script outputs stay within governed roots");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_lib_api(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let api_doc = read(&workspace.path("scripts/_lib/API.md"))?;
    let exported = Regex::new(r"`([A-Za-z_][A-Za-z0-9_]*)")
        .expect("regex")
        .captures_iter(&api_doc)
        .map(|caps| caps[1].to_string())
        .collect::<BTreeSet<_>>();
    let function_re = Regex::new(r"^([A-Za-z_][A-Za-z0-9_]*)\(\)\s*\{").expect("regex");
    let all_funcs = [
        workspace.path("scripts/_lib/common.sh"),
        workspace.path("scripts/_lib/runtime.sh"),
    ]
    .into_iter()
    .flat_map(|path| {
        read(&path)
            .unwrap_or_default()
            .lines()
            .filter_map(|line| {
                function_re
                    .captures(line)
                    .map(|caps| caps[1].to_string())
            })
            .collect::<Vec<_>>()
    })
    .collect::<BTreeSet<_>>();

    let mut violations = Vec::new();
    for function in &all_funcs {
        if !function.starts_with("_internal_") && !exported.contains(function) {
            violations.push(format!(
                "scripts/_lib function `{function}` is missing from scripts/_lib/API.md"
            ));
        }
    }

    for path in runnable_script_paths(workspace)? {
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let raw = read(&path)?;
        for function in &all_funcs {
            if raw.contains(&format!("{function}()")) {
                violations.push(format!("{rel}: duplicates _lib function `{function}`"));
            }
        }
        if !raw.contains("scripts/_lib/common.sh") {
            violations.push(format!("{rel}: must source scripts/_lib/common.sh"));
        }
        if !raw.contains("require_stable_env") {
            violations.push(format!("{rel}: must call require_stable_env"));
        }
        if !raw.contains("SCRIPT_DIR=$(cd \"$(dirname \"${BASH_SOURCE[0]}\")\" && pwd)") {
            violations.push(format!("{rel}: must define SCRIPT_DIR via BASH_SOURCE"));
        }
        if !raw.contains("ROOT_DIR=$(cd \"${SCRIPT_DIR}/..") {
            violations.push(format!("{rel}: must derive ROOT_DIR from SCRIPT_DIR"));
        }
    }

    if violations.is_empty() {
        return pass(check, "script library API stays documented and centralized");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_tree_intent(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let scripts_root = workspace.path("scripts");
    let mut violations = Vec::new();
    for entry in std::fs::read_dir(&scripts_root).with_context(|| format!("read {}", scripts_root.display()))? {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let readme = entry.path().join("README.md");
        if !readme.is_file() {
            violations.push(format!("missing {}", workspace.rel(&readme).display()));
            continue;
        }
        if !read(&readme)?.contains("Purpose:") {
            violations.push(format!("{}: missing Purpose:", workspace.rel(&readme).display()));
        }
    }
    if violations.is_empty() {
        return pass(check, "every scripts subtree declares its purpose");
    }
    fail(check, violations.join("\n"))
}

fn referenced_scripts_from_make(workspace: &Workspace) -> Result<BTreeSet<String>> {
    let mut referenced = BTreeSet::new();
    let re = Regex::new(r"scripts/[A-Za-z0-9_./-]+\.(?:sh|py)").expect("regex");
    for file in make_files(workspace)? {
        let raw = read(&file)?;
        for capture in re.find_iter(&raw) {
            referenced.insert(capture.as_str().to_string());
        }
    }
    Ok(referenced)
}

fn direct_script_path(line: &str) -> Option<String> {
    Regex::new(r"scripts/[A-Za-z0-9_./-]+\.(?:sh|py)")
        .expect("regex")
        .find(line)
        .map(|hit| hit.as_str().to_string())
}
