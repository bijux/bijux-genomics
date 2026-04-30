#![allow(non_snake_case)]
use std::collections::BTreeSet;
use std::path::PathBuf;

use cargo_metadata::MetadataCommand;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__cargo_metadata_snapshot_policy__workspace_dependency_snapshot_matches_committed_contract(
) {
    let root = repo_root();
    let expected_path =
        root.join("crates/bijux-dna-policies/tests/fixtures/cargo_metadata_snapshot/workspace-deps.txt");
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|err| panic!("read {}: {err}", expected_path.display()));

    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()
        .unwrap_or_else(|err| panic!("cargo metadata: {err}"));
    let workspace_names =
        metadata.packages.iter().map(|package| package.name.clone()).collect::<BTreeSet<_>>();

    let mut lines = metadata
        .packages
        .into_iter()
        .map(|package| {
            let deps = package
                .dependencies
                .into_iter()
                .filter(|dependency| workspace_names.contains(&dependency.name))
                .map(|dependency| dependency.name)
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect::<Vec<_>>()
                .join(" ");
            format!("{}\t{}", package.name, deps)
        })
        .collect::<Vec<_>>();
    lines.sort();
    let actual = format!("{}\n", lines.join("\n"));

    bijux_dna_policies::policy_assert!(
        actual == expected,
        "workspace cargo metadata snapshot is stale.\nrefresh with:\n\
         cargo metadata --manifest-path Cargo.toml --format-version 1 --no-deps \\\n+           | jq -r '.packages[] | .name as $$name | [$$name, ((.dependencies // []) | map(select(.path != null) | .name) | sort | join(\" \"))] | @tsv' \\\n+           | sort > crates/bijux-dna-policies/tests/fixtures/cargo_metadata_snapshot/workspace-deps.txt"
    );
}
