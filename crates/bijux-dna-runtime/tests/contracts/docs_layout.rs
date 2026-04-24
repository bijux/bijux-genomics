use std::fs;

fn crate_root() -> std::path::PathBuf {
    crate::support::crate_root("bijux-dna-runtime")
        .unwrap_or_else(|err| panic!("resolve runtime crate root: {err}"))
}

#[test]
fn readme_points_to_existing_runtime_tests() {
    let readme = fs::read_to_string(crate_root().join("README.md"))
        .unwrap_or_else(|err| panic!("read README.md: {err}"));
    assert!(
        readme.contains("tests/contracts/reference/reference_example.rs"),
        "README must reference the contract reference example path"
    );
    assert!(
        readme.contains("tests/schemas/schema/runtime_schema_snapshots.rs"),
        "README must reference the schema snapshot path"
    );
}

#[test]
fn test_docs_match_runtime_test_tree() {
    let docs = fs::read_to_string(crate_root().join("docs/TESTS.md"))
        .unwrap_or_else(|err| panic!("read docs/TESTS.md: {err}"));
    for expected in [
        "tests/boundaries/guardrails.rs",
        "tests/contracts/docs_layout.rs",
        "tests/contracts/reference/reference_example.rs",
        "tests/contracts/reference/docs_reference_example.rs",
        "tests/schemas/schema/runtime_schema_snapshots.rs",
    ] {
        assert!(docs.contains(expected), "docs/TESTS.md must reference {expected}");
    }
}

#[test]
fn public_api_doc_matches_root_modules() {
    let public_api = fs::read_to_string(crate_root().join("PUBLIC_API.md"))
        .unwrap_or_else(|err| panic!("read PUBLIC_API.md: {err}"));
    for module_name in [
        "environment",
        "manifests",
        "observability",
        "provenance",
        "recording",
        "run",
        "run_layout",
        "runner",
        "telemetry",
    ] {
        assert!(
            public_api.contains(&format!("`{module_name}`")),
            "PUBLIC_API.md must list `{module_name}`"
        );
    }
    assert!(
        !public_api.contains("stage_runner_contract"),
        "PUBLIC_API.md must not reference removed pseudo-modules"
    );
}
