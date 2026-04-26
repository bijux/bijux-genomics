use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn engine_tree_matches_architecture_contract() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_eq!(
        dir_entries(root),
        entries([
            "BOUNDARY.md",
            "Cargo.toml",
            "PUBLIC_API.md",
            "README.md",
            "clippy.toml",
            "docs/",
            "src/",
            "tests/",
        ]),
        "crate root must stay minimal and intentional"
    );

    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "control/",
            "engine_config/",
            "engine_driver.rs",
            "errors.rs",
            "executor/",
            "lib.rs",
            "observability/",
            "public_api/",
        ]),
        "src tree must match the documented engine layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/control")),
        entries(["cancellation_state.rs", "mod.rs", "token_contract.rs"]),
        "control tree must separate token contracts from state transitions"
    );

    assert_eq!(
        dir_entries(&root.join("src/engine_config")),
        entries(["graph_policy.rs", "mod.rs"]),
        "engine config tree must separate config contracts from graph policy application"
    );

    assert_eq!(
        dir_entries(&root.join("src/executor")),
        entries([
            "OWNER.toml",
            "contracts/",
            "facade.rs",
            "graph/",
            "mod.rs",
            "recording/",
            "step_execution/",
        ]),
        "executor tree must stay partitioned by execution concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/executor/graph")),
        entries(["mod.rs", "topology.rs"]),
        "executor graph tree must keep normalization and ordering together"
    );

    assert_eq!(
        dir_entries(&root.join("src/executor/contracts")),
        entries(["metrics.rs", "mod.rs", "outputs.rs", "run_artifacts.rs"]),
        "executor contract checks must stay partitioned by artifact concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/executor/recording")),
        entries(["mod.rs", "payload.rs", "writer.rs"]),
        "executor recording tree must keep payload and persistence separated"
    );

    assert_eq!(
        dir_entries(&root.join("src/executor/step_execution")),
        entries(["mod.rs", "stage_record.rs"]),
        "step execution tree must separate lifecycle orchestration from record shaping"
    );

    assert_eq!(
        dir_entries(&root.join("src/observability")),
        entries(["events.rs", "hooks.rs", "mod.rs"]),
        "observability tree must separate event contracts from hook contracts"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        entries(["OWNER.toml", "mod.rs", "stable_surface.rs"]),
        "public api tree must stay curated"
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
            "guardrails.rs",
            "schemas/",
            "support/",
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
