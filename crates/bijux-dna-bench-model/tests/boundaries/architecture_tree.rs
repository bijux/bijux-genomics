use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn bench_model_tree_matches_architecture_contract() {
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
            "compare/",
            "contract/",
            "diagnostics/",
            "lib.rs",
            "model/",
            "policy/",
            "public_api/",
            "stats/",
        ]),
        "src tree must match the documented benchmark model layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/compare")),
        entries(["diff.rs", "mod.rs", "report.rs", "stable_surface.rs", "stratify.rs"]),
        "compare tree must separate diff execution from report contracts"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract")),
        entries(["mod.rs", "records.rs", "schema_versions.rs", "suite/"]),
        "contract tree must separate record validators, schema ids, and suite rules"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/suite")),
        entries([
            "analysis.rs",
            "diversity.rs",
            "edge_ports.rs",
            "governance.rs",
            "graph.rs",
            "mod.rs",
            "param_bindings.rs",
            "validation/",
        ]),
        "suite contract tree must stay partitioned by enduring concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/contract/suite/validation")),
        entries([
            "declared_stage_nodes.rs",
            "edge_contracts.rs",
            "mod.rs",
            "stage_contracts.rs",
            "suite_validation.rs",
        ]),
        "suite validation tree must separate orchestration, shared node contracts, and rule families"
    );

    assert_eq!(
        dir_entries(&root.join("src/policy")),
        entries(["gate_policy/", "mod.rs", "outcomes.rs"]),
        "policy tree must separate evaluation from policy outcomes"
    );

    assert_eq!(
        dir_entries(&root.join("src/diagnostics")),
        entries(["error_taxonomy.rs", "mod.rs"]),
        "diagnostics tree must stay focused on stable error contracts"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["mod.rs", "stable_surface.rs"]),
        "public api tree must stay curated"
    );

    assert_eq!(
        dir_entries(&root.join("src/model/suite/support")),
        entries([
            "analysis_requirements.rs",
            "dataset_spec.rs",
            "diversity_requirements.rs",
            "mod.rs",
            "replicate_policy.rs",
            "stratification_requirement.rs",
        ]),
        "suite support tree must separate durable contract families"
    );

    assert_eq!(
        dir_entries(&root.join("src/stats/robust_estimators")),
        entries(["contracts.rs", "mod.rs"]),
        "robust estimators must separate typed stats contracts from estimator functions"
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
