#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_core::id_catalog;
use chrono::Utc;
use regex::Regex;
use serde::Deserialize;
use serde_json::{json, Value};
use toml::Value as TomlValue;
use walkdir::WalkDir;

mod artifact_support;
mod assets;
mod data_support;
mod dispatch;
mod docs;
mod examples;
mod execution_support;
mod hpc;
mod lab;
mod smoke;
mod tooling;
mod toy_support;
mod verification;

use self::artifact_support::{
    artifact_env, artifact_env_with_common_test_env, artifact_root_path,
    ensure_artifact_root_inside_artifacts, materialize_controlled_file, path_from_arg,
    resolve_workspace_path, run_make_target, sha256_hex_bytes,
};
use self::data_support::{
    assert_no_excess_float_precision, check_schema_doc, collect_warning_strings_json,
    compare_json_key_drift, ensure_exists, find_first_named_file, json_u64,
    normalize_benchmark_html, relative_diff, sorted_unique, toml_string, toml_to_json_value,
    toml_value_string, value_string,
};
use self::execution_support::{
    env_flag, read_json_value, read_utf8, run_program, run_program_with_env, run_programs_with_env,
    walk_file_list, write_json_pretty, write_utf8,
};
use self::toy_support::{
    build_combined_toy_report, compare_toy_goldens, copy_dir_all, generate_toy_profile,
    temp_subdir, toy_profile_id, verify_toy_inputs,
};
use self::verification::{
    ensure_help_only, failure_lines, merge_outcomes, run_check_ids, success_line, test_toy_runs,
};
use crate::application::checks::CheckApplication;
use crate::application::containers::ContainerApplication;
use crate::application::domain::DomainApplication;
use crate::model::check::{CheckSelection, CheckStatus};
use crate::model::ops::{NativeOpsCommandKey, OpsCommandOutcome};
use crate::runtime::workspace::Workspace;

pub fn run_native_ops_command(
    key: NativeOpsCommandKey,
    workspace: &Workspace,
    args: &[String],
) -> Result<OpsCommandOutcome> {
    dispatch::run_native_ops_command(key, workspace, args)
}

fn trim_quoted(raw: &str) -> String {
    raw.trim().trim_matches('"').to_string()
}

fn stable_now_utc_string() -> String {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.parse::<i64>().ok())
        .and_then(|epoch| chrono::DateTime::<Utc>::from_timestamp(epoch, 0))
        .map_or_else(
            || "1970-01-01T00:00:00Z".to_string(),
            |value| value.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        )
}

fn stable_now_utc_compact() -> String {
    stable_now_utc_string().replace([':', '-'], "")
}

fn ci_test_env(workspace: &Workspace, slow: bool) -> Result<Vec<(String, String)>> {
    let mut envs = artifact_env(workspace)?;
    let artifact_root = artifact_root_path(workspace)?;
    envs.push(("TZ".to_string(), "UTC".to_string()));
    envs.push(("LC_ALL".to_string(), "C".to_string()));
    envs.push(("TEST_TARGET_DIR".to_string(), artifact_root.join("target").display().to_string()));
    envs.push(("COV_TARGET_DIR".to_string(), artifact_root.join("target").display().to_string()));
    envs.push(("TEST_TMP_DIR".to_string(), artifact_root.join("tmp/test").display().to_string()));
    envs.push((
        "COV_TMP_DIR".to_string(),
        artifact_root.join("tmp/coverage").display().to_string(),
    ));
    envs.push((
        "TEST_PROFRAW_DIR".to_string(),
        artifact_root.join("coverage/profraw-test").display().to_string(),
    ));
    envs.push((
        "COV_PROFRAW_DIR".to_string(),
        artifact_root.join("coverage/profraw-coverage").display().to_string(),
    ));
    if slow {
        if let Ok(output) = std::process::Command::new("sh")
            .args(["-c", "command -v sccache || true"])
            .current_dir(&workspace.root)
            .output()
        {
            let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if !path.is_empty() {
                envs.push(("RUSTC_WRAPPER".to_string(), path));
            }
        }
    }
    Ok(envs)
}

fn resolved_nextest_expression(fast_only: bool) -> Option<String> {
    if let Ok(value) = std::env::var("NEXTEST_TEST_EXPR") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    if let Ok(value) = std::env::var("NEXTEST_FAST_EXPR") {
        if !value.trim().is_empty() {
            return Some(value);
        }
    }
    if fast_only || std::env::var("NEXTEST_PROFILE").ok().as_deref() == Some("fast-unit") {
        return Some("not test(/::slow__/)".to_string());
    }
    None
}

fn resolved_nextest_profile(slow: bool) -> Result<String> {
    if let Ok(value) = std::env::var("NEXTEST_PROFILE") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&Workspace::resolve()?.path("configs/coverage/runner.toml"))?)?;
    if slow {
        return Ok("slow-integration".to_string());
    }
    Ok(cfg.get("nextest_profile").and_then(TomlValue::as_str).unwrap_or("ci").to_string())
}

fn resolved_nextest_threads(slow: bool) -> Result<String> {
    if let Ok(value) = std::env::var("NEXTEST_TEST_THREADS") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    if slow {
        return Ok("8".to_string());
    }
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&Workspace::resolve()?.path("configs/coverage/runner.toml"))?)?;
    Ok(cfg.get("test_threads").and_then(TomlValue::as_integer).unwrap_or(1).to_string())
}

fn resolved_run_ignored(slow: bool) -> Result<String> {
    if let Ok(value) = std::env::var("RUN_IGNORED") {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    if slow {
        return Ok("--run-ignored all".to_string());
    }
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&Workspace::resolve()?.path("configs/coverage/runner.toml"))?)?;
    Ok(if cfg.get("run_ignored").and_then(TomlValue::as_bool).unwrap_or(true) {
        "--run-ignored all".to_string()
    } else {
        String::new()
    })
}

fn read_coverage_runner_flag(workspace: &Workspace, key: &str, flag: &str) -> Result<String> {
    let cfg: TomlValue =
        toml::from_str(&read_utf8(&workspace.path("configs/coverage/runner.toml"))?)?;
    Ok(if cfg.get(key).and_then(TomlValue::as_bool).unwrap_or(false) {
        flag.to_string()
    } else {
        String::new()
    })
}

fn set_assets_readonly(workspace: &Workspace, readonly: bool) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;

        for entry in
            WalkDir::new(workspace.path("assets")).into_iter().filter_map(std::result::Result::ok)
        {
            let metadata = std::fs::metadata(entry.path())
                .with_context(|| format!("read metadata {}", entry.path().display()))?;
            let mut perms = metadata.permissions();
            let mode = perms.mode();
            if readonly {
                perms.set_mode(mode & !0o222);
            } else {
                perms.set_mode((mode | 0o200) & !0o022);
            }
            std::fs::set_permissions(entry.path(), perms)
                .with_context(|| format!("set permissions {}", entry.path().display()))?;
        }
    }
    Ok(())
}

fn glob_paths(workspace: &Workspace, pattern: &str) -> Result<Vec<PathBuf>> {
    let regex = glob_to_regex(pattern)?;
    let outcome = run_program(
        workspace,
        "rg",
        &["--files".to_string(), workspace.root.display().to_string()],
    );
    if let Ok(outcome) = outcome {
        if outcome.is_success() {
            return Ok(outcome
                .stdout
                .lines()
                .map(PathBuf::from)
                .filter(|path| regex.is_match(&workspace.rel(path).to_string_lossy()))
                .collect());
        }
    }
    Ok(WalkDir::new(&workspace.root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(walkdir::DirEntry::into_path)
        .filter(|path| regex.is_match(&workspace.rel(path).to_string_lossy()))
        .collect())
}

fn glob_to_regex(pattern: &str) -> Result<Regex> {
    let mut raw = String::from("^");
    let mut chars = pattern.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '*' if chars.peek() == Some(&'*') => {
                let _ = chars.next();
                raw.push_str(".*");
            }
            '*' => raw.push_str("[^/]*"),
            '.' => raw.push_str(r"\."),
            '?' => raw.push('.'),
            '/' => raw.push('/'),
            other => raw.push_str(&regex::escape(&other.to_string())),
        }
    }
    raw.push('$');
    Regex::new(&raw).context("compile glob regex")
}

fn rg_lines(workspace: &Workspace, path: &str, pattern: &str) -> Result<Vec<String>> {
    if command_exists(workspace, "rg")? {
        let outcome = run_program(
            workspace,
            "rg",
            &["-n".to_string(), pattern.to_string(), workspace.path(path).display().to_string()],
        )?;
        if !outcome.is_success() {
            return Ok(Vec::new());
        }
        return Ok(outcome.stdout.lines().map(ToOwned::to_owned).collect());
    }
    let outcome = run_program(
        workspace,
        "grep",
        &[
            "-R".to_string(),
            "-n".to_string(),
            "--".to_string(),
            pattern.to_string(),
            workspace.path(path).display().to_string(),
        ],
    )?;
    if !outcome.is_success() {
        return Ok(Vec::new());
    }
    Ok(outcome.stdout.lines().map(ToOwned::to_owned).collect())
}

fn find_example_dir(workspace: &Workspace, example_id: &str) -> Result<Option<PathBuf>> {
    for example_toml in glob_paths(workspace, "examples/**/example.toml")? {
        let data: TomlValue = toml::from_str(&read_utf8(&example_toml)?)?;
        if data.get("id").and_then(TomlValue::as_str) == Some(example_id) {
            return Ok(example_toml.parent().map(Path::to_path_buf));
        }
    }
    Ok(None)
}

fn ensure_generated_header(
    workspace: &Workspace,
    rel: &str,
    errors: &mut Vec<String>,
) -> Result<()> {
    ensure_generated_header_path(workspace, &workspace.path(rel), errors)
}

fn ensure_generated_header_path(
    workspace: &Workspace,
    path: &Path,
    errors: &mut Vec<String>,
) -> Result<()> {
    let head = read_utf8(path)?.lines().take(6).collect::<Vec<_>>().join("\n");
    if !head.contains("GENERATED FILE - DO NOT EDIT") {
        errors.push(format!("missing generated header in {}", workspace.rel(path).display()));
    }
    Ok(())
}

fn generate_tool_index(workspace: &Workspace, out: &Path) -> Result<()> {
    let summary_path = workspace.path("artifacts/containers/summary.json");
    let mut tools = BTreeMap::<String, Value>::new();
    let mut vcf_downstream = BTreeMap::<String, Value>::new();
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        let value: TomlValue = toml::from_str(&read_utf8(&workspace.path(rel))?)?;
        let entries = value.get("tools").and_then(TomlValue::as_array).cloned().unwrap_or_default();
        for entry in entries {
            let Some(tool_id) = entry.get("id").and_then(TomlValue::as_str) else {
                continue;
            };
            let stage_ids = entry
                .get("stage_ids")
                .and_then(TomlValue::as_array)
                .cloned()
                .unwrap_or_default()
                .into_iter()
                .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                .collect::<Vec<_>>();
            tools.insert(
                tool_id.to_string(),
                json!({
                    "purpose": entry.get("tool_role").and_then(TomlValue::as_str).unwrap_or("unknown"),
                    "stages": stage_ids,
                    "container_ref": entry.get("container_ref").and_then(TomlValue::as_str).unwrap_or("-"),
                    "citation": entry.get("citation").and_then(TomlValue::as_str).unwrap_or("TBD"),
                    "status": entry.get("status").and_then(TomlValue::as_str).unwrap_or("unknown"),
                    "version": entry.get("version").and_then(TomlValue::as_str).unwrap_or("-"),
                }),
            );
            if entry.get("domain").and_then(TomlValue::as_str) == Some("vcf")
                && stage_ids.iter().any(|stage| stage.starts_with("vcf."))
            {
                vcf_downstream.insert(
                    tool_id.to_string(),
                    json!({
                        "status": entry.get("status").and_then(TomlValue::as_str).unwrap_or("unknown"),
                        "stages": stage_ids,
                    }),
                );
            }
        }
    }
    let mut self_reports = BTreeMap::<String, Value>::new();
    if summary_path.is_file() {
        let summary: Value = serde_json::from_str(&read_utf8(&summary_path)?)?;
        if let Some(items) = summary.get("items").and_then(Value::as_array) {
            for item in items {
                let Some(tool) = item.get("tool").and_then(Value::as_str) else {
                    continue;
                };
                let Some(manifest_path) = item.get("manifest").and_then(Value::as_str) else {
                    continue;
                };
                let manifest_path = PathBuf::from(manifest_path);
                if !manifest_path.is_file() {
                    continue;
                }
                let manifest: Value = serde_json::from_str(&read_utf8(&manifest_path)?)?;
                let Some(report_path) = manifest.get("self_report_path").and_then(Value::as_str)
                else {
                    continue;
                };
                let report_path = PathBuf::from(report_path);
                if report_path.is_file() {
                    self_reports
                        .insert(tool.to_string(), serde_json::from_str(&read_utf8(&report_path)?)?);
                }
            }
        }
    }
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-tool-index -->".to_string(),
        String::new(),
        "# TOOL_INDEX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated index of registry tools with stage bindings and container references/self-reports.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Source of truth = registry contracts + `artifacts/containers/summary.json` self-reports when available.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing full scientific method docs for each domain.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Manual edits are forbidden; regenerate via native control-plane.".to_string(),
        "- Source of truth is registry + containers; this file is a rendered view.".to_string(),
        "- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.".to_string(),
        String::new(),
        "See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)".to_string(),
        "See also: [VCF Downstream Roadmap](vcf/ROADMAP.md)".to_string(),
        String::new(),
        "## VCF Downstream / IBD Toolkit".to_string(),
        String::new(),
    ];
    for (tool_id, info) in &vcf_downstream {
        let stages = info
            .get("stages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        lines.push(format!(
            "- `{tool_id}` ({}) : {}",
            info.get("status").and_then(Value::as_str).unwrap_or("unknown"),
            if stages.is_empty() { "-".to_string() } else { stages }
        ));
    }
    lines.extend([
        String::new(),
        "| Tool ID | Purpose | Stage Bindings | Container Ref | Version | Citation | Status |"
            .to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    for (tool_id, row) in tools {
        let stages = row
            .get("stages")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|value| value.as_str().map(ToOwned::to_owned))
            .collect::<Vec<_>>()
            .join(", ");
        let version = self_reports
            .get(&tool_id)
            .and_then(|report| report.get("version"))
            .and_then(Value::as_str)
            .unwrap_or_else(|| row.get("version").and_then(Value::as_str).unwrap_or("-"));
        lines.push(format!(
            "| `{tool_id}` | `{}` | `{}` | `{}` | `{}` | {} | `{}` |",
            row.get("purpose").and_then(Value::as_str).unwrap_or("unknown"),
            if stages.is_empty() { "-" } else { &stages },
            row.get("container_ref").and_then(Value::as_str).unwrap_or("-"),
            version,
            row.get("citation").and_then(Value::as_str).unwrap_or("TBD"),
            row.get("status").and_then(Value::as_str).unwrap_or("unknown"),
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_domain_coverage_doc(workspace: &Workspace, out: &Path) -> Result<()> {
    let domain_root = workspace.path("domain");
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-domain-coverage-doc -->".to_string(),
        String::new(),
        "# DOMAIN_COVERAGE".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated coverage table for domain stages/tools/fixtures.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Derived from `domain/*/{stages,tools,fixtures}`.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing per-domain scientific specifications.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Generated-only document; manual edits are forbidden.".to_string(),
        "- Counts must be deterministic for a fixed repository state.".to_string(),
        String::new(),
        "| Domain | Stage Count | Tool Count | Fixture Count |".to_string(),
        "|---|---:|---:|---:|".to_string(),
    ];
    let mut domain_entries = fs::read_dir(&domain_root)?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect::<Vec<_>>();
    domain_entries.sort();
    for path in domain_entries {
        if !path.is_dir() {
            continue;
        }
        let domain = path.file_name().and_then(|value| value.to_str()).unwrap_or("unknown");
        let stages = count_schema_filtered(path.join("stages"))?;
        let tools = count_schema_filtered(path.join("tools"))?;
        let fixtures = glob_count(path.join("fixtures"), "*.txt")?;
        lines.push(format!("| `{domain}` | {stages} | {tools} | {fixtures} |"));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn tracked_visible_repo_root_entries(workspace: &Workspace) -> Result<Vec<PathBuf>> {
    let tracked = run_program(workspace, "git", &["ls-files".to_string()])?;
    if !tracked.is_success() {
        return Err(anyhow!("git ls-files failed while generating repo root map"));
    }

    let mut names = BTreeSet::new();
    for rel in tracked.stdout.lines().filter(|line| !line.trim().is_empty()) {
        let top = rel.split('/').next().unwrap_or_default().trim();
        if top.is_empty() || top.starts_with('.') {
            continue;
        }
        names.insert(top.to_string());
    }

    Ok(names.into_iter().map(|name| workspace.path(&name)).collect())
}

fn generate_repo_root_map(workspace: &Workspace, out: &Path) -> Result<()> {
    let owners_path = workspace.path("configs/OWNERS.toml");
    let owners: TomlValue = toml::from_str(&read_utf8(&owners_path)?)?;
    let rules = owners.get("rule").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-repo-root-map -->".to_string(),
        String::new(),
        "# REPO_ROOT_MAP".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated map of repository root entries with inferred ownership and intent.".to_string(),
        String::new(),
        "## Scope".to_string(),
        "Top-level workspace paths only.".to_string(),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing detailed per-subtree architecture docs.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Ownership for config paths is sourced from `configs/OWNERS.toml`.".to_string(),
        "- Script subtree intent is sourced from README `Purpose:` lines.".to_string(),
        String::new(),
        "| Path | Kind | Owner | Purpose |".to_string(),
        "|---|---|---|---|".to_string(),
    ];
    for path in tracked_visible_repo_root_entries(workspace)? {
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        let rel = name.to_string();
        let kind = if path.is_dir() { "dir" } else { "file" };
        let purpose = path
            .join("README.md")
            .is_file()
            .then(|| read_purpose_line(&path.join("README.md")))
            .transpose()?
            .flatten()
            .unwrap_or_else(|| "-".to_string());
        let owner = owner_for(&rules, if kind == "dir" { format!("{rel}/") } else { rel.clone() });
        lines.push(format!("| `{rel}` | `{kind}` | `{owner}` | {purpose} |"));
    }
    lines.extend([
        String::new(),
        "## Automation Intent".to_string(),
        "| Control Plane Path | Purpose |".to_string(),
        "|---|---|".to_string(),
    ]);
    for rel in ["crates/bijux-dna-dev", "makes"] {
        let path = workspace.path(rel);
        let purpose =
            read_purpose_line(&path.join("README.md"))?.unwrap_or_else(|| "-".to_string());
        lines.push(format!("| `{rel}` | {purpose} |"));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

fn generate_compatibility_matrix(workspace: &Workspace, out: &Path) -> Result<()> {
    let catalog_root = workspace.path("crates/bijux-dna-core/src/id_catalog/pipeline");
    let mut catalog = String::new();
    for entry in WalkDir::new(&catalog_root).into_iter().filter_map(std::result::Result::ok).filter(
        |entry| {
            entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs")
        },
    ) {
        catalog.push_str(&read_utf8(entry.path())?);
        catalog.push('\n');
    }
    let profile_re = Regex::new(r#"pub const PIPELINE_[A-Z0-9_]+: &str = "([^"]+)";"#)?;
    let profiles = profile_re
        .captures_iter(&catalog)
        .filter_map(|capture| capture.get(1).map(|value| value.as_str().to_string()))
        .collect::<Vec<_>>();
    let mut tool_count = 0usize;
    for rel in [
        "configs/ci/registry/tool_registry.toml",
        "configs/ci/registry/tool_registry_vcf.toml",
        "configs/ci/registry/tool_registry_experimental.toml",
        "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ] {
        tool_count += read_utf8(&workspace.path(rel))?
            .lines()
            .filter(|line| line.trim() == "[[tools]]")
            .count();
    }
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-compatibility-matrix -->".to_string(),
        String::new(),
        "# COMPATIBILITY_MATRIX".to_string(),
        String::new(),
        "## Purpose".to_string(),
        "Generated compatibility matrix derived from pipeline profile IDs and tool registry inventory.".to_string(),
        String::new(),
        "## Scope".to_string(),
        format!(
            "Profiles sourced from `crates/bijux-dna-core/src/id_catalog/pipeline/`; registries include {tool_count} tool entries."
        ),
        String::new(),
        "## Non-goals".to_string(),
        "- Replacing detailed per-domain migration guides.".to_string(),
        String::new(),
        "## Contracts".to_string(),
        "- Matrix is generated-only and must not be manually edited.".to_string(),
        "- Breaking contract changes require version/schema updates and matrix regeneration.".to_string(),
        String::new(),
        "| Pipeline Profile | Domain | Stability | Plan Contract | Report Contract | Compatibility Rule |".to_string(),
        "|---|---|---|---|---|---|".to_string(),
    ];
    let mut rows = profiles
        .into_iter()
        .map(|profile| {
            let domain = profile.split("-to-").next().unwrap_or("unknown").to_string();
            let stability = if profile.contains("reference") || profile.contains("default") {
                "stable"
            } else {
                "experimental"
            };
            (profile, domain, stability.to_string())
        })
        .collect::<Vec<_>>();
    rows.sort();
    for (profile, domain, stability) in rows {
        lines.push(format!(
            "| `{profile}` | `{domain}` | `{stability}` | `v1` | `v1` | compatible if stage/tool contracts unchanged |"
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))
}

#[derive(Debug, Deserialize)]
struct CompatibilityDeprecationsConfig {
    schema_version: String,
    deprecation: Vec<CompatibilityDeprecationRow>,
}

#[derive(Debug, Deserialize)]
struct CompatibilityDeprecationRow {
    kind: String,
    subject: String,
    replacement: String,
    deadline: String,
    migration_test_status: String,
    source: String,
    notes: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseChangesConfig {
    schema_version: String,
    release_id: String,
    title: String,
    area_status: ReleaseAreaStatus,
    change: Vec<ReleaseChangeRow>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAreaStatus {
    #[serde(flatten)]
    workflow_assets: ReleaseWorkflowAssetStatus,
    #[serde(flatten)]
    runtime_assets: ReleaseRuntimeAssetStatus,
    #[serde(flatten)]
    interface_changes: ReleaseInterfaceStatus,
}

#[derive(Debug, Deserialize)]
struct ReleaseWorkflowAssetStatus {
    schemas: bool,
    defaults: bool,
    tools: bool,
}

#[derive(Debug, Deserialize)]
struct ReleaseRuntimeAssetStatus {
    containers: bool,
    evidence_expectations: bool,
}

#[derive(Debug, Deserialize)]
struct ReleaseInterfaceStatus {
    api: bool,
    errors: bool,
}

#[derive(Debug, Deserialize)]
struct ReleaseChangeRow {
    area: String,
    subject: String,
    changed: bool,
    summary: String,
    migration: String,
    test: String,
}

fn generate_compatibility_reference_docs(workspace: &Workspace, out_root: &Path) -> Result<()> {
    generate_schema_registry_doc(workspace, &out_root.join("SCHEMA_REGISTRY.md"))?;
    generate_api_versioning_doc(workspace, &out_root.join("API_VERSIONING.md"))?;
    generate_deprecation_dashboard_doc(workspace, &out_root.join("DEPRECATION_DASHBOARD.md"))?;
    generate_upgrade_guide_doc(workspace, &out_root.join("UPGRADE_GUIDE.md"))?;
    Ok(())
}

fn push_standard_doc_sections(
    lines: &mut Vec<String>,
    purpose: &str,
    scope: &str,
    non_goals: &str,
    contracts: &str,
) {
    lines.extend([
        "## Purpose".to_string(),
        purpose.to_string(),
        String::new(),
        "## Scope".to_string(),
        scope.to_string(),
        String::new(),
        "## Non-goals".to_string(),
        non_goals.to_string(),
        String::new(),
        "## Contracts".to_string(),
        contracts.to_string(),
        String::new(),
    ]);
}

fn generate_schema_registry_doc(_workspace: &Workspace, out: &Path) -> Result<()> {
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->"
            .to_string(),
        String::new(),
        "# SCHEMA_REGISTRY".to_string(),
        String::new(),
    ];
    push_standard_doc_sections(
        &mut lines,
        "Generated registry of governed workflow, plan, runtime, evidence, metric, report, and error compatibility surfaces.",
        "Lists authoritative compatibility surfaces, their semantic versions, and durable error code ownership.",
        "Does not replace crate-level API docs, implementation details, or migration playbooks.",
        "This page is generated from governed registries in code and must be updated via `cargo run -p bijux-dna-dev -- tooling run generate-docs`.",
    );
    lines.extend([
        "## Schema Families".to_string(),
        "| Family | Schema | Semantic Version | Surface | Compatibility | Migration Rule | Owner |"
            .to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    for entry in bijux_dna_core::contract::governed_schema_registry() {
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{}` | `{}` | `{}` | `{}` |",
            entry.schema_family,
            entry.schema_version,
            entry.semantic_version,
            serde_json::to_string(&entry.surface_kind)?.trim_matches('"'),
            serde_json::to_string(&entry.compatibility_class)?.trim_matches('"'),
            serde_json::to_string(&entry.migration_rule)?.trim_matches('"'),
            entry.owner_crate
        ));
    }
    lines.extend([
        String::new(),
        "## Durable Error Codes".to_string(),
        "| Error ID | Area | Wire Code | Owner | Remediation |".to_string(),
        "|---|---|---|---|---|".to_string(),
    ]);
    for entry in bijux_dna_core::contract::governed_error_code_registry() {
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{}` | {} |",
            entry.error_id,
            serde_json::to_string(&entry.area)?.trim_matches('"'),
            entry.wire_code,
            entry.owner_surface,
            entry.remediation
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn generate_api_versioning_doc(_workspace: &Workspace, out: &Path) -> Result<()> {
    let inventory = bijux_dna_api::v1::api::route_version_inventory();
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->"
            .to_string(),
        String::new(),
        "# API_VERSIONING".to_string(),
        String::new(),
    ];
    push_standard_doc_sections(
        &mut lines,
        "Generated inventory linking stable v1 API routes to the governed workflow, plan, runtime, and evidence schemas they read or surface.",
        "Documents route-level read/write schema surfaces for the currently shipped API version.",
        "Does not define transport-level behavior, authentication policy, or route implementation internals.",
        "Route inventory is sourced from `bijux-dna-api` and regenerated by `cargo run -p bijux-dna-dev -- tooling run generate-docs`.",
    );
    lines.extend([
        format!("- Inventory schema: `{}`", inventory.schema_version),
        format!("- API version: `{}`", inventory.api_version),
        String::new(),
        "| Route | Response Struct | Reads | Writes |".to_string(),
        "|---|---|---|---|".to_string(),
    ]);
    for route in inventory.routes {
        let reads = if route.reads_schema_families.is_empty() {
            "-".to_string()
        } else {
            route.reads_schema_families.join(", ")
        };
        let writes = if route.writes_schema_families.is_empty() {
            "-".to_string()
        } else {
            route.writes_schema_families.join(", ")
        };
        lines.push(format!(
            "| `{}` | `{}` | `{}` | `{}` |",
            route.route_id, route.response_struct, reads, writes
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn generate_deprecation_dashboard_doc(workspace: &Workspace, out: &Path) -> Result<()> {
    let cfg: CompatibilityDeprecationsConfig =
        toml::from_str(&read_utf8(&workspace.path("configs/ci/compatibility/deprecations.toml"))?)?;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->"
            .to_string(),
        String::new(),
        "# DEPRECATION_DASHBOARD".to_string(),
        String::new(),
    ];
    push_standard_doc_sections(
        &mut lines,
        "Generated dashboard of deprecated stage ids, tool ids, metric ids, params, and fields with replacement and migration coverage.",
        "Summarizes governed deprecations from compatibility policy configuration and their migration status.",
        "Does not define runtime enforcement behavior beyond what source policies and tests already enforce.",
        "Deprecation rows come from `configs/ci/compatibility/deprecations.toml` and are regenerated by `cargo run -p bijux-dna-dev -- tooling run generate-docs`.",
    );
    lines.extend([
        format!("- Source schema: `{}`", cfg.schema_version),
        String::new(),
        "| Kind | Subject | Replacement | Deadline | Migration Test Status | Source | Notes |"
            .to_string(),
        "|---|---|---|---|---|---|---|".to_string(),
    ]);
    for row in cfg.deprecation {
        lines.push(format!(
            "| `{}` | {} | {} | `{}` | `{}` | `{}` | {} |",
            row.kind,
            row.subject,
            row.replacement,
            row.deadline,
            row.migration_test_status,
            row.source,
            row.notes
        ));
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn generate_upgrade_guide_doc(workspace: &Workspace, out: &Path) -> Result<()> {
    let cfg: ReleaseChangesConfig = toml::from_str(&read_utf8(
        &workspace.path("configs/ci/compatibility/release_changes.toml"),
    )?)?;
    let mut lines = vec![
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        "<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->"
            .to_string(),
        String::new(),
        "# UPGRADE_GUIDE".to_string(),
        String::new(),
    ];
    push_standard_doc_sections(
        &mut lines,
        "Release-specific compatibility notes for governed schema, API, and evidence changes.",
        "Covers migration-facing impacts declared in `configs/ci/compatibility/release_changes.toml`.",
        "Does not replace crate changelogs, release process policy, or ad hoc migration playbooks.",
        "This guide is generated from governed compatibility config via `cargo run -p bijux-dna-dev -- tooling run generate-docs`.",
    );
    lines.extend([
        format!("Release: `{}`", cfg.release_id),
        String::new(),
        format!("Title: {}", cfg.title),
        String::new(),
        format!("Source schema: `{}`", cfg.schema_version),
        String::new(),
        "## Area Status".to_string(),
        format!("- Schemas changed: `{}`", cfg.area_status.workflow_assets.schemas),
        format!("- Defaults changed: `{}`", cfg.area_status.workflow_assets.defaults),
        format!("- Tools changed: `{}`", cfg.area_status.workflow_assets.tools),
        format!("- Containers changed: `{}`", cfg.area_status.runtime_assets.containers),
        format!(
            "- Evidence expectations changed: `{}`",
            cfg.area_status.runtime_assets.evidence_expectations
        ),
        format!("- API changed: `{}`", cfg.area_status.interface_changes.api),
        format!("- Error registry changed: `{}`", cfg.area_status.interface_changes.errors),
        String::new(),
        "## Changes".to_string(),
    ]);
    let areas = [
        ("schemas", "Schemas"),
        ("defaults", "Defaults"),
        ("tools", "Tools"),
        ("containers", "Containers"),
        ("evidence_expectations", "Evidence Expectations"),
        ("api", "API"),
        ("errors", "Errors"),
    ];
    for (area_id, heading) in areas {
        lines.push(String::new());
        lines.push(format!("### {heading}"));
        let matching =
            cfg.change.iter().filter(|row| row.area == area_id && row.changed).collect::<Vec<_>>();
        if matching.is_empty() {
            lines.push("No governed changes declared in this release.".to_string());
            continue;
        }
        for row in matching {
            lines.push(format!("- `{}`: {}", row.subject, row.summary));
            lines.push(format!("  Migration: {}", row.migration));
            lines.push(format!("  Test: `{}`", row.test));
        }
    }
    write_utf8(out, &format!("{}\n", lines.join("\n")))?;
    Ok(())
}

fn generate_docs_graph(workspace: &Workspace, out: &Path) -> Result<()> {
    let docs_root = workspace.path("docs");
    let mut lines = vec![
        "# GENERATED FILE - DO NOT EDIT".to_string(),
        "# Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs-graph"
            .to_string(),
        String::new(),
    ];
    let mut dirs = vec![docs_root.clone()];
    dirs.extend(
        WalkDir::new(&docs_root)
            .into_iter()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry.file_type().is_dir())
            .map(|entry| entry.path().to_path_buf()),
    );
    dirs.sort();
    for dir in dirs {
        let index = dir.join("index.md");
        if !index.is_file() {
            continue;
        }
        let from = workspace.rel(&index).display().to_string();
        let mut children = Vec::new();
        for entry in fs::read_dir(&dir)?.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if path.is_file()
                && path.extension().and_then(|ext| ext.to_str()) == Some("md")
                && path.file_name().and_then(|value| value.to_str()) != Some("index.md")
            {
                children.push(workspace.rel(&path).display().to_string());
            }
            if path.is_dir() && path.join("index.md").is_file() {
                children.push(workspace.rel(&path.join("index.md")).display().to_string());
            }
        }
        children.sort();
        lines.push("[[edge]]".to_string());
        lines.push(format!("from = \"{from}\""));
        lines.push("children = [".to_string());
        for child in children {
            lines.push(format!("  \"{child}\","));
        }
        lines.push("]".to_string());
        lines.push(String::new());
    }
    write_utf8(out, &lines.join("\n"))
}

fn write_checksum_manifest(manifest_path: &Path, rel_paths: &[&str]) -> Result<()> {
    let base = manifest_path.parent().context("checksum manifest path missing parent directory")?;
    let mut lines = Vec::new();
    for rel in rel_paths {
        let path = base.join(rel);
        lines.push(format!("{}  {}", sha256_hex(&path)?, rel));
    }
    write_utf8(manifest_path, &format!("{}\n", lines.join("\n")))
}

fn write_refresh_report(
    content_root: &Path,
    report_path: &Path,
    asset: &str,
    generator_command: &str,
) -> Result<()> {
    let mut files = WalkDir::new(content_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.path().to_path_buf())
        .collect::<Vec<_>>();
    files.sort();

    let mut checksums = serde_json::Map::new();
    let mut listed = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(content_root)
            .context("strip content root prefix")?
            .to_string_lossy()
            .to_string();
        listed.push(rel.clone());
        checksums.insert(rel, json!(sha256_hex(&path)?));
    }

    write_json_pretty(
        report_path,
        &json!({
            "schema_version": "bijux.assets.refresh_report.v1",
            "asset": asset,
            "generator_command": generator_command,
            "inputs": listed,
            "input_list": listed,
            "output_checksums": checksums,
            "tool_versions": refresh_tool_versions(),
            "checksums": checksums,
        }),
    )
}

fn refresh_tool_versions() -> Value {
    json!({
        "bijux-dna-dev": env!("CARGO_PKG_VERSION"),
        "cargo": command_version_line("cargo", &["--version"]),
        "rustc": command_version_line("rustc", &["--version"]),
    })
}

fn command_version_line(program: &str, args: &[&str]) -> String {
    std::process::Command::new(program)
        .args(args)
        .output()
        .ok()
        .and_then(|output| {
            output.status.success().then(|| {
                String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string()
            })
        })
        .filter(|line| !line.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn replace_dir(src: &Path, dst: &Path) -> Result<()> {
    if dst.exists() {
        fs::remove_dir_all(dst).with_context(|| format!("remove {}", dst.display()))?;
    }
    if let Some(parent) = dst.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    copy_dir_recursive(src, dst)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    bijux_dna_infra::ensure_dir(dst).with_context(|| format!("create {}", dst.display()))?;
    for entry in WalkDir::new(src).into_iter().filter_map(std::result::Result::ok) {
        let path = entry.path();
        let rel = path.strip_prefix(src).context("strip copy source prefix")?;
        if rel.as_os_str().is_empty() {
            continue;
        }
        let target = dst.join(rel);
        if entry.file_type().is_dir() {
            bijux_dna_infra::ensure_dir(&target)
                .with_context(|| format!("create {}", target.display()))?;
        } else {
            if let Some(parent) = target.parent() {
                bijux_dna_infra::ensure_dir(parent)
                    .with_context(|| format!("create {}", parent.display()))?;
            }
            fs::copy(path, &target)
                .with_context(|| format!("copy {} -> {}", path.display(), target.display()))?;
        }
    }
    Ok(())
}

fn config_tree_snapshot_text(workspace: &Workspace) -> Result<String> {
    let configs_root = workspace.path("configs");
    let mut files = WalkDir::new(&configs_root)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| workspace.rel(entry.path()).to_string_lossy().to_string())
        .collect::<Vec<_>>();
    files.sort();
    let mut lines = vec![
        "# GENERATED - DO NOT EDIT".to_string(),
        "# generator = cargo run -p bijux-dna-dev -- tooling run generate-config-tree-snapshot"
            .to_string(),
        "# schema_version = 1".to_string(),
        "# owner = bijux-dna-infra".to_string(),
    ];
    lines.extend(files);
    Ok(format!("{}\n", lines.join("\n")))
}

fn config_snapshot_inputs_changed(workspace: &Workspace) -> Result<bool> {
    let in_repo = run_program(
        workspace,
        "git",
        &["rev-parse".to_string(), "--is-inside-work-tree".to_string()],
    )?;
    if !in_repo.is_success() {
        return Ok(true);
    }
    let watched = [
        "configs/",
        "crates/bijux-dna-dev/src/model/ops.rs",
        "crates/bijux-dna-dev/src/commands/ops.rs",
        "crates/bijux-dna-dev/src/catalog/ops.rs",
    ];
    let mut staged_args = vec![
        "diff".to_string(),
        "--name-only".to_string(),
        "--cached".to_string(),
        "--".to_string(),
    ];
    staged_args.extend(watched.iter().map(|item| (*item).to_string()));
    let staged = run_program(workspace, "git", &staged_args)?;
    if staged.is_success() && !staged.stdout.trim().is_empty() {
        return Ok(true);
    }

    let mut working_args = vec!["diff".to_string(), "--name-only".to_string(), "--".to_string()];
    working_args.extend(watched.iter().map(|item| (*item).to_string()));
    let working = run_program(workspace, "git", &working_args)?;
    Ok(!working.is_success() || !working.stdout.trim().is_empty())
}

fn count_schema_filtered(dir: PathBuf) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    Ok(fs::read_dir(dir)?
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml")
                && entry.file_name().to_string_lossy() != "_schema.yaml"
        })
        .count())
}

fn glob_count(dir: PathBuf, suffix: &str) -> Result<usize> {
    if !dir.is_dir() {
        return Ok(0);
    }
    let wanted = suffix.trim_start_matches('*');
    Ok(WalkDir::new(dir)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| {
            entry
                .path()
                .file_name()
                .and_then(|value| value.to_str())
                .is_some_and(|name| name.ends_with(wanted))
        })
        .count())
}

fn read_purpose_line(path: &Path) -> Result<Option<String>> {
    if !path.is_file() {
        return Ok(None);
    }
    Ok(read_utf8(path)?
        .lines()
        .find_map(|line| line.strip_prefix("Purpose:").map(|value| value.trim().to_string())))
}

fn owner_for(rules: &[TomlValue], rel: String) -> String {
    let hits = rules
        .iter()
        .filter_map(|rule| {
            let prefix = rule.get("prefix").and_then(TomlValue::as_str)?;
            rel.starts_with(prefix)
                .then(|| rule.get("owner").and_then(TomlValue::as_str).unwrap_or("-").to_string())
        })
        .collect::<Vec<_>>();
    if hits.len() == 1 {
        hits[0].clone()
    } else {
        "-".to_string()
    }
}

fn lab_config(workspace: &Workspace) -> Result<TomlValue> {
    let path = PathBuf::from(env_or_default("CONFIG_PATH", "configs/lab/config.toml"));
    let resolved =
        if path.is_absolute() { path } else { workspace.path(path.to_string_lossy().as_ref()) };
    if !resolved.is_file() {
        return Err(anyhow!(
            "config not found: {}\ncopy configs/lab/config_example.toml to configs/lab/config.toml",
            resolved.display()
        ));
    }
    let mut value: TomlValue =
        toml::from_str(&read_utf8(&resolved)?).context("parse lab config")?;
    expand_toml_env_placeholders(&mut value);
    Ok(value)
}

fn required_config_string(config: &TomlValue, field: &str, config_name: &str) -> Result<String> {
    config_string(config, field)
        .ok_or_else(|| anyhow!("{config_name} is missing required key `{field}`"))
}

fn config_string(config: &TomlValue, field: &str) -> Option<String> {
    let value = config.get(field)?;
    match value {
        TomlValue::String(text) => {
            let trimmed = text.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        TomlValue::Array(items) => {
            let values = items
                .iter()
                .map(TomlValue::as_str)
                .collect::<Option<Vec<_>>>()?
                .into_iter()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .collect::<Vec<_>>();
            if values.is_empty() {
                None
            } else {
                Some(values.join(","))
            }
        }
        _ => None,
    }
}

fn expand_toml_env_placeholders(value: &mut TomlValue) {
    match value {
        TomlValue::String(text) => *text = expand_env_placeholders_string(text),
        TomlValue::Array(items) => {
            for item in items {
                expand_toml_env_placeholders(item);
            }
        }
        TomlValue::Table(table) => {
            for (_, item) in table.iter_mut() {
                expand_toml_env_placeholders(item);
            }
        }
        _ => {}
    }
}

fn expand_env_placeholders_string(raw: &str) -> String {
    let mut expanded = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '$' && chars.peek() == Some(&'{') {
            chars.next();
            let mut name = String::new();
            for next in chars.by_ref() {
                if next == '}' {
                    break;
                }
                name.push(next);
            }
            expanded.push_str(&std::env::var(&name).unwrap_or_default());
            continue;
        }
        expanded.push(ch);
    }
    expanded
}

fn resolve_optional_output_arg(
    workspace: &Workspace,
    command: &str,
    args: &[String],
    default_rel: &str,
) -> Result<PathBuf> {
    match args {
        [] => Ok(workspace.path(default_rel)),
        [flag] if flag == "--help" || flag == "-h" => {
            Err(anyhow!("Usage: cargo run -p bijux-dna-dev -- tooling run {command} -- [out]"))
        }
        [out] => Ok(resolve_workspace_path(workspace, out)),
        _ => Err(anyhow!("Usage: cargo run -p bijux-dna-dev -- tooling run {command} -- [out]")),
    }
}

fn free_space_gb(path: &Path) -> Result<u64> {
    let outcome = run_program(
        &Workspace { root: path.canonicalize().unwrap_or_else(|_| path.to_path_buf()) },
        "df",
        &["-Pk".to_string(), path.display().to_string()],
    )?;
    let line = outcome.stdout.lines().nth(1).context("parse df output row")?;
    let available_kb = line
        .split_whitespace()
        .nth(3)
        .context("parse df available column")?
        .parse::<u64>()
        .context("parse df available kilobytes")?;
    Ok(available_kb / 1024 / 1024)
}

fn command_exists(workspace: &Workspace, program: &str) -> Result<bool> {
    let outcome = run_program(workspace, "which", &[program.to_string()])?;
    Ok(outcome.is_success())
}

fn hostname(workspace: &Workspace) -> Result<String> {
    let fqdn = run_program(workspace, "hostname", &["-f".to_string()])?;
    if fqdn.is_success() && !fqdn.stdout.trim().is_empty() {
        return Ok(trim_newline(&fqdn.stdout));
    }
    let fallback = run_program(workspace, "hostname", &[])?;
    Ok(trim_newline(&fallback.stdout))
}

fn host_matches_policy(host: &str, pattern: &str) -> Result<bool> {
    if pattern.trim().is_empty() {
        return Ok(false);
    }
    Ok(Regex::new(pattern)?.is_match(host))
}

fn trim_newline(raw: &str) -> String {
    raw.trim().to_string()
}

fn benchmark_sync_source_payload(workspace: &Workspace) -> Result<Value> {
    let benchmark_workspace = load_benchmark_workspace_paths(workspace)?;
    let source_commit = trim_newline(
        &run_program(workspace, "git", &["rev-parse".to_string(), "HEAD".to_string()])?.stdout,
    );
    let source_branch = trim_newline(
        &run_program(
            workspace,
            "git",
            &["rev-parse".to_string(), "--abbrev-ref".to_string(), "HEAD".to_string()],
        )?
        .stdout,
    );
    Ok(json!({
        "schema_version": "bijux.benchmark.sync_source.v1",
        "source_commit": source_commit,
        "source_branch": source_branch,
        "benchmark_workspace": {
            "local_results_root": benchmark_workspace.local_results_root,
            "local_cache_mirror_root": benchmark_workspace.local_cache_mirror_root,
            "remote_ssh_host": benchmark_workspace.remote_ssh_host,
            "remote_repo_root": benchmark_workspace.remote_repo_root,
            "remote_cache_root": benchmark_workspace.remote_cache_root,
            "remote_corpus_root": benchmark_workspace.remote_corpus_root,
            "remote_results_root": benchmark_workspace.remote_results_root,
            "remote_extra_data_root": benchmark_workspace.remote_extra_data_root,
            "remote_reference_root": benchmark_workspace.remote_reference_root,
            "remote_containers_root": benchmark_workspace.remote_containers_root,
        },
        "synced_at_utc": Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
    }))
}

fn write_benchmark_sync_source(workspace: &Workspace, path: &Path) -> Result<()> {
    write_json_pretty(path, &benchmark_sync_source_payload(workspace)?)
}

fn benchmark_sync_revision(workspace: &Workspace, host: &str, repo_dir: &str) -> Result<String> {
    let git_commit = trim_newline(
        &run_program(
            workspace,
            "ssh",
            &[
                host.to_string(),
                format!("cd '{repo_dir}' && git rev-parse HEAD 2>/dev/null || echo 'no-git-repo'"),
            ],
        )?
        .stdout,
    );
    if git_commit != "no-git-repo" {
        return Ok(git_commit);
    }
    let sync_source = run_program(
        workspace,
        "ssh",
        &[
            host.to_string(),
            format!("cat '{repo_dir}/BENCHMARK_SYNC_SOURCE.json' 2>/dev/null || true"),
        ],
    )?;
    let payload = trim_newline(&sync_source.stdout);
    if payload.is_empty() {
        return Ok("no-git-repo".to_string());
    }
    match serde_json::from_str::<Value>(&payload) {
        Ok(value) => Ok(value
            .get("source_commit")
            .and_then(Value::as_str)
            .unwrap_or("no-git-repo")
            .to_string()),
        Err(error) if error.io_error_kind() == Some(ErrorKind::UnexpectedEof) => {
            Ok("no-git-repo".to_string())
        }
        Err(_) => Ok("no-git-repo".to_string()),
    }
}

fn benchmark_sync_profile_path(path: &Path, profile: &str, field: &str) -> Result<Option<String>> {
    let value: TomlValue = toml::from_str(&read_utf8(path)?)?;
    let profiles = value.get("profiles").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    Ok(profiles.into_iter().find_map(|row| {
        (row.get("name").and_then(TomlValue::as_str) == Some(profile))
            .then(|| row.get(field).and_then(TomlValue::as_str).map(ToOwned::to_owned))
            .flatten()
    }))
}

#[derive(Default)]
struct BenchmarkWorkspacePaths {
    local_results_root: Option<String>,
    local_cache_mirror_root: Option<String>,
    sync_default_pull_base: Option<String>,
    sync_default_pull_mode: Option<String>,
    sync_default_include_profile: Option<String>,
    sync_default_exclude_profile: Option<String>,
    sync_default_clean_context: Option<bool>,
    sync_default_allow_dirty: Option<bool>,
    sync_default_include_containers_manifest: Option<bool>,
    sync_default_data_manifest_glob: Option<String>,
    remote_ssh_host: Option<String>,
    remote_repo_root: Option<String>,
    remote_cache_root: Option<String>,
    remote_corpus_root: Option<String>,
    remote_results_root: Option<String>,
    remote_extra_data_root: Option<String>,
    remote_reference_root: Option<String>,
    remote_containers_root: Option<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct BenchmarkSyncProfile {
    name: String,
    include_file: Option<String>,
    exclude_file: Option<String>,
    workspace_scope: Option<String>,
    pull_destination: Option<String>,
    remote_roots: Vec<String>,
    data_manifest_globs: Vec<String>,
}

fn load_benchmark_workspace_paths(workspace: &Workspace) -> Result<BenchmarkWorkspacePaths> {
    let path = workspace.path("configs/bench/benchmark.toml");
    if !path.is_file() {
        return Ok(BenchmarkWorkspacePaths::default());
    }
    let value: TomlValue =
        toml::from_str(&read_utf8(&path)?).with_context(|| format!("parse {}", path.display()))?;
    let workspace_table = value.get("workspace").and_then(TomlValue::as_table);
    let local = workspace_table.and_then(|table| table.get("local")).and_then(TomlValue::as_table);
    let remote =
        workspace_table.and_then(|table| table.get("remote")).and_then(TomlValue::as_table);
    let sync_defaults = workspace_table
        .and_then(|table| table.get("sync"))
        .and_then(TomlValue::as_table)
        .and_then(|table| table.get("defaults"))
        .and_then(TomlValue::as_table);
    Ok(BenchmarkWorkspacePaths {
        local_results_root: local
            .and_then(|table| table.get("results_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        local_cache_mirror_root: local
            .and_then(|table| table.get("cache_mirror_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_pull_base: sync_defaults
            .and_then(|table| table.get("pull_base"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_pull_mode: sync_defaults
            .and_then(|table| table.get("pull_mode"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_include_profile: sync_defaults
            .and_then(|table| table.get("include_profile"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_exclude_profile: sync_defaults
            .and_then(|table| table.get("exclude_profile"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        sync_default_clean_context: sync_defaults
            .and_then(|table| table.get("clean_context"))
            .and_then(TomlValue::as_bool),
        sync_default_allow_dirty: sync_defaults
            .and_then(|table| table.get("allow_dirty"))
            .and_then(TomlValue::as_bool),
        sync_default_include_containers_manifest: sync_defaults
            .and_then(|table| table.get("include_containers_manifest"))
            .and_then(TomlValue::as_bool),
        sync_default_data_manifest_glob: sync_defaults
            .and_then(|table| table.get("data_manifest_glob"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_ssh_host: remote
            .and_then(|table| table.get("ssh_host"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_repo_root: remote
            .and_then(|table| table.get("repo_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_cache_root: remote
            .and_then(|table| table.get("cache_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_corpus_root: remote
            .and_then(|table| table.get("corpus_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_results_root: remote
            .and_then(|table| table.get("results_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_extra_data_root: remote
            .and_then(|table| table.get("extra_data_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_reference_root: remote
            .and_then(|table| table.get("reference_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
        remote_containers_root: remote
            .and_then(|table| table.get("containers_root"))
            .and_then(TomlValue::as_str)
            .map(ToOwned::to_owned),
    })
}

fn load_benchmark_sync_profiles(path: &Path) -> Result<Vec<BenchmarkSyncProfile>> {
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let value: TomlValue = toml::from_str(&read_utf8(path)?)?;
    let profiles = value.get("profiles").and_then(TomlValue::as_array).cloned().unwrap_or_default();
    Ok(profiles
        .into_iter()
        .filter_map(|row| {
            Some(BenchmarkSyncProfile {
                name: row.get("name")?.as_str()?.to_string(),
                include_file: row
                    .get("include_file")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                exclude_file: row
                    .get("exclude_file")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                workspace_scope: row
                    .get("workspace_scope")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                pull_destination: row
                    .get("pull_destination")
                    .and_then(TomlValue::as_str)
                    .map(ToOwned::to_owned),
                remote_roots: row
                    .get("remote_roots")
                    .and_then(TomlValue::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect(),
                data_manifest_globs: row
                    .get("data_manifest_globs")
                    .and_then(TomlValue::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(|value| value.as_str().map(ToOwned::to_owned))
                    .collect(),
            })
        })
        .collect())
}

fn benchmark_sync_profile<'a>(
    profiles: &'a [BenchmarkSyncProfile],
    name: &str,
) -> Option<&'a BenchmarkSyncProfile> {
    profiles.iter().find(|profile| profile.name == name)
}

fn benchmark_corpus_dir_name(benchmark_workspace: &BenchmarkWorkspacePaths) -> String {
    benchmark_workspace
        .remote_corpus_root
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .and_then(|value| value.to_str())
        .filter(|value| !value.trim().is_empty())
        .map_or_else(|| "corpus".to_string(), ToOwned::to_owned)
}

fn benchmark_workspace_lookup<'a>(
    benchmark_workspace: &'a BenchmarkWorkspacePaths,
    key: &str,
) -> Option<&'a str> {
    match key {
        "local.results_root" => benchmark_workspace.local_results_root.as_deref(),
        "local.cache_mirror_root" => benchmark_workspace.local_cache_mirror_root.as_deref(),
        "remote.repo_root" => benchmark_workspace.remote_repo_root.as_deref(),
        "remote.cache_root" => benchmark_workspace.remote_cache_root.as_deref(),
        "remote.corpus_root" => benchmark_workspace.remote_corpus_root.as_deref(),
        "remote.results_root" => benchmark_workspace.remote_results_root.as_deref(),
        "remote.extra_data_root" => benchmark_workspace.remote_extra_data_root.as_deref(),
        "remote.reference_root" => benchmark_workspace.remote_reference_root.as_deref(),
        "remote.containers_root" => benchmark_workspace.remote_containers_root.as_deref(),
        _ => None,
    }
}

fn validate_benchmark_sync_roots(benchmark_workspace: &BenchmarkWorkspacePaths) -> Result<()> {
    let remote_repo_root = benchmark_workspace.remote_repo_root.as_deref().map(PathBuf::from);
    let remote_cache_root = benchmark_workspace.remote_cache_root.as_deref().map(PathBuf::from);
    let local_results_root = benchmark_workspace.local_results_root.as_deref().map(PathBuf::from);
    let local_cache_mirror_root =
        benchmark_workspace.local_cache_mirror_root.as_deref().map(PathBuf::from);

    if let (Some(repo_root), Some(cache_root)) = (&remote_repo_root, &remote_cache_root) {
        if repo_root == cache_root
            || repo_root.starts_with(cache_root)
            || cache_root.starts_with(repo_root)
        {
            return Err(anyhow!(
                "invalid benchmark sync contract: private frontend repo root {} and shared cache root {} must be separate trees",
                repo_root.display(),
                cache_root.display()
            ));
        }
    }

    if let (Some(results_root), Some(cache_mirror_root)) =
        (&local_results_root, &local_cache_mirror_root)
    {
        if !cache_mirror_root.starts_with(results_root) {
            return Err(anyhow!(
                "invalid benchmark sync contract: local cache mirror {} must live under local results root {}",
                cache_mirror_root.display(),
                results_root.display()
            ));
        }
    }
    Ok(())
}

fn default_pull_destination(
    explicit_destination: &str,
    configured_destination: Option<&str>,
    pull_base: &str,
    home: &str,
    use_governed_results_root: bool,
) -> PathBuf {
    if !explicit_destination.is_empty() {
        return PathBuf::from(expand_home_placeholder(explicit_destination, home));
    }
    if use_governed_results_root || configured_destination.is_some() {
        return PathBuf::from(expand_home_placeholder(
            configured_destination.unwrap_or(pull_base),
            home,
        ));
    }
    let timestamp = Utc::now().format("%Y%m%d-%H%M%S").to_string();
    PathBuf::from(expand_home_placeholder(pull_base, home))
        .join(format!("benchmark-sync-{timestamp}"))
}

fn benchmark_remote_layout_candidates(
    benchmark_workspace: &BenchmarkWorkspacePaths,
) -> Vec<(String, String)> {
    let mut candidates = Vec::new();
    if let Some(results_root) = benchmark_workspace.remote_results_root.as_deref() {
        candidates.push(("canonical-results-root".to_string(), results_root.to_string()));
    }
    if let Some(reference_root) = benchmark_workspace.remote_reference_root.as_deref() {
        candidates.push(("canonical-reference-root".to_string(), reference_root.to_string()));
    }
    if let Some(cache_root) = benchmark_workspace.remote_cache_root.as_deref() {
        let cache_path = Path::new(cache_root);
        if let Some(parent) = cache_path.parent() {
            for sibling in [
                "results".to_string(),
                benchmark_corpus_dir_name(benchmark_workspace),
                "extra-data".to_string(),
            ] {
                candidates.push((
                    format!("non-cache-sibling:{sibling}"),
                    parent.join(sibling).display().to_string(),
                ));
            }
            candidates.push((
                "legacy-reference-root".to_string(),
                parent.join("bijux-reference").display().to_string(),
            ));
        }
    }
    candidates
}

fn remote_layout_conflicts(
    workspace: &Workspace,
    host: &str,
    benchmark_workspace: &BenchmarkWorkspacePaths,
) -> Result<Vec<String>> {
    let mut conflicts = Vec::new();
    let candidates = benchmark_remote_layout_candidates(benchmark_workspace);
    for (label, path) in candidates {
        if label == "canonical-results-root" || label == "canonical-reference-root" {
            continue;
        }
        if remote_path_exists(workspace, host, &path)? {
            conflicts.push(format!("unexpected remote root {label} at {path}"));
        }
    }
    Ok(conflicts)
}

fn expand_home_placeholder(raw: &str, home: &str) -> String {
    raw.replace("${HOME}", home)
}

fn shell_single_quote(raw: &str) -> String {
    raw.replace('\'', "'\"'\"'")
}

fn remote_path_exists(workspace: &Workspace, host: &str, remote_path: &str) -> Result<bool> {
    let outcome = run_program(
        workspace,
        "ssh",
        &[host.to_string(), format!("test -e '{}'", shell_single_quote(remote_path))],
    )?;
    Ok(outcome.is_success())
}

fn mirror_remote_path(base: &Path, remote_path: &str) -> PathBuf {
    base.join(remote_path.trim_start_matches('/'))
}

fn pull_benchmark_sync_tree(
    workspace: &Workspace,
    host: &str,
    remote_dir: &str,
    dest_root: &Path,
) -> Result<PathBuf> {
    let local_dir = mirror_remote_path(dest_root, remote_dir);
    bijux_dna_infra::ensure_dir(&local_dir)?;
    let outcome = run_program(
        workspace,
        "rsync",
        &["-az".to_string(), format!("{host}:{remote_dir}/"), format!("{}/", local_dir.display())],
    )?;
    if !outcome.is_success() {
        return Err(anyhow!(
            "rsync failed while pulling {host}:{remote_dir}/ to {}/",
            local_dir.display()
        ));
    }
    Ok(local_dir)
}

fn pull_benchmark_sync_path(
    workspace: &Workspace,
    host: &str,
    remote_path: &str,
    dest_root: &Path,
) -> Result<PathBuf> {
    let local_path = mirror_remote_path(dest_root, remote_path);
    if let Some(parent) = local_path.parent() {
        bijux_dna_infra::ensure_dir(parent)?;
    }
    let outcome = run_program(
        workspace,
        "rsync",
        &["-az".to_string(), format!("{host}:{remote_path}"), local_path.display().to_string()],
    )?;
    if !outcome.is_success() {
        return Err(anyhow!(
            "rsync failed while pulling {host}:{remote_path} to {}",
            local_path.display()
        ));
    }
    Ok(local_path)
}

fn env_or_default(key: &str, fallback: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| fallback.to_string())
}

fn env_or_contract(key: &str, contract_value: Option<&str>, contract_key: &str) -> Result<String> {
    if let Ok(value) = std::env::var(key) {
        if !value.trim().is_empty() {
            return Ok(value);
        }
    }
    contract_value
        .filter(|value| !value.trim().is_empty())
        .map(ToOwned::to_owned)
        .ok_or_else(|| anyhow!("{key} or {contract_key} must be declared"))
}

fn sha256_hex(path: &Path) -> Result<String> {
    use sha2::{Digest, Sha256};
    let bytes = fs::read(path).with_context(|| format!("read {}", path.display()))?;
    Ok(Sha256::digest(bytes).iter().map(|byte| format!("{byte:02x}")).collect())
}

#[cfg(test)]
mod tests {
    use anyhow::Context;
    use std::path::PathBuf;
    use toml::Value as TomlValue;

    use super::{
        benchmark_corpus_dir_name, benchmark_sync_profile, benchmark_workspace_lookup,
        config_string, env_or_contract, expand_toml_env_placeholders, load_benchmark_sync_profiles,
        load_benchmark_workspace_paths, BenchmarkWorkspacePaths,
    };
    use crate::runtime::workspace::Workspace;

    #[test]
    fn load_benchmark_sync_profiles_reads_workspace_profile_fields() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-benchmark-sync-profiles")?;
        let path = temp.path().join("benchmark_sync_profiles.toml");
        bijux_dna_infra::write_bytes(
            &path,
            br#"
[[profiles]]
name = "pull-benchmark-publication"
include_file = "configs/hpc/rsync/pull-results-includes.txt"
workspace_scope = "benchmark-fastq-publication"
pull_destination = "local.results_root"
remote_roots = ["remote.results_root", "remote.extra_data_root"]
data_manifest_globs = ["benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv"]
"#,
        )?;

        let profiles = load_benchmark_sync_profiles(&path)?;
        let profile = benchmark_sync_profile(&profiles, "pull-benchmark-publication")
            .context("missing sync profile")?;

        assert_eq!(profile.workspace_scope.as_deref(), Some("benchmark-fastq-publication"));
        assert_eq!(profile.pull_destination.as_deref(), Some("local.results_root"));
        assert_eq!(profile.remote_roots, vec!["remote.results_root", "remote.extra_data_root"]);
        assert_eq!(
            profile.data_manifest_globs,
            vec![
                "benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv"
            ]
        );
        Ok(())
    }

    #[test]
    fn load_benchmark_workspace_paths_reads_unified_benchmark_contract() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-benchmark-workspace-paths")?;
        let config_dir = temp.path().join("configs/bench");
        std::fs::create_dir_all(&config_dir)?;
        bijux_dna_infra::write_bytes(
            config_dir.join("benchmark.toml"),
            br#"[workspace.local]
results_root = "/tmp/results"
cache_mirror_root = "/tmp/results/.cache"

[workspace.remote]
ssh_host = "cluster"
repo_root = "/opt/benchmark/repo"
cache_root = "/opt/benchmark/.cache"
corpus_root = "/opt/benchmark/.cache/benchmark_corpus"
results_root = "/opt/benchmark/.cache/results"
extra_data_root = "/opt/benchmark/.cache/extra-data"
reference_root = "/opt/benchmark/.cache/reference"
containers_root = "/opt/benchmark/.cache/containers"

[workspace.sync.defaults]
pull_base = "/tmp/pulls"
pull_mode = "results"
include_profile = "pull-results-default"
exclude_profile = "pull-full-default"
clean_context = true
allow_dirty = false
include_containers_manifest = false
data_manifest_glob = ""
"#,
        )?;

        let workspace = Workspace { root: temp.path().to_path_buf() };
        let paths = load_benchmark_workspace_paths(&workspace)?;

        assert_eq!(paths.local_results_root.as_deref(), Some("/tmp/results"));
        assert_eq!(paths.remote_repo_root.as_deref(), Some("/opt/benchmark/repo"));
        assert_eq!(paths.remote_results_root.as_deref(), Some("/opt/benchmark/.cache/results"));
        assert_eq!(paths.sync_default_pull_base.as_deref(), Some("/tmp/pulls"));
        Ok(())
    }

    #[test]
    fn config_string_reads_string_arrays_as_csv() {
        let value: TomlValue = toml::from_str("pipeline_ids = [\"one\", \"two\"]")
            .unwrap_or_else(|err| panic!("parse config: {err}"));
        assert_eq!(config_string(&value, "pipeline_ids"), Some("one,two".to_string()));
    }

    #[test]
    fn expand_toml_env_placeholders_expands_nested_strings() {
        let mut value: TomlValue = toml::from_str(
            r#"
corpus_root = "${BIJUX_TEST_CORPUS_ROOT}"
pipeline_ids = ["${BIJUX_TEST_PIPELINE_A}", "fixed"]
"#,
        )
        .unwrap_or_else(|err| panic!("parse config: {err}"));
        std::env::set_var("BIJUX_TEST_CORPUS_ROOT", "/tmp/corpus");
        std::env::set_var("BIJUX_TEST_PIPELINE_A", "pipe-a");
        expand_toml_env_placeholders(&mut value);
        std::env::remove_var("BIJUX_TEST_CORPUS_ROOT");
        std::env::remove_var("BIJUX_TEST_PIPELINE_A");

        assert_eq!(config_string(&value, "corpus_root"), Some("/tmp/corpus".to_string()));
        assert_eq!(config_string(&value, "pipeline_ids"), Some("pipe-a,fixed".to_string()));
    }

    #[test]
    fn benchmark_workspace_lookup_reads_governed_sync_roots() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: Some("/tmp/results".to_string()),
            local_cache_mirror_root: Some("/tmp/cache".to_string()),
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/repo".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: Some("/remote/.cache/benchmark_corpus".to_string()),
            remote_results_root: Some("/remote/.cache/results".to_string()),
            remote_extra_data_root: Some("/remote/.cache/extra-data".to_string()),
            remote_reference_root: Some("/remote/.cache/reference".to_string()),
            remote_containers_root: Some("/remote/.cache/bijux-dna-container".to_string()),
        };

        assert_eq!(
            benchmark_workspace_lookup(&workspace, "local.results_root"),
            Some("/tmp/results")
        );
        assert_eq!(
            benchmark_workspace_lookup(&workspace, "remote.extra_data_root"),
            Some("/remote/.cache/extra-data")
        );
        assert_eq!(
            benchmark_workspace_lookup(&workspace, "remote.reference_root"),
            Some("/remote/.cache/reference")
        );
    }

    #[test]
    fn validate_benchmark_sync_roots_rejects_overlapping_remote_roots() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: Some("/tmp/results".to_string()),
            local_cache_mirror_root: Some("/tmp/results/home/user/.cache".to_string()),
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/.cache/bijux-dna".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: None,
            remote_results_root: None,
            remote_extra_data_root: None,
            remote_reference_root: None,
            remote_containers_root: None,
        };

        let error = match super::validate_benchmark_sync_roots(&workspace) {
            Ok(()) => panic!("expected overlapping remote roots to fail"),
            Err(error) => error,
        };
        assert!(
            error.to_string().contains("private frontend repo root"),
            "unexpected error: {error}"
        );
    }

    #[test]
    fn validate_benchmark_sync_roots_requires_local_cache_mirror_under_results_root() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: Some("/tmp/results".to_string()),
            local_cache_mirror_root: Some("/tmp/cache".to_string()),
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/repo".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: None,
            remote_results_root: None,
            remote_extra_data_root: None,
            remote_reference_root: None,
            remote_containers_root: None,
        };

        let error = match super::validate_benchmark_sync_roots(&workspace) {
            Ok(()) => panic!("expected invalid local cache mirror to fail"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("local cache mirror"), "unexpected error: {error}");
    }

    #[test]
    fn benchmark_remote_layout_candidates_include_non_cache_roots() {
        let workspace = BenchmarkWorkspacePaths {
            local_results_root: None,
            local_cache_mirror_root: None,
            sync_default_pull_base: None,
            sync_default_pull_mode: None,
            sync_default_include_profile: None,
            sync_default_exclude_profile: None,
            sync_default_clean_context: None,
            sync_default_allow_dirty: None,
            sync_default_include_containers_manifest: None,
            sync_default_data_manifest_glob: None,
            remote_ssh_host: None,
            remote_repo_root: Some("/remote/repo".to_string()),
            remote_cache_root: Some("/remote/.cache".to_string()),
            remote_corpus_root: Some("/remote/.cache/benchmark_corpus".to_string()),
            remote_results_root: Some("/remote/.cache/results".to_string()),
            remote_extra_data_root: Some("/remote/.cache/extra-data".to_string()),
            remote_reference_root: Some("/remote/.cache/reference".to_string()),
            remote_containers_root: Some("/remote/.cache/bijux-dna-container".to_string()),
        };

        let candidates = super::benchmark_remote_layout_candidates(&workspace);

        assert!(candidates.iter().any(|(label, path)| {
            label == "legacy-reference-root" && path == "/remote/bijux-reference"
        }));
        assert!(candidates.iter().any(|(label, path)| {
            label == "non-cache-sibling:results" && path == "/remote/results"
        }));
        assert!(candidates.iter().any(|(label, path)| {
            label == "non-cache-sibling:benchmark_corpus" && path == "/remote/benchmark_corpus"
        }));
    }

    #[test]
    fn benchmark_corpus_dir_name_falls_back_to_generic_contract_name() {
        assert_eq!(benchmark_corpus_dir_name(&BenchmarkWorkspacePaths::default()), "corpus");
    }

    #[test]
    fn env_or_contract_requires_declared_value() {
        let error = match env_or_contract("BIJUX_TEST_MISSING", None, "workspace.remote.repo_root")
        {
            Ok(value) => panic!("missing contract must fail, got {value}"),
            Err(error) => error,
        };
        assert!(error
            .to_string()
            .contains("BIJUX_TEST_MISSING or workspace.remote.repo_root must be declared"));
    }

    #[test]
    fn default_pull_destination_prefers_governed_profile_destination() {
        let destination = super::default_pull_destination(
            "",
            Some("/tmp/results-archive"),
            "/tmp/fallback",
            "/home/operator",
            false,
        );

        assert_eq!(destination, PathBuf::from("/tmp/results-archive"));
    }

    #[test]
    fn default_pull_destination_uses_explicit_destination_when_present() {
        let destination = super::default_pull_destination(
            "${HOME}/custom-pull",
            Some("/tmp/results-archive"),
            "/tmp/fallback",
            "/home/operator",
            true,
        );

        assert_eq!(destination, PathBuf::from("/home/operator/custom-pull"));
    }
}
