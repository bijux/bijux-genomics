use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use aho_corasick::AhoCorasick;
use anyhow::{ensure, Context, Result};
use bijux_dna_infra::{atomic_write_bytes, atomic_write_json, ensure_dir};
use regex::Regex;
use schemars::JsonSchema;
use serde::Serialize;

use crate::commands::command_support::{pass, run_command};
use crate::model::check::{CheckDefinition, CheckOutcome};
use crate::runtime::workspace::Workspace;

const OUTPUT_JSON: &str = "artifacts/architecture/container-benchmark-ownership.json";
const OUTPUT_MARKDOWN: &str = "artifacts/architecture/container-benchmark-ownership.md";
const OUTPUT_SCHEMA: &str = "artifacts/architecture/container-benchmark-ownership.schema.json";
const OWNERSHIP_AUDIT_SOURCE: &str = "crates/bijux-dna-dev/src/commands/repo_checks/ownership.rs";
const DIRECT_BENCH_PRODUCT_CONSUMERS: [&str; 2] = ["bijux-dna-api", "bijux-dna-dev"];
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

#[derive(Clone, Debug)]
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
    unknown_owner_files: usize,
    classification_counts: BTreeMap<String, usize>,
    scope_counts: BTreeMap<String, usize>,
    direct_caller_edges: usize,
    indirect_caller_edges: usize,
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
    direct_callers: Vec<String>,
    indirect_callers: Vec<String>,
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
        audit.schema_version == "bijux.architecture.ownership_audit.v3",
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
    ensure!(audit.summary.unknown_owner_files == 0, "ownership audit contains unknown owners");
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
        ensure!(
            row.direct_callers.windows(2).all(|pair| pair[0] < pair[1]),
            "{} direct callers are not sorted and unique",
            row.path
        );
        ensure!(
            row.indirect_callers.windows(2).all(|pair| pair[0] < pair[1]),
            "{} indirect callers are not sorted and unique",
            row.path
        );
        ensure!(
            row.direct_callers.iter().all(|caller| !row.indirect_callers.contains(caller)),
            "{} direct and indirect callers overlap",
            row.path
        );
        let mut complete_callers = row.direct_callers.clone();
        complete_callers.extend(row.indirect_callers.iter().cloned());
        complete_callers.sort();
        ensure!(
            row.callers == complete_callers,
            "{} complete caller closure is inconsistent",
            row.path
        );
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
    ensure!(
        audit.summary.direct_caller_edges
            == audit.files.iter().map(|row| row.direct_callers.len()).sum::<usize>(),
        "direct caller edge count does not match file rows"
    );
    ensure!(
        audit.summary.indirect_caller_edges
            == audit.files.iter().map(|row| row.indirect_callers.len()).sum::<usize>(),
        "indirect caller edge count does not match file rows"
    );
    let bench_consumers = audit
        .summary
        .dependency_consumers
        .get("bijux-dna-bench")
        .context("missing bijux-dna-bench dependency inventory")?;
    ensure!(
        bench_consumers
            == &DIRECT_BENCH_PRODUCT_CONSUMERS.into_iter().map(str::to_string).collect::<Vec<_>>(),
        "unexpected bijux-dna-bench consumers: {bench_consumers:?}"
    );
    Ok(())
}

fn build_audit(workspace: &Workspace) -> Result<OwnershipAudit> {
    let repository_files = tracked_repository_files(workspace)?;
    let files = repository_files.iter().filter(|file| is_relevant(&file.path)).collect::<Vec<_>>();
    let schema_pattern = Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")
        .context("compile schema identity pattern")?;
    let legacy_callers = legacy_callers(&repository_files);
    let caller_graph = production_callers(&repository_files)?;
    let mut rows =
        files.iter().map(|file| classify(file, &schema_pattern, &caller_graph)).collect::<Vec<_>>();
    rows.sort_by(|left, right| left.path.cmp(&right.path));

    let mut classification_counts = BTreeMap::new();
    let mut scope_counts = BTreeMap::new();
    for row in &rows {
        *classification_counts.entry(row.classification.clone()).or_insert(0) += 1;
        *scope_counts.entry(row.scope.clone()).or_insert(0) += 1;
    }

    Ok(OwnershipAudit {
        schema_version: "bijux.architecture.ownership_audit.v3",
        summary: AuditSummary {
            generated_from_commit: git_value(workspace, &["rev-parse", "HEAD"])?.trim().to_string(),
            total_files: rows.len(),
            unclassified_files: rows
                .iter()
                .filter(|row| row.classification == "unclassified")
                .count(),
            unknown_owner_files: rows
                .iter()
                .filter(|row| row.current_owner == "unknown" || row.target_owner == "unknown")
                .count(),
            classification_counts,
            scope_counts,
            direct_caller_edges: rows.iter().map(|row| row.direct_callers.len()).sum(),
            indirect_caller_edges: rows.iter().map(|row| row.indirect_callers.len()).sum(),
            dependency_consumers: dependency_consumers(&repository_files),
            legacy_authorities: legacy_authorities(&rows, legacy_callers),
        },
        files: rows,
    })
}

fn tracked_repository_files(workspace: &Workspace) -> Result<Vec<SourceFile>> {
    let output = run_command(workspace, "git", &["ls-files", "-z"])?;
    let mut files = output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|path| !path.is_empty())
        .filter_map(|bytes| std::str::from_utf8(bytes).ok())
        .filter(|path| is_supported_file(path))
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
    if !is_supported_file(path) {
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

fn is_supported_file(path: &str) -> bool {
    let source_extension = Path::new(path).extension().and_then(|value| value.to_str());
    matches!(
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
    )
}

fn classify(
    file: &SourceFile,
    schema_pattern: &Regex,
    caller_graph: &BTreeMap<String, CallerClosure>,
) -> OwnershipRow {
    let (classification, reason) = classification_for(&file.path, &file.content);
    let current_owner = current_owner(&file.path);
    let target_owner = target_owner(&file.path, classification);
    let caller_closure = caller_graph.get(&file.path).cloned().unwrap_or_default();
    let mut callers = caller_closure.direct.clone();
    callers.extend(caller_closure.indirect.iter().cloned());
    callers.sort();
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
        callers,
        direct_callers: caller_closure.direct,
        indirect_callers: caller_closure.indirect,
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
    if is_test_source(path) {
        return (
            "test contract",
            "test-only verification may exercise product contracts but cannot own production semantics or evidence",
        );
    }
    if path.starts_with("crates/") && path.ends_with("/Cargo.toml") {
        return (
            "declarative product asset",
            "the manifest declares package composition without owning product lifecycle semantics",
        );
    }
    if path.starts_with("crates/bijux-dna/src/commands/benchmark/readiness/")
        && path.ends_with("/mod.rs")
    {
        return (
            "thin external adapter",
            "the readiness namespace declares modules without owning their product decisions",
        );
    }
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
            || is_thin_container_operator(path, content)
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
        if path.ends_with("/mod.rs")
            || path.ends_with("/dispatch.rs")
            || path.contains("public")
            || matches!(
                path,
                "crates/bijux-dna/src/commands/benchmark/local_vcf_stage_catalog.rs"
                    | "crates/bijux-dna/src/commands/benchmark/local_vcf_stage_matrix.rs"
            )
        {
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

fn is_thin_container_operator(path: &str, content: &str) -> bool {
    const MIGRATED_SURFACES: &[&str] = &[
        "crates/bijux-dna-dev/src/commands/containers/external_artifact.rs",
        "crates/bijux-dna-dev/src/commands/containers/image_activation.rs",
        "crates/bijux-dna-dev/src/commands/containers/qualification_cycle.rs",
        "crates/bijux-dna-dev/src/commands/containers/qualification_slurm.rs",
    ];
    const FORBIDDEN_OPERATOR_PRIMITIVES: &[&str] = &[
        "use std::fs",
        "std::fs::",
        "use std::process",
        "Command::",
        "ProcessRunner",
        "chrono::Utc::now",
        "serde_json::from_",
        "toml::",
        "read_utf8(",
        "write_utf8(",
        "atomic_write",
        "create_dir_all(",
        "remove_file(",
        "rename(",
    ];
    MIGRATED_SURFACES.contains(&path)
        && content.contains("bijux_dna_")
        && content.contains("::api::")
        && !FORBIDDEN_OPERATOR_PRIMITIVES.iter().any(|primitive| content.contains(primitive))
}

fn is_test_source(path: &str) -> bool {
    path.starts_with("tests/")
        || path.contains("/tests/")
        || path.ends_with("_test.py")
        || path.ends_with("_tests.rs")
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

#[derive(Clone, Debug, Default)]
struct CallerClosure {
    direct: Vec<String>,
    indirect: Vec<String>,
}

#[derive(Clone, Copy, Debug)]
enum ReferenceScope {
    Any,
    SameOwner,
    SiblingModule,
}

#[derive(Clone, Copy, Debug)]
enum ReferenceSyntax {
    None,
    RustModule,
    RustSourcePath,
    PythonModule,
}

#[derive(Clone, Debug)]
struct ReferenceTarget {
    path: String,
    scope: ReferenceScope,
    syntax: ReferenceSyntax,
}

fn production_callers(files: &[SourceFile]) -> Result<BTreeMap<String, CallerClosure>> {
    let mut references = BTreeMap::<String, Vec<ReferenceTarget>>::new();
    for target in files {
        references.entry(target.path.clone()).or_default().push(ReferenceTarget {
            path: target.path.clone(),
            scope: ReferenceScope::Any,
            syntax: ReferenceSyntax::None,
        });
        let rust_module =
            (!is_namespace_module(&target.path)).then(|| rust_module_path(&target.path)).flatten();
        if let Some(module_path) = rust_module {
            references.entry(module_path.clone()).or_default().push(ReferenceTarget {
                path: target.path.clone(),
                scope: ReferenceScope::SameOwner,
                syntax: if module_path.contains("::") {
                    ReferenceSyntax::None
                } else {
                    ReferenceSyntax::RustModule
                },
            });
            let crate_module =
                format!("{}::{module_path}", current_owner(&target.path).replace('-', "_"));
            references.entry(crate_module).or_default().push(ReferenceTarget {
                path: target.path.clone(),
                scope: ReferenceScope::Any,
                syntax: ReferenceSyntax::None,
            });
            if let Some(module_name) = module_path.rsplit("::").next() {
                references.entry(module_name.to_string()).or_default().push(ReferenceTarget {
                    path: target.path.clone(),
                    scope: ReferenceScope::SiblingModule,
                    syntax: ReferenceSyntax::RustModule,
                });
            }
            if let Some(file_name) =
                Path::new(&target.path).file_name().and_then(|value| value.to_str())
            {
                references.entry(file_name.to_string()).or_default().push(ReferenceTarget {
                    path: target.path.clone(),
                    scope: ReferenceScope::SameOwner,
                    syntax: ReferenceSyntax::RustSourcePath,
                });
            }
        } else if let Some(module_path) = python_module_path(&target.path) {
            references.entry(module_path.clone()).or_default().push(ReferenceTarget {
                path: target.path.clone(),
                scope: ReferenceScope::Any,
                syntax: ReferenceSyntax::None,
            });
            if let Some(module_name) = module_path.rsplit('.').next() {
                references.entry(module_name.to_string()).or_default().push(ReferenceTarget {
                    path: target.path.clone(),
                    scope: ReferenceScope::SiblingModule,
                    syntax: ReferenceSyntax::PythonModule,
                });
            }
        }
    }

    let patterns = references.keys().cloned().collect::<Vec<_>>();
    let matcher = AhoCorasick::new(&patterns).context("compile ownership caller matcher")?;
    let mut direct = files
        .iter()
        .map(|target| (target.path.clone(), BTreeSet::<String>::new()))
        .collect::<BTreeMap<_, _>>();
    for caller in files
        .iter()
        .filter(|caller| caller.path != OWNERSHIP_AUDIT_SOURCE)
        .filter(|caller| is_production_code_reference(&caller.path))
    {
        for matched in matcher.find_overlapping_iter(caller.content.as_bytes()) {
            let pattern = &patterns[matched.pattern().as_usize()];
            if !has_identifier_boundaries(&caller.content, matched.start(), matched.end()) {
                continue;
            }
            for target in &references[pattern] {
                if caller.path == target.path || !reference_scope_matches(caller, target) {
                    continue;
                }
                let syntax_matches = match target.syntax {
                    ReferenceSyntax::None => true,
                    ReferenceSyntax::RustModule => {
                        has_rust_module_syntax(&caller.content, matched.start(), matched.end())
                    }
                    ReferenceSyntax::RustSourcePath => {
                        has_rust_source_path_syntax(&caller.content, matched.start())
                    }
                    ReferenceSyntax::PythonModule => {
                        has_python_module_syntax(&caller.content, matched.start(), matched.end())
                    }
                };
                if !syntax_matches {
                    continue;
                }
                direct
                    .get_mut(&target.path)
                    .context("caller graph target disappeared")?
                    .insert(caller.path.clone());
            }
        }
    }
    let direct = direct
        .into_iter()
        .map(|(target, callers)| (target, callers.into_iter().collect::<Vec<_>>()))
        .collect::<BTreeMap<_, _>>();

    direct
        .iter()
        .map(|(target, immediate)| {
            let direct_set = immediate.iter().cloned().collect::<BTreeSet<_>>();
            let mut visited = direct_set.clone();
            let mut frontier = immediate.clone();
            while let Some(caller) = frontier.pop() {
                // A namespace declares children but does not execute all of them. Stopping here
                // prevents one child's caller closure from expanding through unrelated siblings.
                if is_namespace_module(&caller) {
                    continue;
                }
                if let Some(parents) = direct.get(&caller) {
                    for parent in parents {
                        if parent != target && visited.insert(parent.clone()) {
                            frontier.push(parent.clone());
                        }
                    }
                }
            }
            let indirect = visited.difference(&direct_set).cloned().collect::<Vec<_>>();
            Ok((target.clone(), CallerClosure { direct: immediate.clone(), indirect }))
        })
        .collect()
}

fn reference_scope_matches(caller: &SourceFile, target: &ReferenceTarget) -> bool {
    match target.scope {
        ReferenceScope::Any => true,
        ReferenceScope::SameOwner => current_owner(&caller.path) == current_owner(&target.path),
        ReferenceScope::SiblingModule => {
            current_owner(&caller.path) == current_owner(&target.path)
                && shares_module_parent(&caller.path, &target.path)
        }
    }
}

fn has_identifier_boundaries(content: &str, start: usize, end: usize) -> bool {
    let bytes = content.as_bytes();
    let identifier = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
    (start == 0 || !identifier(bytes[start - 1])) && (end == bytes.len() || !identifier(bytes[end]))
}

fn has_rust_module_syntax(content: &str, start: usize, end: usize) -> bool {
    let before = &content[..start];
    let after = &content[end..];
    let preceding = before.trim_end();
    let following = after.trim_start();
    following.starts_with("::")
        || preceding.ends_with("::")
        || preceding.ends_with("mod")
        || (preceding.ends_with('{') || preceding.ends_with(','))
            && (following.starts_with(',')
                || following.starts_with('}')
                || following.starts_with("::"))
}

fn has_rust_source_path_syntax(content: &str, start: usize) -> bool {
    let line_start = content[..start].rfind('\n').map_or(0, |index| index + 1);
    let before = &content[line_start..start];
    before.contains("include!(") || before.contains("#[path")
}

fn has_python_module_syntax(content: &str, start: usize, end: usize) -> bool {
    let line_start = content[..start].rfind('\n').map_or(0, |index| index + 1);
    let line_end = content[end..].find('\n').map_or(content.len(), |index| end + index);
    let before = content[line_start..start].trim_end();
    let after = content[end..line_end].trim_start();
    before.ends_with("import")
        || before.ends_with("from")
        || before.ends_with('.')
        || after.starts_with("import")
}

fn rust_module_path(path: &str) -> Option<String> {
    let (_, source_path) = path.split_once("/src/")?;
    let mut components =
        source_path.trim_end_matches(".rs").split('/').map(str::to_string).collect::<Vec<_>>();
    if components.last().is_some_and(|component| component == "lib" || component == "main") {
        return None;
    }
    if components.last().is_some_and(|component| component == "mod") {
        components.pop();
    }
    (!components.is_empty()).then(|| components.join("::"))
}

fn shares_module_parent(caller: &str, target: &str) -> bool {
    let Some(target_module) = rust_module_path(target) else {
        return Path::new(caller).parent() == Path::new(target).parent();
    };
    let Some(target_parent) = target_module.rsplit_once("::").map(|(parent, _)| parent) else {
        return Path::new(caller).parent() == Path::new(target).parent();
    };
    let Some(caller_module) = rust_module_path(caller) else {
        return false;
    };
    caller_module == target_parent
        || caller_module.rsplit_once("::").is_some_and(|(parent, _)| parent == target_parent)
}

fn is_namespace_module(path: &str) -> bool {
    path.ends_with("/mod.rs") || path.ends_with("/lib.rs") || path.ends_with("/main.rs")
}

fn python_module_path(path: &str) -> Option<String> {
    path.strip_suffix(".py").map(|module| module.replace('/', "."))
}

fn legacy_callers(files: &[SourceFile]) -> BTreeMap<String, Vec<String>> {
    ROOT_PYTHON_AUTHORITIES
        .iter()
        .map(|authority| {
            let mut matches = files
                .iter()
                .filter(|file| file.path != OWNERSHIP_AUDIT_SOURCE)
                .filter(|file| is_production_executable_reference(&file.path))
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

fn is_production_executable_reference(path: &str) -> bool {
    is_executable_reference(path) && !is_test_source(path)
}

fn is_production_code_reference(path: &str) -> bool {
    (path == "Makefile"
        || path.starts_with("makes/")
        || path.starts_with(".github/workflows/")
        || matches!(
            Path::new(path).extension().and_then(|value| value.to_str()),
            Some("mk" | "py" | "rs" | "sh" | "yaml" | "yml")
        ))
        && !is_test_source(path)
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
                    file.path.starts_with("crates/")
                        && file.path.ends_with("Cargo.toml")
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
        format!("- Unknown owner files: `{}`", audit.summary.unknown_owner_files),
        format!("- Direct production caller edges: `{}`", audit.summary.direct_caller_edges),
        format!("- Indirect production caller edges: `{}`", audit.summary.indirect_caller_edges),
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
        classify, dependency_consumers, is_relevant, legacy_authorities, legacy_callers,
        production_callers, render_markdown, validate_audit, AuditSummary, OwnershipAudit,
        OwnershipRow, SourceFile, DIRECT_BENCH_PRODUCT_CONSUMERS, OUTPUT_JSON, OUTPUT_MARKDOWN,
        OUTPUT_SCHEMA, OWNERSHIP_AUDIT_SOURCE, ROOT_PYTHON_AUTHORITIES,
    };

    #[test]
    fn ownership_outputs_match_the_execution_contract() {
        assert_eq!(OUTPUT_JSON, "artifacts/architecture/container-benchmark-ownership.json");
        assert_eq!(OUTPUT_MARKDOWN, "artifacts/architecture/container-benchmark-ownership.md");
        assert_eq!(
            OUTPUT_SCHEMA,
            "artifacts/architecture/container-benchmark-ownership.schema.json"
        );
    }

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
    fn benchmark_product_direct_consumers_are_declared_composition_surfaces() {
        let files = [
            SourceFile {
                path: "crates/bijux-dna-api/Cargo.toml".to_string(),
                content: "bijux-dna-bench.workspace = true".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna-dev/Cargo.toml".to_string(),
                content: "bijux-dna-bench.workspace = true".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/Cargo.toml".to_string(),
                content: "bijux-dna-api.workspace = true".to_string(),
            },
            SourceFile {
                path: "Cargo.toml".to_string(),
                content: "bijux-dna-bench = { path = \"crates/bijux-dna-bench\" }".to_string(),
            },
        ];
        let consumers = dependency_consumers(&files);
        assert_eq!(
            consumers["bijux-dna-bench"], DIRECT_BENCH_PRODUCT_CONSUMERS,
            "the user CLI must consume benchmark services through the API composition surface",
        );
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
    fn crate_owned_vcf_planning_commands_are_thin_adapters() -> anyhow::Result<()> {
        let schema_pattern = Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?;
        for path in [
            "crates/bijux-dna/src/commands/benchmark/local_vcf_stage_catalog.rs",
            "crates/bijux-dna/src/commands/benchmark/local_vcf_stage_matrix.rs",
        ] {
            let row = classify(
                &SourceFile { path: path.to_string(), content: String::new() },
                &schema_pattern,
                &BTreeMap::new(),
            );
            assert_eq!(row.classification, "thin external adapter", "{path}");
        }
        Ok(())
    }

    #[test]
    fn migrated_container_operator_commands_remain_thin_adapters() -> anyhow::Result<()> {
        let schema_pattern = Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?;
        for (path, content) in [
            (
                "crates/bijux-dna-dev/src/commands/containers/external_artifact.rs",
                include_str!("../containers/external_artifact.rs"),
            ),
            (
                "crates/bijux-dna-dev/src/commands/containers/image_activation.rs",
                include_str!("../containers/image_activation.rs"),
            ),
            (
                "crates/bijux-dna-dev/src/commands/containers/qualification_cycle.rs",
                include_str!("../containers/qualification_cycle.rs"),
            ),
            (
                "crates/bijux-dna-dev/src/commands/containers/qualification_slurm.rs",
                include_str!("../containers/qualification_slurm.rs"),
            ),
        ] {
            let row = classify(
                &SourceFile { path: path.to_string(), content: content.to_string() },
                &schema_pattern,
                &BTreeMap::new(),
            );
            assert_eq!(row.classification, "thin external adapter", "{path}");
        }
        Ok(())
    }

    #[test]
    fn migrated_container_operator_commands_reject_direct_io_regression() -> anyhow::Result<()> {
        let row = classify(
            &SourceFile {
                path: "crates/bijux-dna-dev/src/commands/containers/qualification_cycle.rs"
                    .to_string(),
                content: "bijux_dna_runner::api::status(); std::fs::write(path, bytes);"
                    .to_string(),
            },
            &Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?,
            &BTreeMap::new(),
        );
        assert_eq!(row.classification, "duplicate legacy logic");
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
    fn tests_and_manifests_are_not_reported_as_product_authorities() -> anyhow::Result<()> {
        let schema_pattern = Regex::new(r"bijux(?:\.[A-Za-z0-9_-]+)+\.v[0-9]+")?;
        let test_row = classify(
            &SourceFile {
                path: "crates/bijux-dna-bench-model/tests/contracts/local_hpc_simulation.rs"
                    .to_string(),
                content: String::new(),
            },
            &schema_pattern,
            &BTreeMap::new(),
        );
        let manifest_row = classify(
            &SourceFile {
                path: "crates/bijux-dna-api/Cargo.toml".to_string(),
                content: String::new(),
            },
            &schema_pattern,
            &BTreeMap::new(),
        );
        let readiness_namespace = classify(
            &SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/readiness/mod.rs".to_string(),
                content: "pub(crate) mod catalog;".to_string(),
            },
            &schema_pattern,
            &BTreeMap::new(),
        );

        assert_eq!(test_row.classification, "test contract");
        assert_eq!(manifest_row.classification, "declarative product asset");
        assert_eq!(readiness_namespace.classification, "thin external adapter");
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
    fn legacy_inventory_ignores_test_only_negative_references() {
        let callers = legacy_callers(&[SourceFile {
            path: "crates/bijux-dna-runner/tests/qualification.rs".to_string(),
            content: "assert!(!script.contains(\"container_validation.py\"));".to_string(),
        }]);

        assert!(callers["containers/contracts/container_validation.py"].is_empty());
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
            direct_callers: Vec::new(),
            indirect_callers: Vec::new(),
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
    fn production_callers_include_direct_and_indirect_rust_consumers() -> anyhow::Result<()> {
        let files = [
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/local_stage_inventory.rs"
                    .to_string(),
                content: "pub fn load() {}".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/local_pipeline_dag.rs".to_string(),
                content: "use super::local_stage_inventory::load;".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/src/commands/fastq/meta/entrypoint.rs".to_string(),
                content:
                    "crate::commands::benchmark::local_pipeline_dag::validate_pipeline_dag_path();"
                        .to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/mod.rs".to_string(),
                content: "pub mod local_pipeline_dag; pub mod local_stage_inventory;".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/tests/local_stage_inventory.rs".to_string(),
                content: "crate::commands::benchmark::local_stage_inventory::load();".to_string(),
            },
        ];

        let graph = production_callers(&files)?;
        assert!(!is_relevant("crates/bijux-dna/src/commands/fastq/meta/entrypoint.rs"));
        let callers = &graph["crates/bijux-dna/src/commands/benchmark/local_stage_inventory.rs"];
        assert_eq!(
            callers.direct,
            [
                "crates/bijux-dna/src/commands/benchmark/local_pipeline_dag.rs",
                "crates/bijux-dna/src/commands/benchmark/mod.rs"
            ]
        );
        assert_eq!(callers.indirect, ["crates/bijux-dna/src/commands/fastq/meta/entrypoint.rs"]);
        Ok(())
    }

    #[test]
    fn production_callers_follow_python_imports_into_automation() -> anyhow::Result<()> {
        let files = [
            SourceFile {
                path: "benchmarks/data_catalog.py".to_string(),
                content: "def load(): pass".to_string(),
            },
            SourceFile {
                path: "benchmarks/catalog_consumer.py".to_string(),
                content: "from data_catalog import load".to_string(),
            },
            SourceFile {
                path: "Makefile".to_string(),
                content: "catalog:\n\tpython3 benchmarks/catalog_consumer.py".to_string(),
            },
        ];

        let graph = production_callers(&files)?;
        let callers = &graph["benchmarks/data_catalog.py"];
        assert_eq!(callers.direct, ["benchmarks/catalog_consumer.py"]);
        assert_eq!(callers.indirect, ["Makefile"]);
        Ok(())
    }

    #[test]
    fn production_callers_resolve_file_modules_and_included_rust_sources() -> anyhow::Result<()> {
        let files = [
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/workspace/config_loading.rs"
                    .to_string(),
                content: "pub fn load() {}".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/workspace.rs".to_string(),
                content: "mod config_loading;".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/suite/suite_sections/run_suite.rs"
                    .to_string(),
                content: "pub fn run() {}".to_string(),
            },
            SourceFile {
                path: "crates/bijux-dna/src/commands/benchmark/suite/mod.rs".to_string(),
                content: "include!(\"suite_sections/run_suite.rs\");".to_string(),
            },
        ];

        let graph = production_callers(&files)?;
        assert_eq!(
            graph["crates/bijux-dna/src/commands/benchmark/workspace/config_loading.rs"].direct,
            ["crates/bijux-dna/src/commands/benchmark/workspace.rs"]
        );
        assert_eq!(
            graph["crates/bijux-dna/src/commands/benchmark/suite/suite_sections/run_suite.rs"]
                .direct,
            ["crates/bijux-dna/src/commands/benchmark/suite/mod.rs"]
        );
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
            schema_version: "bijux.architecture.ownership_audit.v3",
            summary: AuditSummary {
                generated_from_commit: "abc123".to_string(),
                total_files: 0,
                unclassified_files: 0,
                unknown_owner_files: 0,
                classification_counts: BTreeMap::new(),
                scope_counts: BTreeMap::new(),
                direct_caller_edges: 0,
                indirect_caller_edges: 0,
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
            schema_version: "bijux.architecture.ownership_audit.v3",
            summary: AuditSummary {
                generated_from_commit: "a".repeat(40),
                total_files: 0,
                unclassified_files: 0,
                unknown_owner_files: 0,
                classification_counts: BTreeMap::new(),
                scope_counts: BTreeMap::new(),
                direct_caller_edges: 0,
                indirect_caller_edges: 0,
                dependency_consumers: BTreeMap::new(),
                legacy_authorities: Vec::new(),
            },
            files: Vec::new(),
        };
        assert!(validate_audit(&audit).is_err());
    }
}
