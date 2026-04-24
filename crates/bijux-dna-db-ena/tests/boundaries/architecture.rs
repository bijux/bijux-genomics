use std::collections::BTreeSet;

#[test]
fn db_ena_tree_matches_architecture_contract() {
    let root = crate_root("bijux-dna-db-ena");

    assert_eq!(
        dir_entries(&root),
        btree_set(&[
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
        btree_set(&[
            "cli/",
            "cli_entrypoint.rs",
            "client/",
            "download/",
            "lib.rs",
            "main.rs",
            "manifest_store.rs",
            "model/",
            "public_api/",
        ]),
        "src tree must match the documented db-ena layout"
    );

    assert_eq!(
        dir_entries(&root.join("src/cli")),
        btree_set(&["args.rs", "commands/", "mod.rs"]),
        "cli tree must remain decomposed by parsing and command assembly"
    );

    assert_eq!(
        dir_entries(&root.join("src/cli/commands")),
        btree_set(&["download.rs", "mod.rs", "query.rs"]),
        "cli commands must remain decomposed by query and download concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/client")),
        btree_set(&["error.rs", "filereport/", "mod.rs"]),
        "client tree must remain decomposed by error and filereport concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/client/filereport")),
        btree_set(&["headers.rs", "mod.rs", "request.rs", "rows.rs"]),
        "filereport tree must remain decomposed by request, headers, and row decoding"
    );

    assert_eq!(
        dir_entries(&root.join("src/download")),
        btree_set(&[
            "config.rs",
            "mod.rs",
            "output_layout.rs",
            "plan.rs",
            "report.rs",
            "runtime.rs",
            "task.rs",
            "transfer.rs",
        ]),
        "download tree must remain decomposed by config, planning, runtime, transfer, and report concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/model")),
        btree_set(&[
            "manifest.rs",
            "mod.rs",
            "query.rs",
            "record.rs",
            "source_selection.rs",
        ]),
        "model tree must remain decomposed by manifest, query, record, and source-selection concern"
    );

    assert_eq!(
        dir_entries(&root.join("src/public_api")),
        btree_set(&["mod.rs"]),
        "public api tree must remain curated"
    );
}

fn crate_root(crate_name: &str) -> std::path::PathBuf {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let actual = root.file_name().and_then(|name| name.to_str()).unwrap_or_default();
    assert_eq!(actual, crate_name, "unexpected integration-test crate root");
    root
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

fn btree_set(entries: &[&str]) -> BTreeSet<String> {
    entries.iter().map(|entry| (*entry).to_string()).collect()
}
