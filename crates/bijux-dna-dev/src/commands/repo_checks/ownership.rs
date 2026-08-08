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
const OWNERSHIP_AUDIT_SOURCE: &str = "crates/bijux-dna-dev/src/commands/repo_checks/ownership.rs";
const ROOT_PYTHON_AUTHORITIES: [&str; 8] = [
    "benchmarks/corpus_acquisition.py",
    "benchmarks/corpus_catalog_enrichment.py",
    "benchmarks/corpus_materialization.py",
    "benchmarks/corpus_materialization_spec.py",
    "benchmarks/data_catalog.py",
    "containers/contracts/container_validation.py",
    "containers/runs/manage.py",
    "containers/runs/result_layout.py",
];
const CRATE_SCOPES: [&str; 11] = [
    "crates/bijux-dna-bench-model/",
    "crates/bijux-dna-bench/",
    "crates/bijux-dna-runtime/",
    "crates/bijux-dna-runner/",
    "crates/bijux-dna-containers/",
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
    legacy_authorities: Vec<LegacyAuthorityRow>,
}

#[derive(Debug, JsonSchema, Serialize)]
struct LegacyAuthorityRow {
    path: String,
    status: String,
    callers: Vec<String>,
    target_owner: String,
    replacement: String,
    removal_gate: String,
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
        audit.schema_version == "bijux.architecture.ownership_audit.v2",
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
        ensure!(row.classification != "unclassified", "{} is unclassified", row.path);
        ensure!(!row.current_owner.is_empty(), "{} has no current owner", row.path);
        ensure!(!row.target_owner.is_empty(), "{} has no target owner", row.path);
        ensure!(!row.replacement.is_empty(), "{} has no replacement contract", row.path);
        ensure!(!row.removal_gate.is_empty(), "{} has no removal gate", row.path);
        ensure!(!row.reasons.is_empty(), "{} has no classification reason", row.path);
        if is_live_root_python_authority(row) {
            ensure!(
                ROOT_PYTHON_AUTHORITIES.contains(&row.path.as_str()),
                "root Python authority is missing from the caller inventory: {}",
                row.path
            );
        }
    }
    ensure!(
        audit.summary.legacy_authorities.len() == ROOT_PYTHON_AUTHORITIES.len(),
        "legacy authority inventory must contain exactly {} rows",
        ROOT_PYTHON_AUTHORITIES.len()
    );
    let mut authority_paths = BTreeSet::new();
    for authority in &audit.summary.legacy_authorities {
        ensure!(
            authority_paths.insert(authority.path.as_str()),
            "duplicate legacy authority row for {}",
            authority.path
        );
        ensure!(
            ROOT_PYTHON_AUTHORITIES.contains(&authority.path.as_str()),
            "unknown legacy authority {}",
            authority.path
        );
        ensure!(!authority.target_owner.is_empty(), "{} has no target owner", authority.path);
        ensure!(!authority.replacement.is_empty(), "{} has no replacement", authority.path);
        ensure!(!authority.removal_gate.is_empty(), "{} has no removal gate", authority.path);
        let present = audit.files.iter().any(|row| row.path == authority.path);
        match authority.status.as_str() {
            "present" => {
                ensure!(present, "present legacy authority is absent: {}", authority.path);
                ensure!(
                    audit.files.iter().any(|row| {
                        row.path == authority.path && row.classification == "duplicate legacy logic"
                    }),
                    "present legacy authority is not classified as duplicate logic: {}",
                    authority.path
                );
            }
            "removed" => {
                ensure!(!present, "removed legacy authority is present: {}", authority.path);
                ensure!(
                    authority.callers.is_empty(),
                    "removed legacy authority still has executable callers: {}",
                    authority.path
                );
            }
            status => anyhow::bail!("invalid legacy authority status `{status}`"),
        }
    }
    ensure!(
        ROOT_PYTHON_AUTHORITIES.iter().all(|authority| authority_paths.contains(authority)),
        "legacy authority inventory is incomplete"
    );
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
        schema_version: "bijux.architecture.ownership_audit.v2",
        summary: AuditSummary {
            generated_from_commit: git_value(workspace, &["rev-parse", "HEAD"])?.trim().to_string(),
            total_files: rows.len(),
            unclassified_files: rows
                .iter()
                .filter(|row| row.classification == "unclassified")
                .count(),
            classification_counts,
            scope_counts,
            dependency_consumers: dependency_consumers(&files),
            legacy_authorities: legacy_authorities(&rows, callers),
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
        .filter(|path| workspace.path(path).is_file())
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
    let bridges_legacy_control_plane = ROOT_PYTHON_AUTHORITIES.iter().any(|authority| {
        file.path == *authority || references_legacy_authority(&file.path, &file.content, authority)
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
        || path.starts_with("benchmarks/readiness/")
        || path.contains("/simulation/")
        || path.contains("_simulation.")
    {
        return (
            "obsolete simulation/readiness logic",
            "readiness and simulation cannot substitute for admitted execution evidence",
        );
    }
    if path.starts_with("benchmarks/reports/") || path.starts_with("containers/reports/") {
        return (
            "generated artifact",
            "the file is a governed publication derived from canonical crate evidence",
        );
    }
    if path.starts_with("crates/bijux-dna-bench-model/")
        || path.starts_with("crates/bijux-dna-bench/")
        || path.starts_with("crates/bijux-dna-runtime/")
        || path.starts_with("crates/bijux-dna-runner/")
        || path.starts_with("crates/bijux-dna-containers/")
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
                "the file only translates an external process invocation to a crate-owned command",
            );
        }
        return (
            "duplicate legacy logic",
            "root Python must not own lifecycle, validation, admission, or publication decisions",
        );
    }
    if path.starts_with("benchmarks/execution/") {
        return (
            "thin external adapter",
            "the external tool adapter normalizes process I/O under crate-owned contracts",
        );
    }
    if path.starts_with(".github/workflows/") || path == "Makefile" || path.starts_with("makes/") {
        if ROOT_PYTHON_AUTHORITIES
            .iter()
            .any(|authority| references_legacy_authority(path, content, authority))
        {
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
    if is_declarative_asset(path) {
        return (
            "declarative product asset",
            "the file declares inputs, recipes, schemas, interfaces, or documentation",
        );
    }
    ("unclassified", "the ownership audit has no explicit rule for this product file")
}

fn is_declarative_asset(path: &str) -> bool {
    [
        "benchmarks/configs/",
        "benchmarks/contracts/",
        "benchmarks/corpora/",
        "benchmarks/runs/",
        "benchmarks/schemas/",
        "benchmarks/tests/fixtures/",
        "containers/apptainer/",
        "containers/contracts/",
        "containers/docker/",
        "containers/docs/",
        "containers/licenses/",
        "containers/network/",
        "containers/runs/",
        "containers/tools/",
        "containers/versions/",
    ]
    .iter()
    .any(|prefix| path.starts_with(prefix))
        || matches!(
            path,
            "benchmarks/README.md"
                | "benchmarks/tests/README.md"
                | "containers/README.md"
                | "containers/index.md"
                | "containers/TOOL_IDS.txt"
                | "crates/bijux-dna-api/Cargo.toml"
        )
        || path.starts_with("crates/") && path.ends_with("/Cargo.toml")
}

fn current_owner(path: &str) -> String {
    if let Some(rest) = path.strip_prefix("crates/") {
        return rest.split('/').next().unwrap_or("unknown").to_string();
    }
    path.split('/').next().unwrap_or("unknown").to_string()
}

fn target_owner(path: &str, classification: &str) -> String {
    if let Some(owner) = legacy_target_owner(path) {
        return owner.to_string();
    }
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
        return "bijux-dna-containers".to_string();
    }
    if path.contains("science") || path.contains("metric") || path.contains("truth") {
        return "bijux-dna-science".to_string();
    }
    "bijux-dna-bench and bijux-dna-bench-model".to_string()
}

fn legacy_target_owner(path: &str) -> Option<&'static str> {
    match path {
        "benchmarks/corpus_acquisition.py" => {
            Some("bijux-dna-bench, bijux-dna-runner, and data-source crates")
        }
        "benchmarks/corpus_catalog_enrichment.py" | "benchmarks/data_catalog.py" => {
            Some("bijux-dna-bench-model and bijux-dna-bench")
        }
        "benchmarks/corpus_materialization.py" => {
            Some("bijux-dna-bench, bijux-dna-runner, and bijux-dna-runtime")
        }
        "benchmarks/corpus_materialization_spec.py" => {
            Some("bijux-dna-bench and bijux-dna-runner")
        }
        "containers/contracts/container_validation.py" => Some(
            "bijux-dna-runner, bijux-dna-runtime, bijux-dna-containers-qa, and bijux-dna-stage-contract",
        ),
        "containers/runs/manage.py" => {
            Some("bijux-dna-containers and bijux-dna-runtime")
        }
        "containers/runs/result_layout.py" => {
            Some("bijux-dna-infra, bijux-dna-runtime, and bijux-dna-containers")
        }
        _ => None,
    }
}

fn scope_for(path: &str) -> String {
    CRATE_SCOPES.iter().find(|scope| path.starts_with(**scope)).map_or_else(
        || path.split('/').next().unwrap_or("repository").to_string(),
        |scope| scope.trim_end_matches('/').to_string(),
    )
}

fn legacy_callers(files: &[SourceFile]) -> BTreeMap<String, Vec<String>> {
    ROOT_PYTHON_AUTHORITIES
        .iter()
        .map(|authority| {
            let mut matches = files
                .iter()
                .filter(|file| file.path != OWNERSHIP_AUDIT_SOURCE)
                .filter(|file| is_executable_reference(&file.path))
                .filter(|file| references_legacy_authority(&file.path, &file.content, authority))
                .map(|file| file.path.clone())
                .collect::<Vec<_>>();
            matches.sort();
            matches.dedup();
            ((*authority).to_string(), matches)
        })
        .collect()
}

fn legacy_authorities(
    files: &[OwnershipRow],
    callers: BTreeMap<String, Vec<String>>,
) -> Vec<LegacyAuthorityRow> {
    ROOT_PYTHON_AUTHORITIES
        .iter()
        .map(|authority| {
            let present = files.iter().any(|row| row.path == *authority);
            let target_owner = target_owner(authority, "duplicate legacy logic");
            LegacyAuthorityRow {
                path: (*authority).to_string(),
                status: if present { "present" } else { "removed" }.to_string(),
                callers: callers.get(*authority).cloned().unwrap_or_default(),
                replacement: if present {
                    format!("replace with typed services owned by {target_owner}")
                } else {
                    format!("authority removed; retained behavior is owned by {target_owner}")
                },
                removal_gate: if present {
                    "remove after golden, negative, real-STUDY import, application, CLI, and replay parity; production caller search must be empty".to_string()
                } else {
                    "retain absence and zero executable callers".to_string()
                },
                target_owner,
            }
        })
        .collect()
}

fn references_legacy_authority(path: &str, content: &str, authority: &str) -> bool {
    let authority_path = Path::new(authority);
    let name = authority_path.file_name().and_then(|value| value.to_str()).unwrap_or(authority);
    if content.contains(authority) || content.contains(name) {
        return true;
    }
    if Path::new(path).extension().and_then(|value| value.to_str()) != Some("py") {
        return false;
    }
    authority_path.file_stem().and_then(|value| value.to_str()).is_some_and(|stem| {
        let qualified = authority.trim_end_matches(".py").replace('/', ".");
        content.lines().map(str::trim).any(|line| {
            line == format!("import {stem}")
                || line.starts_with(&format!("import {stem} as "))
                || line.starts_with(&format!("from {stem} import "))
                || line == format!("import {qualified}")
                || line.starts_with(&format!("import {qualified} as "))
                || line.starts_with(&format!("from {qualified} import "))
        })
    })
}

fn is_executable_reference(path: &str) -> bool {
    path == "Makefile"
        || path.starts_with("makes/")
        || path.starts_with(".github/workflows/")
        || matches!(
            Path::new(path).extension().and_then(|value| value.to_str()),
            Some("md" | "mk" | "py" | "rs" | "sh" | "yaml" | "yml")
        )
}

fn is_live_root_python_authority(row: &OwnershipRow) -> bool {
    (row.path.starts_with("benchmarks/") || row.path.starts_with("containers/"))
        && row.file_type == ".py"
        && row.classification == "duplicate legacy logic"
        && !row.path.contains("/tests/")
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
    for authority in &audit.summary.legacy_authorities {
        lines.push(format!(
            "- `{}`: `{}`, `{}` caller(s)",
            authority.path,
            authority.status,
            authority.callers.len()
        ));
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
        classify, legacy_authorities, legacy_callers, render_markdown, validate_audit,
        AuditSummary, OwnershipAudit, OwnershipRow, SourceFile, OWNERSHIP_AUDIT_SOURCE,
        ROOT_PYTHON_AUTHORITIES,
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
    fn legacy_authorities_have_explicit_removal_contracts() -> anyhow::Result<()> {
        let schema_pattern = Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?;
        for authority in ROOT_PYTHON_AUTHORITIES {
            let row = classify(
                &SourceFile {
                    path: authority.to_string(),
                    content: "SCHEMA = 'bijux.benchmark.registry.v1'".to_string(),
                },
                &schema_pattern,
                &BTreeMap::new(),
            );
            assert_eq!(row.classification, "duplicate legacy logic", "{authority}");
            assert!(row.removal_gate.contains("real-STUDY import"), "{authority}");
            assert_eq!(row.schema_refs, ["bijux.benchmark.registry.v1"], "{authority}");
        }
        Ok(())
    }

    #[test]
    fn root_execution_helpers_are_thin_adapters() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "benchmarks/execution/fastq_validate_reads.awk".to_string(),
                content: String::new(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert_eq!(row.classification, "thin external adapter");
        Ok(())
    }

    #[test]
    fn generated_publications_are_not_declarative_authority() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "benchmarks/reports/stage-benchmark-coverage.json".to_string(),
                content: String::new(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert_eq!(row.classification, "generated artifact");
        Ok(())
    }

    #[test]
    fn legacy_inventory_retains_removed_authorities_and_self_callers() {
        let callers = legacy_callers(&[
            SourceFile {
                path: "containers/contracts/container_validation.py".to_string(),
                content: "python3 containers/contracts/container_validation.py execute".to_string(),
            },
            SourceFile {
                path: "containers/README.md".to_string(),
                content: "python3 containers/runs/manage.py validate".to_string(),
            },
            SourceFile {
                path: OWNERSHIP_AUDIT_SOURCE.to_string(),
                content: "containers/runs/result_layout.py".to_string(),
            },
        ]);
        assert_eq!(callers.len(), ROOT_PYTHON_AUTHORITIES.len());
        assert_eq!(
            callers["containers/contracts/container_validation.py"],
            ["containers/contracts/container_validation.py"]
        );
        assert!(callers["benchmarks/data_catalog.py"].is_empty());
        assert!(callers["containers/runs/result_layout.py"].is_empty());
        assert_eq!(callers["containers/runs/manage.py"], ["containers/README.md"]);
    }

    #[test]
    fn legacy_authority_rows_distinguish_present_and_removed_files() {
        let files = vec![OwnershipRow {
            path: "containers/contracts/container_validation.py".to_string(),
            scope: "containers".to_string(),
            file_type: ".py".to_string(),
            classification: "duplicate legacy logic".to_string(),
            current_owner: "containers".to_string(),
            target_owner: "bijux-dna-containers".to_string(),
            callers: Vec::new(),
            replacement: "replacement".to_string(),
            removal_gate: "gate".to_string(),
            reasons: vec!["reason".to_string()],
            schema_refs: Vec::new(),
            bridges_legacy_control_plane: true,
        }];
        let callers =
            ROOT_PYTHON_AUTHORITIES.iter().map(|path| ((*path).to_string(), Vec::new())).collect();
        let rows = legacy_authorities(&files, callers);
        assert_eq!(rows.len(), ROOT_PYTHON_AUTHORITIES.len());
        assert_eq!(
            rows.iter()
                .find(|row| row.path == "containers/contracts/container_validation.py")
                .map(|row| row.status.as_str()),
            Some("present")
        );
        assert_eq!(
            rows.iter()
                .find(|row| row.path == "benchmarks/data_catalog.py")
                .map(|row| row.status.as_str()),
            Some("removed")
        );
    }

    #[test]
    fn unknown_product_paths_fail_closed() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "benchmarks/unknown-control-plane.rs".to_string(),
                content: String::new(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert_eq!(row.classification, "unclassified");
        Ok(())
    }

    #[test]
    fn rust_module_names_do_not_imply_legacy_script_calls() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "crates/bijux-dna-dev/src/commands/containers/image_storage.rs".to_string(),
                content: "fn image_storage() {}".to_string(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert!(!row.bridges_legacy_control_plane);
        Ok(())
    }

    #[test]
    fn similarly_named_python_adapters_do_not_imply_legacy_script_calls() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "benchmarks/adapters/tests/test_materialization.py".to_string(),
                content: "ADAPTERS / \"corpus_materialization_slurm.py\"".to_string(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert!(!row.bridges_legacy_control_plane);
        Ok(())
    }

    #[test]
    fn markdown_rendering_is_deterministic() {
        let audit = OwnershipAudit {
            schema_version: "bijux.architecture.ownership_audit.v2",
            summary: AuditSummary {
                generated_from_commit: "abc123".to_string(),
                total_files: 0,
                unclassified_files: 0,
                classification_counts: BTreeMap::new(),
                scope_counts: BTreeMap::new(),
                dependency_consumers: BTreeMap::new(),
                legacy_authorities: Vec::new(),
            },
            files: Vec::new(),
        };
        assert_eq!(render_markdown(&audit), render_markdown(&audit));
    }

    #[test]
    fn semantic_validation_rejects_an_empty_audit() {
        let audit = OwnershipAudit {
            schema_version: "bijux.architecture.ownership_audit.v2",
            summary: AuditSummary {
                generated_from_commit: "a".repeat(40),
                total_files: 0,
                unclassified_files: 0,
                classification_counts: BTreeMap::new(),
                scope_counts: BTreeMap::new(),
                dependency_consumers: BTreeMap::new(),
                legacy_authorities: Vec::new(),
            },
            files: Vec::new(),
        };
        assert!(validate_audit(&audit).is_err());
    }
}
