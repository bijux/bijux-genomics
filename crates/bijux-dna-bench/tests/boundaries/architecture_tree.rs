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
        entries(["mod.rs", "writer.rs"]),
        "artifacts tree must stay focused on deterministic serialization"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["mod.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/repo")),
        entries([
            "mod.rs",
            "run_artifacts.rs",
            "run_repo.rs",
            "sqlite/",
            "workspace_paths.rs",
        ]),
        "repo tree must stay split between repository policy and persisted artifacts"
    );

    assert_eq!(
        dir_entries(&root.join("src/workflow")),
        entries([
            "evaluation.rs",
            "mod.rs",
            "options.rs",
            "run_suite.rs",
            "suite_load.rs",
            "summary_support.rs",
        ]),
        "workflow tree must stay partitioned by enduring benchmark concern"
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
