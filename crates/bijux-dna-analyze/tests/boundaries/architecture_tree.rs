use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn analyze_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_crate_root_layout(root);
    assert_src_layout(root);
    assert_test_layout(root);
}

fn assert_crate_root_layout(root: &Path) {
    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/",]),
        "crate root must stay minimal and intentional"
    );
}

fn assert_src_layout(root: &Path) {
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
    assert_contracts_layout(root);
    assert_exports_layout(root);
    assert_pipeline_layout(root);
    assert_report_layout(root);
}

fn assert_contracts_layout(root: &Path) {
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
}

fn assert_exports_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/exports")),
        entries([
            "OWNER.toml",
            "dashboard_facts.rs",
            "evidence_bundle.rs",
            "facts_summary.rs",
            "facts_support/",
            "mod.rs",
            "run_summary.rs",
            "stage_summary.rs",
        ]),
        "exports tree must keep output writers separated by artifact concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/exports/facts_support")),
        entries(["mod.rs", "params_excerpt.rs", "report_access.rs"]),
        "facts support must keep report access and parameter excerpt policy separate"
    );
}

fn assert_pipeline_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/pipeline")),
        entries(["OWNER.toml", "mod.rs", "steps/"]),
        "pipeline tree must stay thin and step-oriented"
    );

    assert_eq!(
        dir_entries(&root.join("src/pipeline/steps")),
        entries(["compute.rs", "load.rs", "mod.rs", "render.rs", "report.rs", "validate.rs",]),
        "pipeline steps must stay explicit and canonical"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "decision.rs", "exports.rs", "load.rs", "mod.rs", "report.rs"]),
        "public api tree must stay curated"
    );
}

fn assert_report_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src/report")),
        entries([
            "OWNER.toml",
            "bench/",
            "build/",
            "mod.rs",
            "render/",
            "render_model/",
            "sections/",
        ]),
        "report tree must stay decomposed by rendering concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/report/build/report_sections")),
        entries([
            "data_contract_validation.rs",
            "key_findings.rs",
            "mod.rs",
            "pipeline_overview.rs",
        ]),
        "report section builders must stay split by durable output concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/report/render_model")),
        entries(["construction.rs", "mod.rs"]),
        "render model tree must separate contracts from construction"
    );
}

fn assert_test_layout(root: &Path) {
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
