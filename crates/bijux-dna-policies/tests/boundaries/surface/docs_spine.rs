#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use support::{crate_roots, read_to_string, workspace_root};

fn docs_root() -> PathBuf {
    workspace_root().join("docs")
}

fn is_uppercase_stem(path: &Path) -> bool {
    path.file_stem()
        .and_then(|stem| stem.to_str())
        .map(|stem| stem == stem.to_uppercase())
        .unwrap_or(false)
}

fn required_headings() -> [&'static str; 6] {
    [
        "## What",
        "## Why",
        "## Non-goals",
        "## Contracts",
        "## Examples",
        "## Failure modes",
    ]
}

#[test]
fn policy__boundaries__docs_spine__docs_placement_contract() {
    let root = docs_root();
    let allowed_root_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "assets",
        "overrides",
    ]);
    let allowed_root_files =
        BTreeSet::from(["ARCHITECTURE_LITMUS.md", "index.md", "TESTS_STYLE.md"]);
    let mut root_entries = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read docs root") {
        let entry = entry.expect("read entry");
        let name = entry.file_name().to_string_lossy().to_string();
        root_entries.push(name.clone());
        let path = entry.path();
        if path.is_dir() {
            if !allowed_root_dirs.contains(name.as_str()) {
                bijux_dna_policies::policy_panic!(
                    "docs root contains unexpected directory: {}",
                    name
                );
            }
        } else if path.is_file() && !allowed_root_files.contains(name.as_str()) {
            bijux_dna_policies::policy_panic!("docs root contains unexpected file: {}", name);
        }
    }
    bijux_dna_policies::policy_assert!(root_entries.contains(&"index.md".to_string()));
}

#[test]
fn policy__boundaries__docs_spine__no_docs_under_src() {
    for crate_root in crate_roots() {
        let src = crate_root.join("src");
        if !src.exists() {
            continue;
        }
        for entry in WalkDir::new(&src) {
            let entry = entry.expect("walk src");
            if entry.file_type().is_file()
                && entry.path().extension().and_then(|ext| ext.to_str()) == Some("md")
                && !matches!(
                    entry.path().file_name().and_then(|name| name.to_str()),
                    Some("OWNER.md") | Some("INDEX.md")
                )
            {
                bijux_dna_policies::policy_panic!(
                    "docs under src are forbidden: {}",
                    entry.path().display()
                );
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__crate_docs_contract() {
    let allowlist = BTreeSet::from([
        "SCOPE.md",
        "ARCHITECTURE.md",
        "INDEX.md",
        "EFFECTS.md",
        "TESTS.md",
        "CHANGE_RULES.md",
        "POLICY_DIAGNOSTICS.md",
        "POLICIES.md",
        "POLICY_MATRIX.md",
        "ENFORCEMENT.md",
        "EXCEPTIONS.md",
        "EVOLUTION.md",
        "CONTRACT.md",
        "SCHEMAS.md",
        "PUBLIC_API.md",
        "CONTRACTS.md",
        "CONTRACT_MAP.md",
        "CONTRACT_VERSIONING.md",
        "SSOT.md",
        "SERIALIZATION.md",
        "INVARIANTS.md",
        "ENGINE_MODEL.md",
        "ENGINE_CONTRACT.md",
        "DETERMINISM.md",
        "ERROR_TAXONOMY.md",
        "ERRORS.md",
        "RECORDING_TRUTH_SET.md",
        "RUNTIME_CONTRACT.md",
        "ARTIFACTS.md",
        "EVENTS.md",
        "GLOSSARY.md",
        "OBSERVABILITY.md",
        "BOUNDARY.md",
        "COMPATIBILITY.md",
        "RECORDER.md",
        "EXAMPLE_RUN.md",
        "EXAMPLE_PLAN.md",
        "EXAMPLE_PLAN.json",
        "TOOL_COVERAGE.md",
        "ADD_FIXTURE.md",
        "FIXTURE_STANDARDS.md",
        "BACKENDS.md",
        "REPLAY.md",
        "EXECUTION_SPEC.md",
        "SECURITY.md",
        "EXTENSION_POINTS.md",
        "FAILURES.md",
        "FAILURE_TAXONOMY.md",
        "FAILURE_ANALYSIS.md",
        "FIXTURES.md",
        "NO_DOMAIN.md",
        "WHY_YAML.md",
        "LOGGING.md",
        "PATHS.md",
        "STABILITY.md",
        "REQUEST_FLOW.md",
        "ENV_REFERENCE.md",
        "ENV_MATRIX.md",
        "SCHEMAS.md",
        "THREAT_MODEL.md",
        "CACHE_SEMANTICS.md",
        "EXTENSION_GUIDE.md",
        "QA_MATRIX.md",
        "DATASETS.md",
        "APPTAINER_PLAN.md",
        "OFFLINE_POLICY.md",
        "ARTIFACT_CONTRACT.md",
        "VERSIONING.md",
        "MINIMALITY.md",
        "DECISION_EXPLAINABILITY.md",
        "OUTPUT_FORMATS.md",
        "METRICS_GLOSSARY.md",
        "BANKS.md",
        "EFFECT_BOUNDARY.md",
        "ADD_PIPELINE_PROFILE.md",
        "STAGE_LIST.md",
        "OBSERVERS.md",
        "STAGE_CONTRACTS.md",
        "TOOL_ROSTER.md",
        "ADD_OBSERVER.md",
        "PHASES.md",
        "DOMAIN_MODEL.md",
        "METRICS.md",
        "PARAMS.md",
        "STAGES.md",
        "FAILURE_PATTERNS.md",
        "PLANNER_MODEL.md",
        "TOOL_SELECTION.md",
        "STAGE_MAPPING.md",
        "EXPLAIN_OUTPUT.md",
        "DETERMINISM.md",
        "ADD_TOOL.md",
        "DETERMINISM.md",
        "ADD_TOOL.md",
        "PIPELINES.md",
        "PIPELINE_MODEL.md",
        "PIPELINE_VERSIONING.md",
        "DEFAULTS_LEDGER.md",
        "PROFILE_RATIONALE.md",
        "PIPELINE_CHANGE_RULES.md",
        "AUDIT_CHECKLIST.md",
        "DATA_MODEL.md",
        "SCHEMA.md",
        "DECISIONS.md",
        "REPORT_CONTRACT.md",
        "INTERPRETATION.md",
        "FAILURE_ANALYSIS.md",
        "API.md",
        "API_STABILITY.md",
        "ENDPOINT_GUIDES.md",
        "BOUNDARIES.md",
        "BOUNDARY_CONTRACT.md",
        "LIFECYCLE.md",
        "COMPATIBILITY_POLICY.md",
        "SECURITY.md",
        "COMMANDS.md",
        "CLI_CONVENTIONS.md",
        "DRY_RUN.md",
        "UX_ERRORS.md",
        "HELP_STABILITY.md",
        "BUG_REPORT.md",
        "FIXTURE_STANDARDS.md",
        "SNAPSHOT_POLICY.md",
        "USAGE.md",
        "BENCH_CONTRACT.md",
        "BENCH_FORMAT.md",
        "EXPLAINABILITY.md",
        "SUITE_DESIGN.md",
        "LEGACY.md",
        "REPRODUCIBILITY.md",
        "MODEL_GLOSSARY.md",
        "STAT_ASSUMPTIONS.md",
        "GATE_POLICY.md",
        "DETERMINISM.md",
        "IMAGE_QA_INDEX.md",
        "CHANGE_MODEL.md",
        "MODEL.md",
        "REFERENCES.md",
        "RUNBOOK.md",
        "PERFORMANCE_BUDGET.md",
        "DETERMINISM.md",
        "ADD_RULES.md",
        "BLESS_WORKFLOW.md",
        "TEST_SUPPORT.md",
    ]);
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let readme = crate_root.join("README.md");
        if !readme.exists() {
            bijux_dna_policies::policy_panic!("crate README.md missing: {}", crate_root.display());
        }
        for entry in std::fs::read_dir(&crate_root).expect("read crate root") {
            let entry = entry.expect("read entry");
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
                continue;
            }
            if path.file_name().unwrap() != "README.md" {
                bijux_dna_policies::policy_panic!(
                    "crate root contains extra doc: {}",
                    path.display()
                );
            }
        }
        let mut required = BTreeSet::from(["SCOPE.md", "ARCHITECTURE.md"]);
        if !docs.exists() {
            bijux_dna_policies::policy_panic!("crate docs directory missing: {}", docs.display());
        }
        for entry in std::fs::read_dir(&docs).expect("read docs dir") {
            let entry = entry.expect("read entry");
            let name = entry.file_name().to_string_lossy().to_string();
            if !allowlist.contains(name.as_str()) {
                bijux_dna_policies::policy_panic!(
                    "crate docs contains non-allowlisted file {} in {}",
                    name,
                    docs.display()
                );
            }
            if !is_uppercase_stem(entry.path().as_path()) {
                bijux_dna_policies::policy_panic!(
                    "crate docs filename must be uppercase: {}",
                    entry.path().display()
                );
            }
            required.remove(name.as_str());
        }
        if !required.is_empty() {
            bijux_dna_policies::policy_panic!(
                "crate docs missing required files in {}: {:?}",
                docs.display(),
                required
            );
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__root_docs_style_template() {
    let root = docs_root();
    let roots = [
        root.join("00-intro"),
        root.join("10-architecture"),
        root.join("20-science"),
        root.join("30-operations"),
        root.join("40-policies"),
        root.join("50-reference"),
    ];
    for dir in roots {
        for entry in WalkDir::new(&dir) {
            let entry = entry.expect("walk docs");
            if !entry.file_type().is_file() {
                continue;
            }
            if entry.path().extension().and_then(|e| e.to_str()) != Some("md") {
                continue;
            }
            if !is_uppercase_stem(entry.path()) {
                bijux_dna_policies::policy_panic!(
                    "root docs filename must be uppercase: {}",
                    entry.path().display()
                );
            }
            let content = read_to_string(entry.path());
            for heading in required_headings() {
                if !content.contains(heading) {
                    bijux_dna_policies::policy_panic!(
                        "doc missing required heading {heading}: {}",
                        entry.path().display()
                    );
                }
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__root_docs_metadata_headers() {
    let root = docs_root();
    let key_docs = [
        root.join("10-architecture/SSOT.md"),
        root.join("10-architecture/BOUNDARY_MAP.md"),
        root.join("10-architecture/CONTRACT_SPINE.md"),
    ];
    let required = [
        "Owner:",
        "Scope:",
        "Last reviewed:",
        "Contract version:",
        "Applies to crates:",
    ];
    for doc in key_docs {
        let content = read_to_string(&doc);
        for header in required {
            if !content.lines().any(|line| line.starts_with(header)) {
                bijux_dna_policies::policy_panic!(
                    "missing metadata header {header} in {}",
                    doc.display()
                );
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__stage_catalog_schema() {
    let root = docs_root();
    let catalogs = [
        root.join("20-science/fastq/STAGE_CATALOG.md"),
        root.join("20-science/bam/STAGE_CATALOG.md"),
    ];
    let required = [
        "Purpose:",
        "Inputs/Outputs:",
        "Metrics:",
        "Tools:",
        "Defaults:",
        "References:",
    ];
    for catalog in catalogs {
        let content = read_to_string(&catalog);
        for section in content.split("\n### ").skip(1) {
            for field in required {
                if !section.contains(field) {
                    bijux_dna_policies::policy_panic!(
                        "stage catalog missing field {field} in {}",
                        catalog.display()
                    );
                }
            }
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__authority_docs_unique_h1() {
    let root = docs_root();
    let key_docs = [
        root.join("10-architecture/SSOT.md"),
        root.join("10-architecture/BOUNDARY_MAP.md"),
        root.join("10-architecture/CONTRACT_SPINE.md"),
    ];
    let mut seen = HashMap::new();
    for doc in key_docs {
        let content = read_to_string(&doc);
        let title = content
            .lines()
            .find(|line| line.starts_with("# "))
            .unwrap_or("");
        if let Some(prev) = seen.insert(title.to_string(), doc.clone()) {
            bijux_dna_policies::policy_panic!(
                "duplicate H1 heading {title} in {} and {}",
                prev.display(),
                doc.display()
            );
        }
    }
}

#[test]
fn policy__boundaries__docs_spine__contract_versioning_note_exists() {
    let doc = docs_root().join("50-reference/CONTRACT_VERSIONING.md");
    let content = read_to_string(&doc);
    let required = ["Breaking change", "major bump", "minor bump", "snapshot"];
    for phrase in required {
        if !content.contains(phrase) {
            bijux_dna_policies::policy_panic!("contract versioning doc missing phrase {phrase}");
        }
    }
}
