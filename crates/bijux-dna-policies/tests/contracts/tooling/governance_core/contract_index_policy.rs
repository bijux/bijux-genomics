#![allow(non_snake_case)]

use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .unwrap_or_else(|| bijux_dna_policies::policy_panic!("resolve repository root"))
        .to_path_buf()
}

fn indexed_contract_rows(content: &str) -> Vec<(&str, &str)> {
    content
        .lines()
        .filter_map(|line| {
            let trimmed = line.trim();
            if !trimmed.starts_with('|') || trimmed.contains("---") {
                return None;
            }
            let cells = trimmed.trim_matches('|').split('|').map(str::trim).collect::<Vec<_>>();
            if cells.len() != 2 || cells[0] == "Contract category" {
                return None;
            }
            let authority = authority_path(cells[1]);
            Some((cells[0], authority))
        })
        .collect()
}

fn authority_path(cell: &str) -> &str {
    let trimmed = cell.trim().trim_matches('`');
    if let Some(rest) = trimmed.strip_prefix('[') {
        if let Some((_, suffix)) = rest.split_once("](") {
            if let Some((target, "")) = suffix.rsplit_once(')') {
                return target;
            }
        }
    }
    trimmed
}

#[test]
fn policy__contracts__contract_index_policy__index_covers_architecture_boundaries() {
    let root = repo_root();
    let path = root.join("docs/10-architecture/CONTRACT_INDEX.md");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));
    let rows = indexed_contract_rows(&content);
    let required = [
        "root architecture map",
        "crate boundaries",
        "dependency boundaries",
        "contract artifact spine",
        "generated files boundary",
        "dry-run side-effect boundary",
        "snapshot/golden boundary",
        "fixture boundary",
        "tool invocation boundary",
        "effective config boundary",
        "asset contract boundary",
        "container contract boundary",
        "science evidence boundary",
        "HPC boundary",
        "network boundary",
        "privacy boundary",
        "waiver boundary",
        "review ownership boundary",
    ];
    let mut missing = Vec::new();
    for category in required {
        if !rows.iter().any(|(seen, _)| *seen == category) {
            missing.push(category.to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "CONTRACT_INDEX.md is missing required architecture categories:\n{}",
        missing.join("\n")
    );
}

#[test]
fn policy__contracts__contract_index_policy__index_authorities_exist() {
    let root = repo_root();
    let path = root.join("docs/10-architecture/CONTRACT_INDEX.md");
    let doc_root = path
        .parent()
        .unwrap_or_else(|| bijux_dna_policies::policy_panic!("resolve {}", path.display()));
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| bijux_dna_policies::policy_panic!("read {}: {err}", path.display()));
    let missing = indexed_contract_rows(&content)
        .into_iter()
        .filter(|(_, authority)| !doc_root.join(authority).exists())
        .map(|(category, authority)| format!("{category}: {authority}"))
        .collect::<Vec<_>>();

    bijux_dna_policies::policy_assert!(
        missing.is_empty(),
        "CONTRACT_INDEX.md references missing authority paths:\n{}",
        missing.join("\n")
    );
}
