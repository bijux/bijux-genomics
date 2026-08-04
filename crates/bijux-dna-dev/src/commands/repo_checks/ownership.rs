use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use anyhow::{ensure, Context, Result};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json, ensure_dir};
use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;

use crate::commands::command_support::{pass, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

const OUTPUT_JSON: &str = "artifacts/architecture/benchmark-container-ownership-audit.json";
const OUTPUT_MARKDOWN: &str = "artifacts/architecture/benchmark-container-ownership-audit.md";
const OUTPUT_SCHEMA: &str =
    "artifacts/architecture/benchmark-container-ownership-audit.schema.json";
const LEGACY_AUTHORITIES: [&str; 4] = [
    "benchmarks/benchmark_evidence.py",
    "benchmarks/benchmark_portfolio.py",
    "benchmarks/benchmark_registry.py",
    "containers/contracts/container_validation.py",
];
const CRATE_SCOPES: [&str; 11] = [
    "crates/bijux-dna-bench-model/",
    "crates/bijux-dna-bench/",
    "crates/bijux-dna-runtime/",
    "crates/bijux-dna-runner/",
    "crates/bijux-dna-environment/",
    "crates/bijux-dna-stage-contract/",
    "crates/bijux-dna-science/",
    "crates/bijux-dna-policies/",
    "crates/bijux-dna-infra/",
    "crates/bijux-dna-dev/src/commands/containers/",
    "crates/bijux-dna/src/commands/benchmark/",
];

#[derive(Debug)]
struct SourceFile {
    path: String,
    content: String,
}

#[derive(Debug, JsonSchema, Serialize)]
struct OwnershipAudit {
    schema_version: &'static str,
    summary: AuditSummary,
    files: Vec<OwnershipRow>,
}

#[derive(Debug, JsonSchema, Serialize)]
struct AuditSummary {
    generated_from_commit: String,
    total_files: usize,
    unclassified_files: usize,
    classification_counts: BTreeMap<String, usize>,
    scope_counts: BTreeMap<String, usize>,
    dependency_consumers: BTreeMap<String, Vec<String>>,
    legacy_authority_callers: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, JsonSchema, Serialize)]
struct OwnershipRow {
    path: String,
    scope: String,
    file_type: String,
    classification: String,
    current_owner: String,
    target_owner: String,
    callers: Vec<String>,
    replacement: String,
    removal_gate: String,
    reasons: Vec<String>,
    schema_refs: Vec<String>,
    bridges_legacy_control_plane: bool,
}

pub(crate) fn check_ownership_audit(
    workspace: &Workspace,
    check: &CheckDefinition,
) -> Result<CheckOutcome> {
    let audit = build_audit(workspace)?;
    validate_audit(&audit)?;
    let markdown = render_markdown(&audit);
    let schema = schemars::schema_for!(OwnershipAudit);
    let output_dir = workspace.path("artifacts/architecture");
    ensure_dir(&output_dir).context("create architecture artifact root")?;
    atomic_write_json(&workspace.path(OUTPUT_JSON), &audit)
        .context("write ownership audit JSON")?;
    atomic_write_bytes(&workspace.path(OUTPUT_MARKDOWN), markdown.as_bytes())
        .context("write ownership audit Markdown")?;
    atomic_write_json(&workspace.path(OUTPUT_SCHEMA), &schema)
        .context("write ownership audit schema")?;
    pass(
        check,
        format!(
            "classified {} files at {} with zero unclassified files; wrote {}, {}, and {}",
            audit.summary.total_files,
            audit.summary.generated_from_commit,
            OUTPUT_JSON,
            OUTPUT_MARKDOWN,
            OUTPUT_SCHEMA
        ),
    )
}

fn validate_audit(audit: &OwnershipAudit) -> Result<()> {
    ensure!(
        audit.schema_version == "bijux.architecture.ownership_audit.v1",
        "unexpected ownership audit schema version"
    );
    ensure!(
        audit.summary.generated_from_commit.len() == 40
            && audit.summary.generated_from_commit.chars().all(|value| value.is_ascii_hexdigit()),
        "ownership audit source commit must be a full Git SHA"
    );
    ensure!(!audit.files.is_empty(), "ownership audit contains no files");
    ensure!(
        audit.summary.total_files == audit.files.len(),
        "ownership audit total does not match file rows"
    );
    ensure!(audit.summary.unclassified_files == 0, "ownership audit contains unclassified files");
    ensure!(
        audit.summary.classification_counts.values().sum::<usize>() == audit.files.len(),
        "ownership classification counts do not match file rows"
    );
    ensure!(
        audit.summary.scope_counts.values().sum::<usize>() == audit.files.len(),
        "ownership scope counts do not match file rows"
    );
    let mut paths = BTreeSet::new();
    for row in &audit.files {
        ensure!(!row.path.is_empty(), "ownership row has an empty path");
        ensure!(paths.insert(&row.path), "duplicate ownership row for {}", row.path);
        ensure!(!row.scope.is_empty(), "{} has no scope", row.path);
        ensure!(!row.classification.is_empty(), "{} is unclassified", row.path);
        ensure!(!row.current_owner.is_empty(), "{} has no current owner", row.path);
        ensure!(!row.target_owner.is_empty(), "{} has no target owner", row.path);
        ensure!(!row.replacement.is_empty(), "{} has no replacement contract", row.path);
        ensure!(!row.removal_gate.is_empty(), "{} has no removal gate", row.path);
        ensure!(!row.reasons.is_empty(), "{} has no classification reason", row.path);
    }
    for authority in LEGACY_AUTHORITIES {
        ensure!(
            audit.summary.legacy_authority_callers.contains_key(authority),
            "legacy authority caller inventory missing {authority}"
        );
        ensure!(
            audit.files.iter().any(|row| row.path == authority),
            "legacy authority row missing {authority}"
        );
    }
    let bench_consumers = audit
        .summary
        .dependency_consumers
        .get("bijux-dna-bench")
        .context("missing bijux-dna-bench dependency inventory")?;
    ensure!(
        bench_consumers == &["bijux-dna-api".to_string(), "bijux-dna-dev".to_string()],
        "unexpected bijux-dna-bench consumers: {bench_consumers:?}"
    );
    Ok(())
}

fn build_audit(workspace: &Workspace) -> Result<OwnershipAudit> {
    let files = tracked_source_files(workspace)?;
    let schema_pattern = Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")
        .context("compile schema identity pattern")?;
    let callers = legacy_callers(&files);
    let mut rows =
        files.iter().map(|file| classify(file, &schema_pattern, &callers)).collect::<Vec<_>>();
    rows.sort_by(|left, right| left.path.cmp(&right.path));

    let mut classification_counts = BTreeMap::new();
    let mut scope_counts = BTreeMap::new();
    for row in &rows {
        *classification_counts.entry(row.classification.clone()).or_insert(0) += 1;
        *scope_counts.entry(row.scope.clone()).or_insert(0) += 1;
    }

    Ok(OwnershipAudit {
        schema_version: "bijux.architecture.ownership_audit.v1",
        summary: AuditSummary {
            generated_from_commit: git_value(workspace, &["rev-parse", "HEAD"])?.trim().to_string(),
            total_files: rows.len(),
            unclassified_files: 0,
            classification_counts,
            scope_counts,
            dependency_consumers: dependency_consumers(&files),
            legacy_authority_callers: callers,
        },
        files: rows,
    })
}

fn tracked_source_files(workspace: &Workspace) -> Result<Vec<SourceFile>> {
    let output = run_command(workspace, "git", &["ls-files", "-z"])?;
    let mut files = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .filter_map(|bytes| std::str::from_utf8(bytes).ok())
        .filter(|path| is_relevant(path))
        .map(|path| {
            let content = std::fs::read_to_string(workspace.path(path))
                .with_context(|| format!("read ownership source {path}"))?;
            Ok(SourceFile { path: path.to_string(), content })
        })
        .collect::<Result<Vec<_>>>()?;
    files.sort_by(|left, right| left.path.cmp(&right.path));
    Ok(files)
}

fn is_relevant(path: &str) -> bool {
    let source_extension = Path::new(path).extension().and_then(|value| value.to_str());
    let supported = matches!(
        source_extension,
        Some(
            "def"
                | "json"
                | "md"
                | "mk"
                | "py"
                | "rs"
                | "sh"
                | "toml"
                | "tsv"
                | "txt"
                | "yaml"
                | "yml"
        )
    );
    if !supported {
        return false;
    }
    if path.starts_with("benchmarks/") || path.starts_with("containers/") {
        return true;
    }
    if CRATE_SCOPES.iter().any(|scope| path.starts_with(scope)) {
        return true;
    }
    if path.starts_with("crates/") && path.ends_with("/Cargo.toml") {
        return true;
    }
    path.starts_with("crates/bijux-dna-api/") && path.contains("bench")
        || path.starts_with(".github/workflows/")
        || path == "Makefile"
        || path.starts_with("makes/")
}

fn classify(
    file: &SourceFile,
    schema_pattern: &Regex,
    callers: &BTreeMap<String, Vec<String>>,
) -> OwnershipRow {
    let (classification, reason) = classification_for(&file.path, &file.content);
    let current_owner = current_owner(&file.path);
    let target_owner = target_owner(&file.path, classification);
    let authority_callers = callers.get(&file.path).cloned().unwrap_or_default();
    let bridges_legacy_control_plane = LEGACY_AUTHORITIES.iter().any(|authority| {
        file.path == *authority
            || file.content.contains(authority)
            || Path::new(authority)
                .file_stem()
                .and_then(|value| value.to_str())
                .is_some_and(|stem| file.content.contains(stem))
    });
    let schema_refs = schema_pattern
        .find_iter(&file.content)
        .map(|value| value.as_str().to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    let removable =
        matches!(classification, "duplicate legacy logic" | "obsolete simulation/readiness logic");

    OwnershipRow {
        path: file.path.clone(),
        scope: scope_for(&file.path),
        file_type: Path::new(&file.path)
            .extension()
            .and_then(|value| value.to_str())
            .map_or_else(|| "none".to_string(), |value| format!(".{value}")),
        classification: classification.to_string(),
        current_owner,
        target_owner: target_owner.clone(),
        callers: authority_callers,
        replacement: if removable {
            format!("replace with typed services owned by {target_owner}")
        } else {
            "retain in its declared ownership boundary".to_string()
        },
        removal_gate: if removable {
            "remove after golden, negative, real-STUDY import, application, CLI, and replay parity; production caller search must be empty".to_string()
        } else {
            "not removable; validate ownership and contract drift".to_string()
        },
        reasons: vec![reason.to_string()],
        schema_refs,
        bridges_legacy_control_plane,
    }
}

fn classification_for(path: &str, content: &str) -> (&'static str, &'static str) {
    if path.starts_with("crates/bijux-dna/src/commands/benchmark/readiness/")
        || path.contains("simulation")
        || path.contains("readiness")
    {
        return (
            "obsolete simulation/readiness logic",
            "readiness and simulation cannot substitute for admitted execution evidence",
        );
    }
    if path.starts_with("crates/bijux-dna-bench-model/")
        || path.starts_with("crates/bijux-dna-bench/")
        || path.starts_with("crates/bijux-dna-runtime/")
        || path.starts_with("crates/bijux-dna-runner/")
        || path.starts_with("crates/bijux-dna-environment/")
        || path.starts_with("crates/bijux-dna-stage-contract/")
        || path.starts_with("crates/bijux-dna-science/")
        || path.starts_with("crates/bijux-dna-policies/")
        || path.starts_with("crates/bijux-dna-infra/")
    {
        return (
            "canonical crate logic",
            "the file resides in the crate assigned this product responsibility",
        );
    }
    if path.starts_with("crates/bijux-dna-dev/src/commands/containers/") {
        if path.ends_with("/mod.rs")
            || path.ends_with("/dispatch.rs")
            || path.ends_with("/command_support.rs")
        {
            return (
                "thin external adapter",
                "the development command surface may compose typed crate services",
            );
        }
        return (
            "duplicate legacy logic",
            "container semantics in the composition root must move to their owning crates",
        );
    }
    if path.starts_with("crates/bijux-dna/src/commands/benchmark/")
        || path.starts_with("crates/bijux-dna-api/")
    {
        if path.ends_with("/mod.rs") || path.ends_with("/dispatch.rs") || path.contains("public") {
            return (
                "thin external adapter",
                "the public command/API surface may delegate to benchmark services",
            );
        }
        return (
            "duplicate legacy logic",
            "benchmark semantics outside benchmark crates duplicate the product authority",
        );
    }
    if path.ends_with(".py") {
        if path.starts_with("benchmarks/runs/") || path.contains("/adapters/") {
            return (
                "thin external adapter",
                "the file is an exceptional process adapter and must remain contract-bound",
            );
        }
        return (
            "duplicate legacy logic",
            "root Python must not own lifecycle, validation, admission, or publication decisions",
        );
    }
    if path.starts_with(".github/workflows/") || path == "Makefile" || path.starts_with("makes/") {
        if LEGACY_AUTHORITIES.iter().any(|authority| {
            content.contains(authority)
                || Path::new(authority)
                    .file_name()
                    .and_then(|value| value.to_str())
                    .is_some_and(|name| content.contains(name))
        }) {
            return (
                "duplicate legacy logic",
                "automation still invokes a legacy product authority",
            );
        }
        return (
            "thin external adapter",
            "automation may invoke stable product commands without owning their decisions",
        );
    }
    (
        "declarative product asset",
        "the file declares inputs, recipes, schemas, interfaces, documentation, or publications",
    )
}

fn current_owner(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("crates/") {
        return rest.split('/').next().unwrap_or("unknown").to_string();
    }
    path.split('/').next().unwrap_or("unknown").to_string()
}

fn target_owner(path: &str, classification: &str) -> String {
    if classification == "canonical crate logic" || classification == "declarative product asset" {
        return current_owner(path);
    }
    if path.contains("security") || path.contains("release") || path.contains("qualification") {
        return "bijux-dna-policies with bijux-dna-dev composition".to_string();
    }
    if path.contains("slurm") || path.contains("campaign") || path.contains("validation") {
        return "bijux-dna-runner and bijux-dna-runtime".to_string();
    }
    if path.starts_with("containers/") || path.contains("image") || path.contains("container") {
        return "bijux-dna-environment".to_string();
    }
    if path.contains("science") || path.contains("metric") || path.contains("truth") {
        return "bijux-dna-science".to_string();
    }
    "bijux-dna-bench and bijux-dna-bench-model".to_string()
}

fn scope_for(path: &str) -> String {
    CRATE_SCOPES.iter().find(|scope| path.starts_with(**scope)).map_or_else(
        || path.split('/').next().unwrap_or("repository").to_string(),
        |scope| scope.trim_end_matches('/').to_string(),
    )
}

fn legacy_callers(files: &[SourceFile]) -> BTreeMap<String, Vec<String>> {
    LEGACY_AUTHORITIES
        .iter()
        .map(|authority| {
            let stem = Path::new(authority)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or(authority);
            let name = Path::new(authority)
                .file_name()
                .and_then(|value| value.to_str())
                .unwrap_or(authority);
            let mut matches = files
                .iter()
                .filter(|file| file.path != *authority)
                .filter(|file| is_executable_reference(&file.path))
                .filter(|file| {
                    file.content.contains(authority)
                        || file.content.contains(name)
                        || file.content.contains(stem)
                })
                .map(|file| file.path.clone())
                .collect::<Vec<_>>();
            matches.sort();
            matches.dedup();
            ((*authority).to_string(), matches)
        })
        .collect()
}

fn is_executable_reference(path: &str) -> bool {
    path == "Makefile"
        || path.starts_with("makes/")
        || path.starts_with(".github/workflows/")
        || matches!(
            Path::new(path).extension().and_then(|value| value.to_str()),
            Some("mk" | "py" | "rs" | "sh" | "yaml" | "yml")
        )
}

fn dependency_consumers(files: &[SourceFile]) -> BTreeMap<String, Vec<String>> {
    ["bijux-dna-bench", "bijux-dna-bench-model"]
        .into_iter()
        .map(|dependency| {
            let consumers = files
                .iter()
                .filter(|file| {
                    file.path.ends_with("Cargo.toml")
                        && file.content.lines().any(|line| {
                            line.trim_start().strip_prefix(dependency).is_some_and(|suffix| {
                                suffix.starts_with(" =") || suffix.starts_with(".workspace")
                            })
                        })
                })
                .map(|file| current_owner(&file.path))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect();
            (dependency.to_string(), consumers)
        })
        .collect()
}

fn git_value(workspace: &Workspace, args: &[&str]) -> Result<String> {
    let output = run_command(workspace, "git", args)?;
    if !output.status.success() {
        anyhow::bail!("git {} failed", args.join(" "));
    }
    String::from_utf8(output.stdout).context("git output is not UTF-8")
}

fn render_markdown(audit: &OwnershipAudit) -> String {
    let mut lines = vec![
        "# Benchmark And Container Ownership Audit".to_string(),
        String::new(),
        "<!-- GENERATED FILE - DO NOT EDIT -->".to_string(),
        String::new(),
        format!("- Source commit: `{}`", audit.summary.generated_from_commit),
        format!("- Files audited: `{}`", audit.summary.total_files),
        format!("- Unclassified files: `{}`", audit.summary.unclassified_files),
        String::new(),
        "## Classification Counts".to_string(),
        String::new(),
    ];
    for (classification, count) in &audit.summary.classification_counts {
        lines.push(format!("- `{classification}`: `{count}`"));
    }
    lines.extend([String::new(), "## Legacy Authority Callers".to_string(), String::new()]);
    for (authority, callers) in &audit.summary.legacy_authority_callers {
        lines.push(format!("- `{authority}`: `{}` caller(s)", callers.len()));
    }
    lines.extend([
        String::new(),
        "The JSON artifact is authoritative and records every file, owner, caller, replacement,"
            .to_string(),
        "removal gate, schema identity, and classification reason.".to_string(),
        String::new(),
    ]);
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use regex::Regex;

    use super::{
        classify, render_markdown, validate_audit, AuditSummary, OwnershipAudit, SourceFile,
    };

    #[test]
    fn authoritative_crate_logic_remains_canonical() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "crates/bijux-dna-bench/src/workflow/mod.rs".to_string(),
                content: String::new(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert_eq!(row.classification, "canonical crate logic");
        assert_eq!(row.target_owner, "bijux-dna-bench");
        Ok(())
    }

    #[test]
    fn legacy_authority_has_explicit_removal_contract() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "benchmarks/benchmark_registry.py".to_string(),
                content: "SCHEMA = 'bijux.benchmark.registry.v1'".to_string(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert_eq!(row.classification, "duplicate legacy logic");
        assert!(row.removal_gate.contains("real-STUDY import"));
        assert_eq!(row.schema_refs, ["bijux.benchmark.registry.v1"]);
        Ok(())
    }

    #[test]
    fn markdown_rendering_is_deterministic() {
        let audit = OwnershipAudit {
            schema_version: "bijux.architecture.ownership_audit.v1",
            summary: AuditSummary {
                generated_from_commit: "abc123".to_string(),
                total_files: 0,
                unclassified_files: 0,
                classification_counts: BTreeMap::new(),
                scope_counts: BTreeMap::new(),
                dependency_consumers: BTreeMap::new(),
                legacy_authority_callers: BTreeMap::new(),
            },
            files: Vec::new(),
        };
        assert_eq!(render_markdown(&audit), render_markdown(&audit));
    }

    #[test]
    fn semantic_validation_rejects_an_empty_audit() {
        let audit = OwnershipAudit {
            schema_version: "bijux.architecture.ownership_audit.v1",
            summary: AuditSummary {
                generated_from_commit: "a".repeat(40),
                total_files: 0,
                unclassified_files: 0,
                classification_counts: BTreeMap::new(),
                scope_counts: BTreeMap::new(),
                dependency_consumers: BTreeMap::new(),
                legacy_authority_callers: BTreeMap::new(),
            },
            files: Vec::new(),
        };
        assert!(validate_audit(&audit).is_err());
    }
}
