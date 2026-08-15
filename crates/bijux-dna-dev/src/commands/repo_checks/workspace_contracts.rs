use std::collections::BTreeSet;
use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_domain_compiler::{compile_domain_configs, CompileOptions, DEFAULT_COMPILE_SCOPE};
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
            ".cargo/config.toml must not set host-specific rustflags in repository policy"
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
        let output = run_native_ops_command(
            NativeOpsCommandKey::ExamplesRun,
            workspace,
            std::slice::from_ref(&id),
        )?;
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
    let generated_root = workspace
        .path(&format!("artifacts/checks/generated-configs/process-{}", std::process::id()));
    if generated_root.exists() {
        std::fs::remove_dir_all(&generated_root)
            .with_context(|| format!("remove stale {}", generated_root.display()))?;
    }
    if let Err(error) = compile_domain_configs(&CompileOptions {
        domain_dir: workspace.path("domain"),
        configs_dir: generated_root.clone(),
        scope: DEFAULT_COMPILE_SCOPE.to_string(),
    }) {
        return fail(
            check,
            format!("domain config generation failed in {}: {error:#}", generated_root.display()),
        );
    }
    let drift = generated_config_drift(&workspace.root, &generated_root, GENERATED_CONFIG_PATHS)?;
    if drift.is_empty() {
        std::fs::remove_dir_all(&generated_root)
            .with_context(|| format!("remove verified {}", generated_root.display()))?;
        return pass(check, "generated configs match committed outputs");
    }
    fail(
        check,
        format!(
            "generated configs drift detected; rendered comparison retained at {}:\n{}",
            generated_root.display(),
            drift.join("\n")
        ),
    )
}

const GENERATED_CONFIG_PATHS: &[&str] = &[
    "configs/ci/registry/tool_registry.toml",
    "configs/ci/registry/tool_registry_experimental.toml",
    "configs/ci/registry/tool_registry_container_experimental.toml",
    "configs/ci/registry/tool_registry_vcf.toml",
    "configs/ci/registry/tool_registry_vcf_downstream.toml",
    "configs/ci/registry/domain_registry_release_bundle.json",
    "configs/ci/registry/domain_defaults_snapshot.json",
    "configs/ci/registry/domain_artifact_contract_snapshots.json",
    "configs/ci/registry/domain_metric_catalogs.json",
    "configs/ci/registry/domain_deprecations_snapshot.json",
    "configs/ci/registry/domain_invariant_catalogs.json",
    "configs/ci/registry/domain_evidence_contracts.json",
    "configs/ci/tools/required_tools.toml",
    "configs/ci/tools/required_tools_vcf_downstream.toml",
    "configs/ci/stages/stages.toml",
    "configs/ci/stages/stages_vcf.toml",
    "configs/ci/tools/images.toml",
];

fn generated_config_drift(
    repository_root: &Path,
    generated_configs_root: &Path,
    paths: &[&str],
) -> Result<Vec<String>> {
    let mut drift = Vec::new();
    for relative in paths {
        let generated_relative = relative.strip_prefix("configs/").with_context(|| {
            format!("generated config path must begin with configs/: {relative}")
        })?;
        let committed_path = repository_root.join(relative);
        let generated_path = generated_configs_root.join(generated_relative);
        let committed = std::fs::read(&committed_path).with_context(|| {
            format!("read committed generated config {}", committed_path.display())
        })?;
        let generated = std::fs::read(&generated_path).with_context(|| {
            format!("read rendered generated config {}", generated_path.display())
        })?;
        if committed != generated {
            drift.push((*relative).to_string());
        }
    }
    Ok(drift)
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
    offenders.extend(embedded_workspace_mirror_roots(&workspace.root)?);
    offenders.extend(product_user_path_literal_offenders(&workspace.root)?);
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
        return pass(
            check,
            "source files avoid host-specific user paths and no embedded workspace mirror exists",
        );
    }
    fail(check, offenders.join("\n"))
}

fn embedded_workspace_mirror_roots(root: &std::path::Path) -> Result<Vec<String>> {
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(root)
        .with_context(|| format!("read workspace root {}", root.display()))?
    {
        let entry = entry?;
        let name = entry.file_name();
        let name = name.to_string_lossy();
        if is_absolute_workspace_prefix(&name) {
            offenders.push(format!("{name}/: embedded absolute-workspace mirror root"));
        }
    }
    offenders.sort();
    Ok(offenders)
}

fn product_user_path_literal_offenders(root: &std::path::Path) -> Result<Vec<String>> {
    let user_path_re = regex(r"/Users/|[A-Za-z]:\\\\Users\\\\")?;
    let text_extensions =
        ["json", "md", "mk", "py", "rs", "sha256", "sh", "toml", "tsv", "txt", "yaml", "yml"];
    let mut offenders = Vec::new();
    for product_root in ["benchmarks", "containers"] {
        let path = root.join(product_root);
        for entry in
            WalkDir::new(&path).follow_links(false).into_iter().filter_map(std::result::Result::ok)
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let extension = entry.path().extension().and_then(std::ffi::OsStr::to_str);
            if !extension.is_some_and(|value| text_extensions.contains(&value)) {
                continue;
            }
            let raw = read(entry.path())?;
            if user_path_re.is_match(&raw) {
                offenders.push(
                    entry.path().strip_prefix(root).unwrap_or(entry.path()).display().to_string(),
                );
            }
        }
    }
    offenders.sort();
    offenders.dedup();
    Ok(offenders)
}

fn is_absolute_workspace_prefix(name: &str) -> bool {
    let windows_drive =
        name.len() == 2 && name.as_bytes()[0].is_ascii_alphabetic() && name.as_bytes()[1] == b':';
    matches!(name, "Users" | "home") || windows_drive
}

const CONTAINER_ROOT_DIRECTORIES: &[&str] = &[
    "apptainer",
    "contracts",
    "docker",
    "docs",
    "licenses",
    "network",
    "reports",
    "runs",
    "tools",
    "versions",
];
const CONTAINER_ROOT_FILES: &[&str] = &["README.md", "TOOL_IDS.txt", "index.md"];
const BENCHMARK_ROOT_DIRECTORIES: &[&str] = &[
    "adapters",
    "configs",
    "contracts",
    "corpora",
    "execution",
    "reports",
    "runs",
    "schemas",
    "tests",
];
const BENCHMARK_ROOT_FILES: &[&str] = &["README.md"];

fn governed_product_root_offenders(
    root: &std::path::Path,
    product_root: &str,
    allowed_directories: &[&str],
    allowed_files: &[&str],
) -> Result<Vec<String>> {
    let path = root.join(product_root);
    if !path.is_dir() {
        return Ok(vec![format!("{product_root}/: missing governed product root")]);
    }

    let mut offenders = Vec::new();
    for entry in
        std::fs::read_dir(&path).with_context(|| format!("read product root {}", path.display()))?
    {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        let file_type = entry.file_type()?;
        if is_absolute_workspace_prefix(&name) {
            offenders
                .push(format!("{product_root}/{name}: embedded absolute-workspace mirror prefix"));
            continue;
        }
        if allowed_directories.contains(&name.as_str()) && file_type.is_dir() {
            continue;
        }
        if allowed_files.contains(&name.as_str()) && file_type.is_file() {
            continue;
        }
        offenders.push(format!("{product_root}/{name}: unexpected product-root entry"));
    }

    for required_directory in allowed_directories {
        if !path.join(required_directory).is_dir() {
            offenders
                .push(format!("{product_root}/{required_directory}/: missing governed directory"));
        }
    }
    for required_file in allowed_files {
        if !path.join(required_file).is_file() {
            offenders.push(format!("{product_root}/{required_file}: missing governed file"));
        }
    }

    for entry in
        WalkDir::new(&path).follow_links(false).into_iter().filter_map(std::result::Result::ok)
    {
        if entry.path() == path || !entry.file_type().is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy();
        if is_absolute_workspace_prefix(&name) {
            offenders.push(format!(
                "{}: embedded absolute-workspace mirror prefix",
                entry.path().strip_prefix(root).unwrap_or(entry.path()).display()
            ));
        }
    }

    offenders.sort();
    offenders.dedup();
    Ok(offenders)
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
    ];
    let mut offenders = Vec::new();
    offenders.extend(embedded_workspace_mirror_roots(&workspace.root)?);
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
            || is_absolute_workspace_prefix(&name)
        {
            continue;
        }
        offenders.push(name);
    }
    offenders.extend(governed_product_root_offenders(
        &workspace.root,
        "containers",
        CONTAINER_ROOT_DIRECTORIES,
        CONTAINER_ROOT_FILES,
    )?);
    offenders.extend(governed_product_root_offenders(
        &workspace.root,
        "benchmarks",
        BENCHMARK_ROOT_DIRECTORIES,
        BENCHMARK_ROOT_FILES,
    )?);
    offenders.sort();
    offenders.dedup();
    if offenders.is_empty() {
        return pass(
            check,
            "workspace and governed product-root layouts stay allowlisted without embedded absolute prefixes",
        );
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

#[cfg(test)]
mod embedded_workspace_mirror_tests {
    use super::{
        embedded_workspace_mirror_roots, governed_product_root_offenders,
        product_user_path_literal_offenders, CONTAINER_ROOT_DIRECTORIES, CONTAINER_ROOT_FILES,
    };

    fn materialize_allowed_root(
        root: &std::path::Path,
        product_root: &str,
        directories: &[&str],
        files: &[&str],
    ) -> anyhow::Result<()> {
        let path = root.join(product_root);
        std::fs::create_dir_all(&path)?;
        for directory in directories {
            std::fs::create_dir(path.join(directory))?;
        }
        for file in files {
            std::fs::write(path.join(file), b"governed\n")?;
        }
        Ok(())
    }

    #[test]
    fn embedded_absolute_roots_are_rejected_without_traversal() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        std::fs::create_dir(root.path().join("Users"))?;
        std::fs::create_dir(root.path().join("home"))?;
        std::fs::create_dir(root.path().join("C:"))?;
        std::fs::create_dir(root.path().join("benchmarks"))?;

        let offenders = embedded_workspace_mirror_roots(root.path())?;

        assert_eq!(
            offenders,
            vec![
                "C:/: embedded absolute-workspace mirror root",
                "Users/: embedded absolute-workspace mirror root",
                "home/: embedded absolute-workspace mirror root",
            ]
        );
        Ok(())
    }

    #[test]
    fn product_manifests_reject_host_specific_absolute_paths() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        std::fs::create_dir_all(root.path().join("benchmarks/tests/fixtures"))?;
        std::fs::create_dir(root.path().join("containers"))?;
        std::fs::write(
            root.path().join("benchmarks/tests/fixtures/CHECKSUMS.sha256"),
            b"0000000000000000000000000000000000000000000000000000000000000000  /Users/example/repository/input.fastq\n",
        )?;
        std::fs::write(root.path().join("containers/catalog.toml"), b"path = \"portable\"\n")?;

        assert_eq!(
            product_user_path_literal_offenders(root.path())?,
            vec!["benchmarks/tests/fixtures/CHECKSUMS.sha256"]
        );
        Ok(())
    }

    #[test]
    fn governed_product_root_rejects_unexpected_entries_and_nested_absolute_prefixes(
    ) -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        materialize_allowed_root(
            root.path(),
            "containers",
            CONTAINER_ROOT_DIRECTORIES,
            CONTAINER_ROOT_FILES,
        )?;
        std::fs::create_dir(root.path().join("containers/cache"))?;
        std::fs::create_dir_all(root.path().join("containers/reports/Users/bijan/workspace"))?;

        let offenders = governed_product_root_offenders(
            root.path(),
            "containers",
            CONTAINER_ROOT_DIRECTORIES,
            CONTAINER_ROOT_FILES,
        )?;

        assert_eq!(
            offenders,
            vec![
                "containers/cache: unexpected product-root entry",
                "containers/reports/Users: embedded absolute-workspace mirror prefix",
            ]
        );
        Ok(())
    }

    #[test]
    fn governed_product_root_accepts_the_documented_tree() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        materialize_allowed_root(
            root.path(),
            "containers",
            CONTAINER_ROOT_DIRECTORIES,
            CONTAINER_ROOT_FILES,
        )?;

        assert!(governed_product_root_offenders(
            root.path(),
            "containers",
            CONTAINER_ROOT_DIRECTORIES,
            CONTAINER_ROOT_FILES,
        )?
        .is_empty());
        Ok(())
    }
}

#[cfg(test)]
mod generated_config_tests {
    use super::generated_config_drift;

    #[test]
    fn comparison_never_requires_source_tree_writes() -> anyhow::Result<()> {
        let repository = tempfile::tempdir()?;
        let generated = tempfile::tempdir()?;
        let paths = ["configs/ci/registry/example.json", "configs/ci/tools/example.toml"];
        for relative in paths {
            let committed_path = repository.path().join(relative);
            let rendered_relative = relative
                .strip_prefix("configs/")
                .ok_or_else(|| anyhow::anyhow!("test config path lacks configs/ prefix"))?;
            let rendered_path = generated.path().join(rendered_relative);
            let committed_parent = committed_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("test committed path lacks parent"))?;
            let rendered_parent = rendered_path
                .parent()
                .ok_or_else(|| anyhow::anyhow!("test rendered path lacks parent"))?;
            std::fs::create_dir_all(committed_parent)?;
            std::fs::create_dir_all(rendered_parent)?;
            std::fs::write(committed_path, b"same\n")?;
            std::fs::write(rendered_path, b"same\n")?;
        }

        assert!(generated_config_drift(repository.path(), generated.path(), &paths)?.is_empty());
        std::fs::write(generated.path().join("ci/tools/example.toml"), b"different\n")?;
        assert_eq!(
            generated_config_drift(repository.path(), generated.path(), &paths)?,
            vec!["configs/ci/tools/example.toml"]
        );
        assert_eq!(
            std::fs::read(repository.path().join("configs/ci/tools/example.toml"))?,
            b"same\n"
        );
        Ok(())
    }
}
