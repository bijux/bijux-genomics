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
    let resolve = metadata.resolve.as_ref().unwrap_or_else(|| panic!("resolve graph missing"));
    let node = resolve
        .nodes
        .iter()
        .find(|node| node.id == runner.id)
        .unwrap_or_else(|| panic!("runner node missing"));
    let has_edge = node.deps.iter().any(|dep| dep.pkg == engine.id);
    assert!(!has_edge, "bijux-dna-runner must not depend on bijux-dna-engine");
}

#[test]
#[allow(clippy::too_many_lines)]
fn runner_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-runner")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> = ["Cargo.toml", "README.md", "docs/", "src/", "tests/"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(root_entries, expected_root, "runner crate root must stay minimal and intentional");

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> = [
        "backend/",
        "command_runner/",
        "command_runner.rs",
        "lib.rs",
        "public_api/",
        "repo_root/",
        "runner_driver/",
        "step_runner/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(src_entries, expected_src, "runner src tree must match the documented architecture");

    let command_runner_entries = dir_entries(&root.join("src/command_runner"));
    let expected_command_runner: BTreeSet<_> =
        ["command_line.rs", "command_output.rs", "invocation_identity.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        command_runner_entries, expected_command_runner,
        "runner command_runner support tree must stay minimal"
    );

    let public_api_entries = dir_entries(&root.join("src/public_api"));
    let expected_public_api: BTreeSet<_> =
        ["mod.rs", "stable_surface.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(public_api_entries, expected_public_api, "runner public api tree must stay curated");

    let backend_entries = dir_entries(&root.join("src/backend"));
    let expected_backend: BTreeSet<_> =
        ["docker/", "facade.rs", "kinds.rs", "mod.rs", "stable_surface.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(backend_entries, expected_backend, "runner backend tree must stay focused");

    let docker_entries = dir_entries(&root.join("src/backend/docker"));
    let expected_docker: BTreeSet<_> = [
        "execution_spec.rs",
        "executor/",
        "executor.rs",
        "facade.rs",
        "image_resolution/",
        "image_resolution.rs",
        "mod.rs",
        "replay.rs",
        "stable_surface.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        docker_entries, expected_docker,
        "runner docker backend tree must match its execution responsibilities"
    );

    let docker_executor_entries = dir_entries(&root.join("src/backend/docker/executor"));
    let expected_docker_executor: BTreeSet<_> =
        ["command_line.rs", "internal_contracts.rs", "lifecycle.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        docker_executor_entries, expected_docker_executor,
        "runner docker executor support tree must stay focused"
    );

    let repo_root_entries = dir_entries(&root.join("src/repo_root"));
    let expected_repo_root: BTreeSet<_> = ["env_override.rs", "mod.rs", "root_detection.rs"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(
        repo_root_entries, expected_repo_root,
        "runner repo_root tree must keep override lookup separate from root detection"
    );

    let runner_driver_entries = dir_entries(&root.join("src/runner_driver"));
    let expected_runner_driver: BTreeSet<_> =
        ["artifact_collection.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        runner_driver_entries, expected_runner_driver,
        "runner driver tree must keep artifact collection separate from driver orchestration"
    );

    let step_runner_entries = dir_entries(&root.join("src/step_runner"));
    let expected_step_runner: BTreeSet<_> = [
        "apptainer_args.rs",
        "apptainer_execution.rs",
        "artifacts.rs",
        "command_template.rs",
        "contracts.rs",
        "docker_execution.rs",
        "effects.rs",
        "execution_dispatch.rs",
        "execution_outcome.rs",
        "identity.rs",
        "inputs.rs",
        "internal_contracts.rs",
        "mod.rs",
        "observer.rs",
        "records.rs",
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
