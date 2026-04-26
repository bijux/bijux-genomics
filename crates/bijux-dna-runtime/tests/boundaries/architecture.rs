use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn runtime_source_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "runtime crate root must stay minimal"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "environment.rs",
            "lib.rs",
            "manifests/",
            "observability/",
            "provenance/",
            "recording/",
            "run/",
            "run_layout/",
            "runner/",
            "telemetry/",
        ]),
        "runtime src tree must stay grouped by contract owner"
    );

    assert_eq!(
        dir_entries(&root.join("src/manifests")),
        entries([
            "classification.rs",
            "domain_registry.rs",
            "generated_registry.rs",
            "loader.rs",
            "mod.rs",
            "source.rs",
            "stable_surface.rs",
        ]),
        "runtime manifest registry modules must stay focused"
    );

    assert_eq!(
        dir_entries(&root.join("src/observability")),
        entries(["contracts.rs", "mod.rs", "reports/", "telemetry/"]),
        "runtime observability modules must separate contracts, reports, and telemetry schema"
    );

    assert_eq!(
        dir_entries(&root.join("src/observability/reports")),
        entries(["mod.rs", "run_reports.rs", "stage_reports.rs"]),
        "runtime report schemas must stay split by run and stage scope"
    );

    assert_eq!(
        dir_entries(&root.join("src/observability/telemetry")),
        entries(["attrs_and_events.rs", "facts_and_provenance.rs", "mod.rs"]),
        "runtime telemetry schemas must stay split by event and provenance concerns"
    );

    assert_eq!(
        dir_entries(&root.join("src/recording")),
        entries([
            "envelope.rs",
            "io.rs",
            "manifests/",
            "metrics.rs",
            "mod.rs",
            "provenance.rs",
            "stable_surface.rs",
            "telemetry.rs",
        ]),
        "runtime recording modules must stay grouped by writer concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/recording/manifests")),
        entries([
            "artifact_catalog.rs",
            "bootstrap.rs",
            "manifest_identity.rs",
            "mod.rs",
            "profile_lock.rs",
            "records.rs",
            "reproducibility.rs",
            "run_dirs.rs",
            "run_manifest.rs",
            "runtime_support_files.rs",
            "stable_surface.rs",
        ]),
        "runtime manifest writers must stay decomposed by artifact responsibility"
    );

    assert_eq!(
        dir_entries(&root.join("src/run_layout")),
        entries([
            "api.rs",
            "contracts.rs",
            "journal.rs",
            "layout_creation.rs",
            "mod.rs",
            "stable_surface.rs",
        ]),
        "runtime run_layout modules must separate contracts, creation, writers, and journal"
    );

    assert_eq!(
        dir_entries(&root.join("src/runner")),
        entries(["contract_kinds.rs", "contracts.rs", "mod.rs", "model.rs", "stable_surface.rs"]),
        "runtime runner modules must define contracts without backend execution"
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
