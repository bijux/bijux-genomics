use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn bench_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries([
            "BOUNDARY.md",
            "Cargo.toml",
            "PUBLIC_API.md",
            "README.md",
            "bench/",
            "docs/",
            "src/",
            "tests/",
        ]),
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
        "repo tree must stay split between workspace policy and persisted artifacts"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo/run_artifacts")),
        entries([
            "manifest_loader.rs",
            "metrics_loader.rs",
            "mod.rs",
            "observations_loader.rs",
        ]),
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
            "README.md",
            "boundaries/",
            "boundaries.rs",
            "contracts/",
            "contracts.rs",
            "determinism/",
            "determinism.rs",
            "fixtures/",
            "guardrails.rs",
            "schemas/",
            "semantics/",
            "semantics.rs",
            "snapshots/",
            "workspace_paths.rs",
        ]),
        "test tree must stay organized by enduring intent"
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
