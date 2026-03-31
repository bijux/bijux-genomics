use cargo_metadata::MetadataCommand;
use std::collections::BTreeSet;

#[test]
fn runner_has_no_engine_dependency() {
    let metadata = MetadataCommand::default()
        .exec()
        .unwrap_or_else(|err| panic!("load cargo metadata: {err}"));
    let runner = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-dna-runner")
        .unwrap_or_else(|| panic!("bijux-dna-runner missing"));
    let engine = metadata
        .packages
        .iter()
        .find(|pkg| pkg.name == "bijux-dna-engine")
        .unwrap_or_else(|| panic!("bijux-dna-engine missing"));
    let resolve = metadata
        .resolve
        .as_ref()
        .unwrap_or_else(|| panic!("resolve graph missing"));
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == runner.id)
        .unwrap_or_else(|| panic!("runner node missing"));
    let has_edge = node.deps.iter().any(|dep| dep.pkg == engine.id);
    assert!(
        !has_edge,
        "bijux-dna-runner must not depend on bijux-dna-engine"
    );
}

#[test]
fn runner_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> = [
        "BOUNDARY.md",
        "Cargo.toml",
        "PUBLIC_API.md",
        "README.md",
        "docs/",
        "src/",
        "tests/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        root_entries, expected_root,
        "runner crate root must stay minimal and intentional"
    );

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> = [
        "backend/",
        "command_runner.rs",
        "lib.rs",
        "repo_root.rs",
        "step_runner/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        src_entries, expected_src,
        "runner src tree must match the documented architecture"
    );

    let backend_entries = dir_entries(&root.join("src/backend"));
    let expected_backend: BTreeSet<_> = ["docker/", "kinds.rs", "mod.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        backend_entries, expected_backend,
        "runner backend tree must stay focused"
    );

    let docker_entries = dir_entries(&root.join("src/backend/docker"));
    let expected_docker: BTreeSet<_> = [
        "execution_spec.rs",
        "executor/",
        "executor.rs",
        "image_resolution/",
        "image_resolution.rs",
        "mod.rs",
        "replay.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        docker_entries, expected_docker,
        "runner docker backend tree must match its execution responsibilities"
    );

    let step_runner_entries = dir_entries(&root.join("src/step_runner"));
    let expected_step_runner: BTreeSet<_> = [
        "apptainer_args.rs",
        "apptainer_execution.rs",
        "artifacts.rs",
        "command_template.rs",
        "contracts.rs",
        "docker_execution.rs",
        "execution_outcome.rs",
        "identity.rs",
        "inputs.rs",
        "mod.rs",
        "observer.rs",
        "runtime_policy.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        step_runner_entries, expected_step_runner,
        "runner step_runner tree must remain decomposed by concern"
    );
}

fn dir_entries(path: &std::path::Path) -> BTreeSet<String> {
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
