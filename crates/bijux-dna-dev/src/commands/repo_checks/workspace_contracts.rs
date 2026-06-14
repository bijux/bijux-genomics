use std::collections::BTreeSet;

use anyhow::{Context, Result};
use walkdir::WalkDir;

use crate::commands::command_support::{fail, has_extension, pass, read, regex, run_command};
use crate::commands::run_native_ops_command;
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::model::ops::NativeOpsCommandKey;
use crate::runtime::workspace::Workspace;

pub(crate) fn check_cargo_config_policy(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let config = read(&workspace.path(".cargo/config.toml"))?;
    let mut violations = Vec::new();
    if regex(r"(?m)^\[target\.")?.is_match(&config) {
        violations.push(".cargo/config.toml must not contain [target.*] blocks".to_string());
    }
    if regex(r"(?m)^jobs\s*=")?.is_match(&config) {
        violations.push(".cargo/config.toml must not set machine-specific build.jobs".to_string());
    }
    if regex(r"(?m)^rustflags\s*=")?.is_match(&config) {
        violations.push(
            ".cargo/config.toml must not set rustflags; use configs/runtime/cargo_build.toml"
                .to_string(),
        );
    }
    if !regex(r"(?m)^\[alias\]")?.is_match(&config) {
        violations.push(".cargo/config.toml must keep alias definitions".to_string());
    }
    if violations.is_empty() {
        return pass(check, "cargo config keeps workspace alias contract");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_docs_build_contract(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let cfg_path = workspace.path("configs/docs/mkdocs.toml");
    if !cfg_path.is_file() {
        return fail(check, format!("docs-build-contract: missing {}", cfg_path.display()));
    }
    let cfg: toml::Value = toml::from_str(&read(&cfg_path)?)?;
    let site_dir = cfg.get("site_dir").and_then(toml::Value::as_str).unwrap_or("").trim();

    let mut violations = Vec::new();
    if site_dir != "artifacts/docs/site" {
        violations.push(format!(
            "docs-build-contract: site_dir must be artifacts/docs/site (got {site_dir:?})"
        ));
    }

    let native_ops =
        read(&workspace.path("crates/bijux-dna-dev/src/commands/ops/tooling/diagnostics.rs"))?;
    if !native_ops.contains("XDG_CACHE_HOME") {
        violations.push(
            "docs-build-contract: native docs workflow must set XDG_CACHE_HOME to artifacts/docs/.cache".to_string(),
        );
    }
    if !native_ops.contains("artifacts/docs/.cache") {
        violations.push(
            "docs-build-contract: native docs workflow must use artifacts/docs/.cache".to_string(),
        );
    }

    if !native_ops.contains("PIP_CACHE_DIR") {
        violations.push(
            "docs-build-contract: native docs venv workflow must set PIP_CACHE_DIR".to_string(),
        );
    }
    if !native_ops.contains("artifacts/docs/.cache/pip") {
        violations.push(
            "docs-build-contract: native docs venv workflow must use artifacts/docs/.cache/pip"
                .to_string(),
        );
    }

    if violations.is_empty() {
        return pass(check, "docs build contract stays aligned with docs cache policy");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_config_schema(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let output = run_command(workspace, "python3", &["configs/schema/validate.py", "--root", "."])?;
    if output.status.success() {
        return pass(check, "config schema validation succeeded");
    }
    fail(
        check,
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

pub(crate) fn check_docs_requirements_lock(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let req = workspace.path("configs/docs/requirements.txt");
    let lock = workspace.path("configs/docs/requirements.lock.txt");
    let raw = read(&req)?;
    let lock_raw = read(&lock)?;
    let exact_pin = regex(r"^[A-Za-z0-9_.-]+==[0-9][A-Za-z0-9_.-]*$")?;
    let floating_pin = regex(r"(^|[^=<>!~])[A-Za-z0-9_.-]+\s*(>=|<=|>|<|~=)")?;
    let mut violations = Vec::new();
    let req_lines = raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let lock_lines = lock_raw
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    let mut sorted_req = req_lines.clone();
    let mut sorted_lock = lock_lines.clone();
    sorted_req.sort();
    sorted_lock.sort();

    if sorted_req != sorted_lock {
        violations.push(
            "docs-req-lock: requirements.lock.txt must match requirements.txt exactly (manual pip-compile style lock)".to_string(),
        );
    }
    if req_lines.iter().chain(lock_lines.iter()).any(|line| floating_pin.is_match(line)) {
        violations.push(
            "docs-req-lock: floating/range versions are forbidden; use exact pins package==x.y.z"
                .to_string(),
        );
    }
    if req_lines.iter().any(|line| !exact_pin.is_match(line)) {
        violations.push(
            "docs-req-lock: requirements.txt must contain exact package==version pins only"
                .to_string(),
        );
    }
    if violations.is_empty() {
        return pass(check, "docs requirements are pinned exactly");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_examples_runner_contract(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let example_toml = read(&workspace.path("examples/fastq/qc-pre-bench/example.toml"))?;
    let id = example_toml
        .lines()
        .find_map(|line| {
            let trimmed = line.trim();
            trimmed.strip_prefix("id = ").map(|value| value.trim_matches('"').to_string())
        })
        .context("resolve baseline example id")?;

    for label in ["examples-runner-a", "examples-runner-b"] {
        let output_dir = workspace.path(&format!("artifacts/examples/{id}"));
        if output_dir.exists() {
            std::fs::remove_dir_all(&output_dir)
                .with_context(|| format!("remove {}", output_dir.display()))?;
        }
        let output =
            run_native_ops_command(NativeOpsCommandKey::ExamplesRun, workspace, &[id.clone()])?;
        if !output.is_success() {
            return fail(
                check,
                format!("examples runner failed:\n{}{}", output.stdout, output.stderr),
            );
        }
        let snapshot = workspace.path(&format!("artifacts/examples/{id}.{label}"));
        if snapshot.exists() {
            std::fs::remove_dir_all(&snapshot)
                .with_context(|| format!("remove {}", snapshot.display()))?;
        }
        bijux_dna_infra::rename(&output_dir, &snapshot)
            .with_context(|| format!("move {} -> {}", output_dir.display(), snapshot.display()))?;
    }

    let a_dir = workspace.path(&format!("artifacts/examples/{id}.examples-runner-a"));
    let b_dir = workspace.path(&format!("artifacts/examples/{id}.examples-runner-b"));
    let mut drift = Vec::new();
    for file in ["plan.json", "explain.json", "report.json"] {
        let a = read(&a_dir.join(file))?;
        let b = read(&b_dir.join(file))?;
        if a != b {
            drift.push(file.to_string());
        }
    }
    if drift.is_empty() {
        return pass(check, format!("example runner is deterministic for `{id}`"));
    }
    fail(check, format!("non-deterministic example outputs for `{id}`: {}", drift.join(", ")))
}

pub(crate) fn check_generated_configs(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let compile = run_command(
        workspace,
        "cargo",
        &[
            "run",
            "-p",
            "bijux-dna-domain-compiler",
            "--bin",
            "compile_domain_configs",
            "--",
            "--domain-dir",
            "domain",
            "--configs-dir",
            "configs",
        ],
    )?;
    if !compile.status.success() {
        return fail(
            check,
            format!(
                "domain config generation failed:\n{}{}",
                String::from_utf8_lossy(&compile.stdout),
                String::from_utf8_lossy(&compile.stderr)
            ),
        );
    }
    let diff = run_command(
        workspace,
        "git",
        &[
            "diff",
            "--exit-code",
            "--",
            "configs/ci/registry/tool_registry.toml",
            "configs/ci/registry/tool_registry_experimental.toml",
            "configs/ci/tools/required_tools.toml",
            "configs/ci/stages/stages.toml",
            "configs/ci/tools/images.toml",
        ],
    )?;
    if diff.status.success() {
        return pass(check, "generated configs match committed outputs");
    }
    fail(
        check,
        format!(
            "generated configs drift detected:\n{}{}",
            String::from_utf8_lossy(&diff.stdout),
            String::from_utf8_lossy(&diff.stderr)
        ),
    )
}

pub(crate) fn check_gitignore_contract(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let raw = read(&workspace.path(".gitignore"))?;
    let required = ["/target/", "**/target/", "/artifacts/", "/artifacts/**", "*.tmp"];
    let mut violations = required
        .iter()
        .filter(|pattern| !raw.lines().any(|line| line.trim() == **pattern))
        .map(|pattern| format!("missing required pattern: {pattern}"))
        .collect::<Vec<_>>();
    for line in raw.lines().map(str::trim) {
        if line.starts_with('!')
            && line.contains("target")
            && line != "!/artifacts/isolate/**"
            && line != "!/artifacts/isolate/"
        {
            violations.push(format!("forbidden target unignore pattern: {line}"));
        }
    }
    if violations.is_empty() {
        return pass(check, ".gitignore keeps target and artifacts boundaries intact");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_hidden_tmp_usage(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let forbidden = ["/tmp/", "${TMPDIR:-/tmp}", "std::env::temp_dir()"];
    let roots =
        [workspace.path("crates/bijux-dna-api/src"), workspace.path("crates/bijux-dna-runner/src")];
    let mut violations = Vec::new();
    for root in roots {
        for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            let raw = read(entry.path())?;
            for needle in forbidden {
                if raw.contains(needle) {
                    violations.push(format!(
                        "{}: forbidden tmp usage `{needle}`",
                        workspace.rel(entry.path()).display()
                    ));
                }
            }
        }
    }
    if violations.is_empty() {
        return pass(check, "runtime code avoids hidden tmp roots");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_hpc_safety(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let root = workspace.path(&["scr", "ipts/hpc"].concat());
    if !root.exists() {
        return pass(check, "legacy hpc automation surface is absent");
    }
    let mut violations = Vec::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("sh") {
            continue;
        }
        let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
        let raw = read(entry.path())?;
        if !raw.contains("--dry-run") {
            violations.push(format!("{rel}: missing --dry-run flag handling"));
        }
        if !raw.contains("--confirm") {
            violations.push(format!("{rel}: missing --confirm flag handling"));
        }
        if !raw.contains("dry_run=1") {
            violations.push(format!("{rel}: must default to dry_run=1"));
        }
        if !raw.contains("confirm=0") {
            violations.push(format!("{rel}: must default to confirm=0"));
        }
        if !raw.contains("pass --confirm to execute") {
            violations.push(format!("{rel}: must document confirm requirement in dry-run output"));
        }
    }
    if violations.is_empty() {
        return pass(check, "HPC automation preserves safe dry-run defaults");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_hpc_rsync_docs_parity(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let index_path = workspace.path("configs/hpc/rsync/index.md");
    let rsync_dir = workspace.path("configs/hpc/rsync");
    let mut violations = Vec::new();

    if index_path.is_file() {
        let index = read(&index_path)?;
        if !index.to_lowercase().contains("owner") {
            violations.push(
                "configs/hpc/rsync/index.md must include owner for each pattern file".to_string(),
            );
        }
        for entry in std::fs::read_dir(&rsync_dir)
            .with_context(|| format!("read {}", rsync_dir.display()))?
        {
            let path = entry?.path();
            if !has_extension(&path, "txt") {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .context("rsync pattern filename is not valid utf-8")?;
            if !index.contains(name) {
                violations.push(format!("configs/hpc/rsync/index.md missing reference to {name}"));
            }
        }
    } else {
        violations.push("configs/hpc/rsync/index.md missing".to_string());
    }

    if violations.is_empty() {
        return pass(check, "hpc rsync docs stay aligned with pattern inventory");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_make_help_sync(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let readme = read(&workspace.path("makes/README.md"))?;
    let readme_targets = public_make_targets(&readme);
    let help = run_command(workspace, "make", &["help"])?;
    if !help.status.success() {
        return fail(
            check,
            format!(
                "make help failed:\n{}{}",
                String::from_utf8_lossy(&help.stdout),
                String::from_utf8_lossy(&help.stderr)
            ),
        );
    }
    let help_text = String::from_utf8_lossy(&help.stdout);
    let help_targets = help_text
        .lines()
        .filter_map(|line| {
            if !line.starts_with("  ") {
                return None;
            }
            line.split_whitespace().next().map(ToOwned::to_owned)
        })
        .collect::<BTreeSet<_>>();
    if readme_targets == help_targets {
        return pass(check, "make help output matches documented public targets");
    }
    fail(
        check,
        format!(
            "README targets and make help drifted\nreadme={readme_targets:?}\nhelp={help_targets:?}"
        ),
    )
}

pub(crate) fn check_logging_contract(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let index = read(&workspace.path("configs/logging/index.md"))?;
    let runtime = read(&workspace.path("configs/logging/runtime.toml"))?;
    let mut errors = Vec::new();
    for needle in ["RUST_LOG", "BIJUX_LOG_FORMAT", "trace_id", "run_id"] {
        if !index.contains(needle) {
            errors.push(format!("configs/logging/index.md must document `{needle}`"));
        }
    }
    if !index.contains("runtime.toml") {
        errors.push("configs/logging/index.md must reference runtime.toml".to_string());
    }
    for key in ["default_level", "default_format", "json_fields"] {
        if !regex(&format!(r"(?m)^{}\s*=", regex::escape(key)))?.is_match(&runtime) {
            errors.push(format!("configs/logging/runtime.toml missing key `{key}`"));
        }
    }
    if errors.is_empty() {
        return pass(check, "logging config and docs stay aligned");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_no_target_paths_in_tests(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let target_re = regex(r"(^|[^A-Za-z0-9_./-])target/")?;
    let excluded = [
        "crates/bijux-dna-policies/tests/boundaries/surface/workspace_rules/boundary_enforcement.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/root/root_pollution_policy.rs",
    ];
    let mut offenders = Vec::new();
    for root in [workspace.path("crates"), workspace.path("makes")] {
        for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
            if rel.contains("/__pycache__/") || has_extension(entry.path(), "pyc") {
                continue;
            }
            if rel.starts_with("crates/bijux-dna-dev/src/commands/repo_checks/") {
                continue;
            }
            if !source_like_scan_path(&rel) {
                continue;
            }
            if excluded.contains(&rel.as_str()) {
                continue;
            }
            let raw = read(entry.path())?;
            if target_re.is_match(&raw) {
                offenders.push(rel);
            }
        }
    }
    if offenders.is_empty() {
        return pass(check, "tests and automation do not hardcode target/ paths");
    }
    fail(check, offenders.join("\n"))
}

pub(crate) fn check_no_user_path_literals(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let user_path_re = regex(r"/Users/|[A-Za-z]:\\\\Users\\\\")?;
    let excluded = [
        "crates/bijux-dna-testkit/tests/schemas/snapshot_normalization.rs",
        "crates/bijux-dna-policies/tests/contracts/snapshots/snapshot_hygiene.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/assets/assets_governance_policy.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/governance_core/fixture_privacy_policy.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/governance_examples/examples_golden_hygiene_policy.rs",
        "crates/bijux-dna-policies/tests/contracts/tooling/governance_quality/snapshot_hygiene_policy.rs",
        "crates/bijux-dna/src/commands/benchmark/repo_checks.rs",
        "crates/bijux-dna-dev/src/commands/containers/runtime/mod.rs",
        "crates/bijux-dna-dev/src/commands/containers/validation/compliance.rs",
        "crates/bijux-dna-dev/src/commands/ops/data_support.rs",
        "crates/bijux-dna-dev/src/commands/ops/tooling/acquisition.rs",
    ];
    let mut offenders = Vec::new();
    for root in [workspace.path("crates"), workspace.path("makes")] {
        for entry in WalkDir::new(&root).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
            if rel.contains("/__pycache__/") || has_extension(entry.path(), "pyc") {
                continue;
            }
            if rel.starts_with("crates/bijux-dna-dev/src/commands/repo_checks/") {
                continue;
            }
            if rel.starts_with("examples/") || has_extension(&rel, "md") {
                continue;
            }
            if excluded.contains(&rel.as_str()) {
                continue;
            }
            let raw = read(entry.path())?;
            if user_path_re.is_match(&raw) {
                offenders.push(rel);
            }
        }
    }
    if offenders.is_empty() {
        return pass(check, "source files avoid host-specific user paths");
    }
    fail(check, offenders.join("\n"))
}

pub(crate) fn check_readme_links(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let code_link_re =
        regex(r"`((configs|artifacts|containers|docs|domain|makes|crates)/[^` ]+)`")?;
    let mut missing = Vec::new();
    for rel in repo_readme_paths(workspace) {
        let raw = read(&workspace.path(&rel))?;
        let readme_dir = workspace.path(&rel).parent().map(std::path::Path::to_path_buf);
        for capture in code_link_re.captures_iter(&raw) {
            let target = &capture[1];
            if target.starts_with("artifacts/") || target.contains('*') {
                continue;
            }
            let root_path = workspace.path(target);
            let relative_path = readme_dir.as_ref().map(|dir| dir.join(target));
            if !root_path.exists() && !relative_path.as_ref().is_some_and(|path| path.exists()) {
                missing.push(format!("{rel} -> {}", &capture[1]));
            }
        }
    }
    if missing.is_empty() {
        return pass(check, "README references resolve");
    }
    fail(check, missing.join("\n"))
}

pub(crate) fn check_root_layout(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let allowlist = [
        "artifacts",
        "assets",
        "benchmarks",
        "bin",
        "configs",
        "containers",
        "crates",
        "docs",
        "domain",
        "examples",
        "makes",
        "science",
        "tests",
    ];
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&workspace.root)
        .with_context(|| format!("read {}", workspace.root.display()))?
    {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().to_string();
        if name.starts_with('.')
            || name.starts_with("target")
            || name == "runs"
            || allowlist.contains(&name.as_str())
        {
            continue;
        }
        offenders.push(name);
    }
    if offenders.is_empty() {
        return pass(check, "top-level workspace layout stays allowlisted");
    }
    fail(check, offenders.join(", "))
}

pub(crate) fn check_runtime_execution_kernel_config(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let cfg: toml::Value =
        toml::from_str(&read(&workspace.path("configs/runtime/execution_kernel.toml"))?)?;
    let mut errors = Vec::new();
    for key in [
        "default_threads",
        "default_memory_mb",
        "default_compression_threads",
        "default_timeout_s",
        "max_local_heavy_parallel",
        "bgzip_tabix_max_parallel",
    ] {
        let Some(value) = cfg.get(key) else {
            continue;
        };
        let Some(value) = value.as_integer() else {
            errors.push(format!("{key} must be an integer"));
            continue;
        };
        if value <= 0 {
            errors.push(format!("{key} must be > 0"));
        }
    }
    for key in ["default_temp_root", "cache_root"] {
        let Some(value) = cfg.get(key) else {
            continue;
        };
        let Some(value) = value.as_str() else {
            errors.push(format!("{key} must be a non-empty string"));
            continue;
        };
        if value.trim().is_empty() {
            errors.push(format!("{key} must be a non-empty string"));
            continue;
        }
        if value == "/tmp"
            || value == "/var/tmp"
            || value.starts_with("/tmp/")
            || value.starts_with("/var/tmp/")
        {
            errors.push(format!("{key} cannot point to system tmp ({value})"));
        }
    }
    if let Some(patterns) = cfg.get("heavy_stage_patterns").and_then(toml::Value::as_array) {
        if patterns
            .iter()
            .any(|value| value.as_str().is_none_or(|pattern| pattern.trim().is_empty()))
        {
            errors.push("heavy_stage_patterns must contain non-empty strings".to_string());
        }
    }
    if let Some(per_stage) = cfg.get("per_stage").and_then(toml::Value::as_table) {
        for (pattern, knobs) in per_stage {
            let Some(table) = knobs.as_table() else {
                errors.push(format!("per_stage.{pattern} must be a table"));
                continue;
            };
            for key in ["threads", "memory_mb", "compression_threads", "timeout_s"] {
                if let Some(value) = table.get(key).and_then(toml::Value::as_integer) {
                    if value <= 0 {
                        errors.push(format!("per_stage.{pattern}.{key} must be > 0"));
                    }
                } else if table.contains_key(key) {
                    errors.push(format!("per_stage.{pattern}.{key} must be an integer"));
                }
            }
            if let Some(temp_root) = table.get("temp_root").and_then(toml::Value::as_str) {
                if temp_root.trim().is_empty() {
                    errors
                        .push(format!("per_stage.{pattern}.temp_root must be a non-empty string"));
                } else if temp_root == "/tmp"
                    || temp_root == "/var/tmp"
                    || temp_root.starts_with("/tmp/")
                    || temp_root.starts_with("/var/tmp/")
                {
                    errors.push(format!(
                        "per_stage.{pattern}.temp_root cannot point to system tmp ({temp_root})"
                    ));
                }
            } else if table.contains_key("temp_root") {
                errors.push(format!("per_stage.{pattern}.temp_root must be a non-empty string"));
            }
        }
    }
    if errors.is_empty() {
        return pass(check, "runtime execution kernel config stays valid");
    }
    fail(check, errors.join("\n"))
}

pub(crate) fn check_rustflags_consistency(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let mut violations = Vec::new();
    for entry in
        WalkDir::new(workspace.path("crates")).into_iter().filter_map(std::result::Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        if workspace.rel(entry.path()) != std::path::Path::new("crates")
            && entry.file_name() == "config.toml"
        {
            let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
            if rel.contains("/.cargo/") && read(entry.path())?.contains("RUSTFLAGS") {
                violations.push(format!("{rel}: crate-local RUSTFLAGS not allowed"));
            }
        }
    }
    for path in [workspace.path("makes"), workspace.path("Makefile")] {
        if path.is_file() {
            let raw = read(&path)?;
            if raw.contains("RUSTFLAGS=") {
                violations.push(workspace.rel(&path).display().to_string());
            }
            continue;
        }
        for entry in WalkDir::new(&path).into_iter().filter_map(std::result::Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let rel = workspace.rel(entry.path()).to_string_lossy().to_string();
            if rel.contains("/__pycache__/") || !scan_for_rustflags(&rel) {
                continue;
            }
            let raw = read(entry.path())?;
            if raw.contains("RUSTFLAGS=") && rel != "crates/bijux-dna-dev/src/commands/ops/mod.rs" {
                violations.push(rel);
            }
        }
    }
    if violations.is_empty() {
        return pass(check, "RUSTFLAGS usage stays in the documented coverage path");
    }
    fail(check, violations.join("\n"))
}

pub(crate) fn check_frontend_mini_domain_validation(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let output = run_native_ops_command(
        NativeOpsCommandKey::HpcRunFrontendMiniE2e,
        workspace,
        &["--confirm".to_string()],
    )?;
    if output.is_success() {
        return pass(check, "frontend mini domain validation completed");
    }
    fail(check, format!("{}{}", output.stdout, output.stderr))
}

fn public_make_targets(readme: &str) -> BTreeSet<String> {
    let mut targets = BTreeSet::new();
    let mut in_public = false;
    for line in readme.lines() {
        let trimmed = line.trim();
        if trimmed == "Public targets (stable contract):" {
            in_public = true;
            continue;
        }
        if in_public && line.starts_with("- `") {
            if let Some(target) = line.split('`').nth(1) {
                targets.insert(target.to_string());
            }
            continue;
        }
        if in_public && !trimmed.is_empty() && !line.starts_with("- ") {
            break;
        }
    }
    targets
}

fn repo_readme_paths(workspace: &Workspace) -> Vec<String> {
    let mut paths = Vec::new();
    for entry in WalkDir::new(&workspace.root).into_iter().filter_map(std::result::Result::ok) {
        let rel = workspace.rel(entry.path()).display().to_string();
        if rel.starts_with(".bijux/shared/")
            || rel.starts_with("artifacts/")
            || rel.starts_with("target/")
        {
            continue;
        }
        if entry.file_type().is_file() && entry.file_name() == "README.md" {
            paths.push(workspace.rel(entry.path()).display().to_string());
        }
    }
    paths.sort();
    paths
}

fn scan_for_rustflags(rel: &str) -> bool {
    rel == "Makefile"
        || has_extension(rel, "sh")
        || has_extension(rel, "mk")
        || has_extension(rel, "toml")
        || has_extension(rel, "py")
}

fn source_like_scan_path(rel: &str) -> bool {
    rel == "Makefile"
        || has_extension(rel, "rs")
        || has_extension(rel, "sh")
        || has_extension(rel, "py")
        || has_extension(rel, "mk")
        || has_extension(rel, "toml")
        || has_extension(rel, "yaml")
        || has_extension(rel, "yml")
}
