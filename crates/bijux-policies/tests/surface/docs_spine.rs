use std::collections::{BTreeSet, HashMap};
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::support::fs::{crate_roots, read_to_string, workspace_root};

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
fn docs_placement_contract() {
    let root = docs_root();
    let allowed_root_dirs = BTreeSet::from([
        "00-intro",
        "10-architecture",
        "20-science",
        "30-operations",
        "40-policies",
        "50-reference",
        "overrides",
    ]);
    let mut root_entries = Vec::new();
    for entry in std::fs::read_dir(&root).expect("read docs root") {
        let entry = entry.expect("read entry");
        let name = entry.file_name().to_string_lossy().to_string();
        root_entries.push(name.clone());
        let path = entry.path();
        if path.is_dir() {
            if !allowed_root_dirs.contains(name.as_str()) {
                panic!("docs root contains unexpected directory: {}", name);
            }
        } else if path.is_file() && name != "index.md" {
            panic!("docs root contains unexpected file: {}", name);
        }
    }
    assert!(root_entries.contains(&"index.md".to_string()));
}

#[test]
fn no_docs_under_src() {
    for crate_root in crate_roots() {
        let src = crate_root.join("src");
        if !src.exists() {
            continue;
        }
        for entry in WalkDir::new(&src) {
            let entry = entry.expect("walk src");
            if entry.file_type().is_file()
                && entry
                    .path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    == Some("md")
            {
                panic!("docs under src are forbidden: {}", entry.path().display());
            }
        }
    }
}

#[test]
fn crate_docs_contract() {
    let allowlist = BTreeSet::from([
        "SCOPE.md",
        "ARCHITECTURE.md",
        "INDEX.md",
        "EFFECTS.md",
        "TESTS.md",
        "CHANGE_RULES.md",
        "POLICY_DIAGNOSTICS.md",
        "POLICIES.md",
        "EXCEPTIONS.md",
        "EVOLUTION.md",
        "CONTRACT.md",
        "PUBLIC_API.md",
        "CONTRACTS.md",
        "SSOT.md",
        "SERIALIZATION.md",
        "INVARIANTS.md",
        "ENGINE_MODEL.md",
        "DETERMINISM.md",
        "ERROR_TAXONOMY.md",
        "RECORDING_TRUTH_SET.md",
        "RUNTIME_CONTRACT.md",
        "EVENTS.md",
        "GLOSSARY.md",
        "BOUNDARY.md",
        "COMPATIBILITY.md",
        "RECORDER.md",
        "EXAMPLE_RUN.md",
        "BACKENDS.md",
        "REPLAY.md",
        "EXECUTION_SPEC.md",
        "SECURITY.md",
        "EXTENSION_POINTS.md",
        "FAILURES.md",
        "NO_DOMAIN.md",
        "WHY_YAML.md",
        "LOGGING.md",
        "PATHS.md",
        "STABILITY.md",
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
        "STAGE_LIST.md",
        "OBSERVERS.md",
        "STAGE_CONTRACTS.md",
        "TOOL_ROSTER.md",
        "PHASES.md",
        "DOMAIN_MODEL.md",
        "METRICS.md",
        "PARAMS.md",
        "STAGES.md",
        "PLANNER_MODEL.md",
        "TOOL_SELECTION.md",
        "STAGE_MAPPING.md",
        "EXPLAIN_OUTPUT.md",
        "PIPELINES.md",
        "PIPELINE_MODEL.md",
        "PIPELINE_VERSIONING.md",
        "DEFAULTS_LEDGER.md",
        "DATA_MODEL.md",
        "SCHEMA.md",
        "DECISIONS.md",
        "API.md",
        "API_STABILITY.md",
        "ENDPOINT_GUIDES.md",
        "BOUNDARIES.md",
        "COMMANDS.md",
        "CLI_CONVENTIONS.md",
        "DRY_RUN.md",
        "UX_ERRORS.md",
        "FIXTURE_STANDARDS.md",
        "SNAPSHOT_POLICY.md",
        "USAGE.md",
        "BENCH_CONTRACT.md",
        "BENCH_FORMAT.md",
        "LEGACY.md",
        "REPRODUCIBILITY.md",
        "MODEL_GLOSSARY.md",
        "STAT_ASSUMPTIONS.md",
        "GATE_POLICY.md",
        "DETERMINISM.md",
        "MODEL.md",
        "REFERENCES.md",
        "RUNBOOK.md",
        "PERFORMANCE_BUDGET.md",
    ]);
    for crate_root in crate_roots() {
        let docs = crate_root.join("docs");
        let readme = crate_root.join("README.md");
        if !readme.exists() {
            panic!("crate README.md missing: {}", crate_root.display());
        }
        for entry in crate_root.glob("*.md").expect("glob md") {
            let entry = entry.expect("glob entry");
            if entry.file_name().unwrap() != "README.md" {
                panic!("crate root contains extra doc: {}", entry.display());
            }
        }
        let mut required = BTreeSet::from(["SCOPE.md", "ARCHITECTURE.md"]);
        if !docs.exists() {
            panic!("crate docs directory missing: {}", docs.display());
        }
        for entry in std::fs::read_dir(&docs).expect("read docs dir") {
            let entry = entry.expect("read entry");
            let name = entry.file_name().to_string_lossy().to_string();
            if !allowlist.contains(name.as_str()) {
                panic!(
                    "crate docs contains non-allowlisted file {} in {}",
                    name,
                    docs.display()
                );
            }
            if !is_uppercase_stem(entry.path().as_path()) {
                panic!("crate docs filename must be uppercase: {}", entry.path().display());
            }
            required.remove(name.as_str());
        }
        if !required.is_empty() {
            panic!(
                "crate docs missing required files in {}: {:?}",
                docs.display(),
                required
            );
        }
    }
}

#[test]
fn root_docs_style_template() {
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
                panic!(
                    "root docs filename must be uppercase: {}",
                    entry.path().display()
                );
            }
            let content = read_to_string(entry.path());
            for heading in required_headings() {
                if !content.contains(heading) {
                    panic!(
                        "doc missing required heading {heading}: {}",
                        entry.path().display()
                    );
                }
            }
        }
    }
}

#[test]
fn root_docs_metadata_headers() {
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
                panic!("missing metadata header {header} in {}", doc.display());
            }
        }
    }
}

#[test]
fn stage_catalog_schema() {
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
                    panic!(
                        "stage catalog missing field {field} in {}",
                        catalog.display()
                    );
                }
            }
        }
    }
}

#[test]
fn authority_docs_unique_h1() {
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
            panic!(
                "duplicate H1 heading {title} in {} and {}",
                prev.display(),
                doc.display()
            );
        }
    }
}

#[test]
fn contract_versioning_note_exists() {
    let doc = docs_root().join("50-reference/CONTRACT_VERSIONING.md");
    let content = read_to_string(&doc);
    let required = ["Breaking change", "major bump", "minor bump", "snapshot"];
    for phrase in required {
        if !content.contains(phrase) {
            panic!("contract versioning doc missing phrase {phrase}");
        }
    }
}
