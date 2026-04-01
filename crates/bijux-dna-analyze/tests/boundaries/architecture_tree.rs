use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn analyze_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries([
            "BOUNDARY.md",
            "Cargo.toml",
            "PUBLIC_API.md",
            "README.md",
            "docs/",
            "src/",
            "tests/",
        ]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "aggregate/",
            "api/",
            "contracts/",
            "decision/",
            "diagnostics/",
            "exports/",
            "failure/",
            "lib.rs",
            "load/",
            "model/",
            "pipeline/",
            "public_api/",
            "report/",
            "semantics/",
        ]),
        "src tree must match the documented analysis layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/contracts")),
        entries(["OWNER.toml", "mod.rs"]),
        "contracts tree must stay focused on the versioned handshake"
    );

    assert_eq!(
        dir_entries(&root.join("src/diagnostics")),
        entries(["OWNER.toml", "aggregate.rs", "load.rs", "mod.rs"]),
        "diagnostics tree must stay split by durable error concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/exports")),
        entries([
            "OWNER.toml",
            "dashboard_facts.rs",
            "mod.rs",
            "run_summary.rs",
            "stage_summary.rs",
            "support.rs",
        ]),
        "exports tree must keep output writers separated by artifact concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/pipeline")),
        entries(["OWNER.toml", "mod.rs", "steps/"]),
        "pipeline tree must stay thin and step-oriented"
    );

    assert_eq!(
        dir_entries(&root.join("src/pipeline/steps")),
        entries([
            "compute.rs",
            "load.rs",
            "mod.rs",
            "render.rs",
            "report.rs",
            "validate.rs",
        ]),
        "pipeline steps must stay explicit and canonical"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/report")),
        entries([
            "OWNER.toml",
            "bench/",
            "build/",
            "mod.rs",
            "render/",
            "render_model.rs",
            "sections/",
        ]),
        "report tree must stay decomposed by rendering concern"
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
            "schemas.rs",
            "semantics/",
            "semantics.rs",
            "snapshots/",
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
