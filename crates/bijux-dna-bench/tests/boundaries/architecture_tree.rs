use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn bench_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_docs_tree(root);
    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "bench/", "docs/", "src/", "tests/"]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries(["artifacts/", "lib.rs", "public_api/", "repo/", "workflow/"]),
        "src tree must match the documented benchmark layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/artifacts")),
        entries(["mod.rs", "writer/"]),
        "artifacts tree must stay focused on deterministic serialization"
    );

    assert_eq!(
        dir_entries(&root.join("src/artifacts/writer")),
        entries([
            "mod.rs",
            "observation_reader.rs",
            "observation_writer.rs",
            "structured_writer.rs",
        ]),
        "artifact writer tree must separate observation loading, observation persistence, and structured report writing"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["mod.rs", "stable_surface.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo")),
        entries([
            "mod.rs",
            "repo_root.rs",
            "repository.rs",
            "run_artifacts/",
            "run_metadata.rs",
            "sqlite/",
            "workspace_paths.rs",
        ]),
        "repo tree must stay split between repository policy and persisted artifacts"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/run_artifacts")),
        entries(["manifest_loader.rs", "metrics_loader.rs", "mod.rs", "observations_loader.rs",]),
        "run artifact loaders must stay separated by persisted artifact kind"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/sqlite/queries")),
        entries(["mod.rs", "run_index/"]),
        "sqlite query tree must stay focused on explicit repository query families"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/sqlite/queries/run_index")),
        entries(["metadata_paths.rs", "mod.rs"]),
        "run index query tree must separate repository queries from metadata path policy"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow")),
        entries([
            "evaluation.rs",
            "mod.rs",
            "options.rs",
            "run_suite/",
            "summary/",
            "suite_load.rs",
            "summary_fairness.rs",
            "summary_scope.rs",
            "summary_statistics.rs",
        ]),
        "workflow tree must stay partitioned by enduring benchmark concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/run_suite")),
        entries(["mod.rs", "persistence.rs"]),
        "suite run tree must separate orchestration from artifact persistence"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow/summary")),
        entries(["grouping.rs", "mod.rs", "row_metrics.rs", "strata.rs"]),
        "summary tree must separate grouping, row metrics, and stratum aggregation"
    );

    assert_eq!(
        dir_entries(&root.join("tests")),
        entries([
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "semantics/",
            "semantics.rs",
            "snapshots/",
            "workspace_paths.rs",
        ]),
        "test tree must stay organized by enduring intent"
    );
}

#[test]
fn manifest_dependency_graph_matches_boundary_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest = root.join("Cargo.toml");
    let content = std::fs::read_to_string(&manifest)
        .unwrap_or_else(|err| panic!("read {}: {err}", manifest.display()));
    let parsed: toml::Value = toml::from_str(&content)
        .unwrap_or_else(|err| panic!("parse {}: {err}", manifest.display()));

    let dependencies = dependency_names(&parsed, "dependencies");
    let dev_dependencies = dependency_names(&parsed, "dev-dependencies");
    let duplicates = dependencies.intersection(&dev_dependencies).cloned().collect::<Vec<_>>();

    assert!(
        duplicates.is_empty(),
        "normal and dev dependencies must not duplicate edges: {duplicates:?}",
    );
    for forbidden in [
        "bijux-dna-api",
        "bijux-dna-analyze",
        "bijux-dna-domain-bam",
        "bijux-dna-domain-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-runner",
        "fastrand",
    ] {
        assert!(
            !dependencies.contains(forbidden),
            "bijux-dna-bench must not carry normal dependency `{forbidden}`",
        );
    }
}

#[test]
fn commands_doc_lists_managed_benchmark_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands_doc = root.join("docs/COMMANDS.md");
    let content = std::fs::read_to_string(&commands_doc)
        .unwrap_or_else(|err| panic!("read {}: {err}", commands_doc.display()));

    for command in
        ["load-suite", "summarize", "compare", "gate", "bench-data-dir", "bench-suites-dir"]
    {
        assert!(
            content.contains(command),
            "docs/COMMANDS.md must list `{command}` as a managed benchmark operation",
        );
    }

    for artifact in ["observations.jsonl", "summary.json", "decision.json", "decisions.json"] {
        assert!(
            content.contains(artifact),
            "docs/COMMANDS.md must list managed benchmark artifact `{artifact}`",
        );
    }
}

fn assert_docs_tree(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("docs")),
        entries([
            "ARCHITECTURE.md",
            "BENCH_CONTRACT.md",
            "BENCH_FORMAT.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "PUBLIC_API.md",
            "REPRODUCIBILITY.md",
            "SUITE_DESIGN.md",
            "TESTS.md",
        ]),
        "bench docs must stay at the 10-document allowance and live under docs/"
    );

    let misplaced_docs = markdown_files_outside_docs(root);
    assert!(
        misplaced_docs.is_empty(),
        "crate markdown outside docs/ must be limited to root README.md: {misplaced_docs:?}",
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}

fn dependency_names(parsed: &toml::Value, table_name: &str) -> BTreeSet<String> {
    parsed
        .get(table_name)
        .and_then(toml::Value::as_table)
        .map(|table| table.keys().cloned().collect())
        .unwrap_or_default()
}

fn markdown_files_outside_docs(root: &Path) -> Vec<String> {
    walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "md"))
        .filter(|entry| {
            let path = entry.path();
            path != root.join("README.md") && !path.starts_with(root.join("docs"))
        })
        .map(|entry| entry.path().display().to_string())
        .collect()
}
