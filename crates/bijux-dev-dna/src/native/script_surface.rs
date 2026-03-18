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
    let referenced_missing = referenced.difference(&listed).cloned().collect::<Vec<_>>();

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
        lines.push(format!(
            "listed script files missing: {}",
            missing_files.join(", ")
        ));
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
    let tmp_root = workspace.path("artifacts/tmp");
    std::fs::create_dir_all(&tmp_root).with_context(|| format!("create {}", tmp_root.display()))?;
    let pollution_dir = workspace.path("artifacts/containers/smoke/pollution");
    let pollution = workspace.path("--__bijux_invalid_flag__");
    let timestamp = chrono::Utc::now().format("%Y%m%dT%H%M%SZ").to_string();

    capture_root_pollution(workspace, &pollution, &pollution_dir, &timestamp)?;

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
            let verbose = run_command(
                workspace,
                "timeout",
                &["8s", &script, "--verbose", "--help"],
            )?;
            if !verbose.status.success() {
                violations.push(format!("{rel}: --verbose --help failed"));
            }
        }
        if help_text.contains("--dry-run") {
            let dry_run = run_command(
                workspace,
                "timeout",
                &["8s", &script, "--dry-run", "--help"],
            )?;
            if !dry_run.status.success() {
                violations.push(format!("{rel}: --dry-run --help failed"));
            }
        }
        let probe_dir = tmp_root.join(format!("script-interface-probe-{}", std::process::id()));
        if probe_dir.exists() {
            std::fs::remove_dir_all(&probe_dir)
                .with_context(|| format!("remove {}", probe_dir.display()))?;
        }
        std::fs::create_dir_all(&probe_dir)
            .with_context(|| format!("create {}", probe_dir.display()))?;
        std::process::Command::new("timeout")
            .args(["1s", &script, "--__bijux_invalid_flag__"])
            .current_dir(&probe_dir)
            .status()
            .with_context(|| format!("run invalid-flag probe for {rel}"))?;
        std::fs::remove_dir_all(&probe_dir)
            .with_context(|| format!("remove {}", probe_dir.display()))?;
        if capture_root_pollution(workspace, &pollution, &pollution_dir, &timestamp)? {
            violations.push(format!(
                "{rel}: invalid-flag probe wrote --__bijux_invalid_flag__ at repo root"
            ));
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
            violations.push(format!(
                "{rel}: missing {}/README.md",
                readme.parent().unwrap().display()
            ));
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
    let mut violations = Vec::new();
    let sed_in_place = Regex::new(r"\bsed\s+-i(\s|$)").expect("regex");
    let readlink_f = Regex::new(r"\breadlink -f\b").expect("regex");
    for entry in load_supported_scripts(workspace)? {
        let path = workspace.path(&entry.path);
        if path.extension().and_then(|ext| ext.to_str()) != Some("sh") || !path.is_file() {
            continue;
        }
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        let raw = read(&path)?;
        let first = raw.lines().next().unwrap_or_default();
        if first == "#!/bin/sh" {
            violations.push(format!("{rel}: uses #!/bin/sh"));
        }
        if sed_in_place.is_match(&raw) && !raw.contains("compat_sed_inplace") {
            violations.push(format!("{rel}: uses sed -i without compatibility wrapper"));
        }
        if readlink_f.is_match(&raw) && !raw.contains("compat_readlink_f") {
            violations.push(format!(
                "{rel}: uses readlink -f without compatibility wrapper"
            ));
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
        let head = read(&path)?.lines().take(16).collect::<Vec<_>>().join("\n");
        if !head.contains("set -euo pipefail") && !head.contains("set -Eeuo pipefail") {
            violations.push(format!("{rel}: missing strict mode header"));
        }
        if !head.contains("LC_ALL=C") {
            violations.push(format!("{rel}: missing LC_ALL=C"));
        }
    }
    if violations.is_empty() {
        return pass(
            check,
            "CI shell scripts keep strict mode and locale headers",
        );
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_no_raw_cargo_in_makes(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let ci_allowed = load_supported_scripts(workspace)?
        .into_iter()
        .filter(|entry| entry.ci_allowed)
        .map(|entry| entry.path)
        .collect::<BTreeSet<_>>();
    let raw_cargo_re = Regex::new(r"^\t@?.*\bcargo(\s|$)").expect("regex");
    let tool_re = Regex::new(
        r"(^|[^[:alnum:]_])(rustup|pip)([[:space:]]|$)|python[0-9.]*[[:space:]]+-m[[:space:]]+venv\b",
    )
    .expect("regex");
    let direct_script_re = Regex::new(r"^\t@?.*scripts/[A-Za-z0-9_./-]+\.sh").expect("regex");
    let run_script_re =
        Regex::new(r"scripts/run\.sh[[:space:]]+([a-z_]+)[[:space:]]+([A-Za-z0-9_./-]+)")
            .expect("regex");
    let mut violations = Vec::new();
    for file in make_files(workspace)? {
        let rel = workspace.rel(&file).to_string_lossy().to_string();
        let raw = read(&file)?;
        for line in raw.lines() {
            if raw_cargo_re.is_match(line)
                && !line.contains("install once: cargo install")
                && !line.contains("policy-no-raw-cargo")
            {
                violations.push(format!("{rel}:{line}"));
            }
            if tool_re.is_match(line) && !line.contains("scripts/tooling/") {
                violations.push(format!("{rel}:{line}"));
            }
            if direct_script_re.is_match(line) && !line.contains("scripts/run.sh") {
                violations.push(format!("{rel}:{line}"));
            }
            for script in script_paths_in_line(line) {
                if script != "scripts/run.sh" && !ci_allowed.contains(&script) {
                    violations.push(format!("{rel}:{line} -> {script} not ci_allowed=true"));
                }
            }
            if let Some(caps) = run_script_re.captures(line) {
                if &caps[1] == "checks" {
                    continue;
                }
                let script = format!("scripts/{}/make.sh", &caps[1]);
                if !ci_allowed.contains(&script) {
                    violations.push(format!("{rel}:{line} -> {script} not ci_allowed=true"));
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
        "scripts/run.sh",
        "scripts/tooling/bijux.sh",
        "scripts/tooling/generate-configs.sh",
        "scripts/tooling/image-qa.sh",
    ];
    let cargo_re = Regex::new(
        r"cargo (fmt|clippy|test|run|deny|nextest|llvm-cov|insta|build|check|doc|install)\b",
    )
    .expect("regex");
    let container_re =
        Regex::new(r"(^|[[:space:];|&])(docker|apptainer)([[:space:]]|$)").expect("regex");
    let mut violations = Vec::new();
    for entry in load_supported_scripts(workspace)? {
        let rel = entry.path;
        if allowlist.contains(&rel.as_str()) {
            continue;
        }
        if rel.starts_with("scripts/tooling/ci-") || rel == "scripts/tooling/cargo-targets.sh" {
            continue;
        }
        if !rel.ends_with(".sh") {
            continue;
        }
        let path = workspace.path(&rel);
        if !path.is_file() {
            continue;
        }
        let raw = read(&path)?;
        for (index, line) in raw.lines().enumerate() {
            if cargo_re.is_match(line)
                && !line.contains("require_artifact_env")
                && !line.contains("Regenerate with: cargo run")
                && !line.contains("ARTIFACT_ROOT=artifacts cargo nextest run")
                && !line.contains("cargo nextest must use")
                && !line.contains("cargo install|policy-no-raw-cargo")
                && !line.contains("cargo llvm-cov nextest must use")
            {
                violations.push(format!("{rel}:{}:{line}", index + 1));
            }
            if !rel.starts_with("scripts/containers/") && container_re.is_match(line) {
                violations.push(format!("{rel}:{}:{line}", index + 1));
            }
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
    let write_re = Regex::new(r"artifacts/[A-Za-z0-9._/-]+").expect("regex");
    let variable_re =
        Regex::new(r"\$\{?(ISO_ROOT|RUN_ID|TAG|TIMESTAMP|UUID|RANDOM|tmp|TMP)").expect("regex");
    let mut violations = Vec::new();
    for entry in load_supported_scripts(workspace)? {
        let path = workspace.path(&entry.path);
        if !path.is_file() {
            continue;
        }
        let raw = read(&path)?;
        for artifact_path in write_re
            .find_iter(&raw)
            .map(|capture| capture.as_str().to_string())
            .collect::<BTreeSet<_>>()
        {
            if [
                "artifacts/tmp",
                "artifacts/inventory",
                "artifacts/test-logs",
                "artifacts/coverage",
                "artifacts/docs",
                "artifacts/policies",
                "artifacts/assets-refresh",
                "artifacts/containers",
                "artifacts/examples",
                "artifacts/domain",
                "artifacts/vcf",
                "artifacts/reference_store",
                "artifacts/test",
                "artifacts/smoke_bam",
                "artifacts/smoke_fastq",
                "artifacts/configs",
            ]
            .iter()
            .any(|prefix| {
                artifact_path == *prefix || artifact_path.starts_with(&format!("{prefix}/"))
            }) {
                continue;
            }
            for line in raw.lines().filter(|line| line.contains(&artifact_path)) {
                if !variable_re.is_match(line) && looks_like_write_line(line) {
                    violations.push(format!(
                        "{} -> fixed artifact path may collide in parallel runs: {}",
                        entry.path, artifact_path
                    ));
                    break;
                }
            }
        }
    }
    if violations.is_empty() {
        return pass(
            check,
            "supported scripts do not share accidental artifact roots",
        );
    }
    fail(check, violations.join("\n"))
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
    let network_re = Regex::new(r"\bcurl\b|\bwget\b|git[[:space:]]+clone\b").expect("regex");
    let mut violations = Vec::new();
    let listed = load_supported_scripts(workspace)?;
    let listed_paths = listed
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<BTreeSet<_>>();
    for entry in listed {
        let rel = entry.path;
        let path = workspace.path(&rel);
        if !path.is_file() {
            continue;
        }
        let raw = read(&path)?;
        if network_re.is_match(&raw) && !entry.network_allowed {
            violations.push(format!(
                "{rel} uses network command(s) but network_allowed != true"
            ));
        }
    }
    for path in shell_script_paths(workspace)? {
        if path.extension().and_then(|ext| ext.to_str()) != Some("sh") {
            continue;
        }
        let rel = workspace.rel(&path).to_string_lossy().to_string();
        if rel.starts_with("scripts/experimental/") || listed_paths.contains(&rel) {
            continue;
        }
        let raw = read(&path)?;
        if network_re.is_match(&raw) {
            violations.push(format!(
                "{rel} uses network command(s) but is not declared in scripts/SUPPORTED.toml with network_allowed=true"
            ));
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
    let absolute_write_re =
        Regex::new(r"(>|>>|cp |mv |mkdir -p|rm -rf)\s*/(tmp|var|opt|usr|etc|home|Users)\b")
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
            .filter_map(|line| function_re.captures(line).map(|caps| caps[1].to_string()))
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
    for entry in std::fs::read_dir(&scripts_root)
        .with_context(|| format!("read {}", scripts_root.display()))?
    {
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
            violations.push(format!(
                "{}: missing Purpose:",
                workspace.rel(&readme).display()
            ));
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

fn capture_root_pollution(
    workspace: &Workspace,
    pollution: &std::path::Path,
    pollution_dir: &std::path::Path,
    timestamp: &str,
) -> Result<bool> {
    if !pollution.is_file() {
        return Ok(false);
    }
    std::fs::create_dir_all(pollution_dir)
        .with_context(|| format!("create {}", pollution_dir.display()))?;
    let target = pollution_dir.join(format!(
        "--__bijux_invalid_flag__.check-script-interface.{}.{}",
        timestamp,
        std::process::id()
    ));
    std::fs::rename(pollution, &target).with_context(|| {
        format!(
            "move {} -> {}",
            pollution.display(),
            workspace.rel(&target).display()
        )
    })?;
    Ok(true)
}

fn script_paths_in_line(line: &str) -> Vec<String> {
    Regex::new(r"scripts/[A-Za-z0-9_./-]+\.(?:sh|py)")
        .expect("regex")
        .find_iter(line)
        .map(|hit| hit.as_str().to_string())
        .collect()
}

fn looks_like_write_line(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.contains('>')
        || trimmed.contains("cp ")
        || trimmed.contains("mv ")
        || trimmed.contains("mkdir ")
        || trimmed.contains("tar ")
        || trimmed.contains("write_json_sorted_file ")
        || trimmed.contains("cat >")
}
