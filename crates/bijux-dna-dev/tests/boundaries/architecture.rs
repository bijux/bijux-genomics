#![allow(clippy::too_many_lines)]

use std::collections::BTreeSet;

#[test]
fn dev_tree_matches_architecture_contract() {
    let root = crate::support::crate_root("bijux-dna-dev")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let root_entries = dir_entries(&root);
    let expected_root: BTreeSet<_> = ["Cargo.toml", "README.md", "docs/", "src/", "tests/"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(root_entries, expected_root, "dev crate root must stay minimal and intentional");

    let docs_entries = dir_entries(&root.join("docs"));
    let expected_docs: BTreeSet<_> = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "CHANGE_RULES.md",
        "COMMANDS.md",
        "CONTRACTS.md",
        "DEPENDENCIES.md",
        "INDEX.md",
        "PUBLIC_API.md",
        "SCOPE.md",
        "TESTS.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(docs_entries, expected_docs, "dev crate docs must stay centralized under docs/");

    let src_entries = dir_entries(&root.join("src"));
    let expected_src: BTreeSet<_> = [
        "application/",
        "catalog/",
        "cli/",
        "commands/",
        "dev_entrypoint.rs",
        "main.rs",
        "model/",
        "runtime/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        src_entries, expected_src,
        "dev src tree must match the documented control-plane layout"
    );

    let application_entries = dir_entries(&root.join("src/application"));
    let expected_application: BTreeSet<_> =
        ["checks/", "checks.rs", "containers.rs", "domain.rs", "mod.rs", "ops.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        application_entries, expected_application,
        "dev application tree must stay decomposed by workflow concern"
    );

    let cli_entries = dir_entries(&root.join("src/cli"));
    let expected_cli: BTreeSet<_> =
        ["command_dispatch.rs", "execution_reporting.rs", "mod.rs", "runner.rs", "schema.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        cli_entries, expected_cli,
        "dev cli tree must stay explicit about routing and reporting"
    );

    let commands_entries = dir_entries(&root.join("src/commands"));
    let expected_commands: BTreeSet<_> = [
        "automation_boundary.rs",
        "checks.rs",
        "command_support.rs",
        "containers/",
        "domain/",
        "mod.rs",
        "native_dispatch.rs",
        "ops/",
        "repo_checks/",
        "repo_checks.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        commands_entries, expected_commands,
        "dev commands tree must stay partitioned by enduring automation surface"
    );

    let repo_checks_entries = dir_entries(&root.join("src/commands/repo_checks"));
    let expected_repo_checks: BTreeSet<_> =
        ["artifacts.rs", "governance.rs", "workspace_contracts.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        repo_checks_entries, expected_repo_checks,
        "repository checks must stay partitioned by concern"
    );

    let containers_entries = dir_entries(&root.join("src/commands/containers"));
    let expected_containers: BTreeSet<_> = [
        "command_support.rs",
        "content_support.rs",
        "dispatch.rs",
        "metadata.rs",
        "mod.rs",
        "registry_catalog.rs",
        "runtime/",
        "validation/",
        "version_state.rs",
        "versioning.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        containers_entries, expected_containers,
        "container commands must stay explicit about runtime and validation ownership"
    );

    let container_runtime_entries = dir_entries(&root.join("src/commands/containers/runtime"));
    let expected_container_runtime: BTreeSet<_> =
        ["frontend_proofs.rs", "mod.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        container_runtime_entries, expected_container_runtime,
        "container runtime support must keep frontend proofing separate"
    );

    let runtime_entries = dir_entries(&root.join("src/runtime"));
    let expected_runtime: BTreeSet<_> =
        ["mod.rs", "process.rs", "workspace.rs", "workspace_root.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        runtime_entries, expected_runtime,
        "dev runtime tree must stay focused on workspace and process boundaries"
    );

    let test_entries = dir_entries(&root.join("tests"));
    let expected_tests: BTreeSet<_> =
        ["boundaries/", "boundaries.rs", "guardrails.rs", "snapshots/", "support/"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(test_entries, expected_tests, "dev test tree must match the documented taxonomy");

    let support_entries = dir_entries(&root.join("tests/support"));
    let expected_support: BTreeSet<_> =
        ["workspace_paths.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(
        support_entries, expected_support,
        "dev test support must keep shared helpers out of suite roots"
    );

    let boundary_entries = dir_entries(&root.join("tests/boundaries"));
    let expected_boundaries: BTreeSet<_> = [
        "architecture.rs",
        "command_inventory.rs",
        "dependencies.rs",
        "docs_layout.rs",
        "guardrails.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        boundary_entries, expected_boundaries,
        "boundary tests must stay focused on architecture and ownership"
    );
}

#[test]
fn dev_markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = crate::support::crate_root("bijux-dna-dev")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let mut offenders = Vec::new();
    collect_markdown_outside_docs(&root, &root, &mut offenders);

    assert!(
        offenders.is_empty(),
        "crate markdown must be root README.md or live under docs/: {}",
        offenders.join(", ")
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

fn collect_markdown_outside_docs(
    root: &std::path::Path,
    path: &std::path::Path,
    offenders: &mut Vec<String>,
) {
    for entry in
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or_else(|err| {
            panic!("strip {} from {}: {err}", root.display(), path.display())
        });
        let rel_text = rel.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            if rel_text != "docs" {
                collect_markdown_outside_docs(root, &path, offenders);
            }
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "md")
            && rel_text != "README.md"
            && !rel_text.starts_with("docs/")
            && rel_text != "tests/snapshots/bijux-dna-dev__tooling__architecture_report.md"
        {
            offenders.push(rel_text);
        }
    }
}
