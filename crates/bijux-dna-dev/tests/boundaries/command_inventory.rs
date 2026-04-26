use std::collections::BTreeSet;

#[test]
fn command_inventory_matches_catalog_ids() {
    let root = crate::support::crate_root("bijux-dna-dev")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));

    let docs = std::fs::read_to_string(root.join("docs/COMMANDS.md"))
        .unwrap_or_else(|err| panic!("read docs/COMMANDS.md: {err}"));

    for heading in [
        "### `assets run <id>`",
        "### `checks run <id>`",
        "### `containers run <id>`",
        "### `docs run <id>`",
        "### `domain run <id>`",
        "### `examples run <id>`",
        "### `hpc run <id>`",
        "### `lab run <id>`",
        "### `smoke run <id>`",
        "### `test run <id>`",
        "### `tooling run <id>`",
    ] {
        assert!(docs.contains(heading), "missing command inventory heading `{heading}`");
    }

    let docs_ids = command_ids_from_markdown(&docs);
    let catalog_ids = command_ids_from_catalogs(&root);

    assert_eq!(
        docs_ids, catalog_ids,
        "docs/COMMANDS.md must list the same command ids as src/catalog"
    );
}

fn command_ids_from_markdown(docs: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    let mut in_block = false;

    for line in docs.lines() {
        let line = line.trim();
        if line == "```text" {
            in_block = true;
            continue;
        }
        if line == "```" {
            in_block = false;
            continue;
        }
        if in_block && !line.is_empty() {
            ids.insert(line.to_string());
        }
    }

    ids
}

fn command_ids_from_catalogs(root: &std::path::Path) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    collect_catalog_ids(
        &root.join("src/catalog/checks.rs"),
        &["native", "policy", "policy_alias", "process", "composite"],
        &mut ids,
    );
    for path in ["containers.rs", "domain.rs", "ops.rs"] {
        collect_catalog_ids(&root.join("src/catalog").join(path), &["native"], &mut ids);
    }
    collect_literal_ids(&root.join("src/catalog/checks.rs"), &mut ids);
    ids
}

fn collect_catalog_ids(path: &std::path::Path, functions: &[&str], ids: &mut BTreeSet<String>) {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    for function in functions {
        let mut remainder = source.as_str();
        let call_prefix = format!("{function}(");
        while let Some(index) = remainder.find(&call_prefix) {
            let after_call = &remainder[index + call_prefix.len()..];
            let after_space = after_call.trim_start();
            if let Some(after_quote) = after_space.strip_prefix('"') {
                if let Some((id, rest)) = after_quote.split_once('"') {
                    ids.insert(id.to_string());
                    remainder = rest;
                    continue;
                }
            }
            remainder = after_call;
        }
    }
}

fn collect_literal_ids(path: &std::path::Path, ids: &mut BTreeSet<String>) {
    let source = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
    for line in source.lines() {
        let line = line.trim();
        if let Some(after_quote) = line.strip_prefix("id: \"") {
            if let Some(id) = after_quote.split('"').next() {
                ids.insert(id.to_string());
            }
        }
    }
}
