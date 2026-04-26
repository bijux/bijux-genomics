use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn stage_contract_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    assert_root_layout(&root);
    assert_docs_layout(&root);
    assert_source_layout(&root);
    assert_test_layout(&root);
}

fn assert_root_layout(root: &Path) {
    assert_eq!(
        dir_entries(root),
        entries(["Cargo.toml", "README.md", "docs/", "src/", "tests/"]),
        "crate root must stay minimal: root README plus Cargo, docs, src, and tests only"
    );

    let misplaced_markdown = markdown_files_outside_docs(root);
    assert_eq!(
        misplaced_markdown,
        vec!["README.md".to_string()],
        "crate markdown outside docs/ must be limited to root README.md"
    );
}

fn assert_docs_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("docs")),
        entries([
            "ARCHITECTURE.md",
            "BOUNDARY.md",
            "CHANGE_RULES.md",
            "COMMANDS.md",
            "CONTRACT.md",
            "EFFECTS.md",
            "EXAMPLE_PLAN.json",
            "INDEX.md",
            "PUBLIC_API.md",
            "TESTS.md",
        ]),
        "docs must stay at the 10-document allowance"
    );
}

fn assert_source_layout(root: &Path) {
    assert_eq!(
        dir_entries(&root.join("src")),
        entries([
            "execution_plan/",
            "executor_registry/",
            "lib.rs",
            "plan_run/",
            "stage_plan/",
            "stage_plugin.rs",
        ]),
        "src must stay grouped by stage-contract concern"
    );
    assert_eq!(
        dir_entries(&root.join("src/execution_plan")),
        entries(["edge.rs", "mod.rs", "model.rs", "support.rs", "validation.rs"]),
        "execution_plan must separate edge, model, support, and validation"
    );
    assert_eq!(
        dir_entries(&root.join("src/executor_registry")),
        entries(["catalog/", "lookup.rs", "mod.rs", "types.rs"]),
        "executor_registry must separate catalog data, lookup, and types"
    );
    assert_eq!(
        dir_entries(&root.join("src/executor_registry/catalog")),
        entries(["executors.rs", "mod.rs"]),
        "executor catalog must keep executor labels separate from entries"
    );
    assert_eq!(
        dir_entries(&root.join("src/plan_run")),
        entries(["artifact_catalog.rs", "mod.rs", "planner_contract.rs", "stage_builder.rs"]),
        "plan_run must separate artifact mapping, planner projection, and stage building"
    );
    assert_eq!(
        dir_entries(&root.join("src/stage_plan")),
        entries(["contract.rs", "execution_step.rs", "json.rs", "mod.rs", "reason.rs"]),
        "stage_plan must separate contract model, JSON projection, execution-step projection, and reason types"
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
            "support/",
        ]),
        "tests must stay grouped by enduring suite intent"
    );
    assert_eq!(
        dir_entries(&root.join("tests/support")),
        entries(["workspace_paths.rs"]),
        "test support must keep shared helpers out of suite roots"
    );
    assert_eq!(
        dir_entries(&root.join("tests/boundaries")),
        entries(["architecture_tree.rs", "guardrails/", "guardrails.rs"]),
        "boundary tests must stay explicit"
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .filter(|entry| entry.file_name().to_string_lossy() != ".DS_Store")
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

fn entries<const N: usize>(expected: [&str; N]) -> BTreeSet<String> {
    expected.into_iter().map(str::to_string).collect()
}

fn markdown_files_outside_docs(root: &Path) -> Vec<String> {
    let mut files = walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().is_some_and(|extension| extension == "md"))
        .filter_map(|entry| {
            let rel = entry
                .path()
                .strip_prefix(root)
                .unwrap_or_else(|err| panic!("strip {}: {err}", entry.path().display()))
                .to_string_lossy()
                .replace('\\', "/");
            (!rel.starts_with("docs/")).then_some(rel)
        })
        .collect::<Vec<_>>();
    files.sort();
    files
}
