use std::path::Path;

#[test]
fn public_api_doc_matches_library_root_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api = read_to_string(&root.join("docs/PUBLIC_API.md"));
    let lib = read_to_string(&root.join("src/lib.rs"));

    let documented = bullet_items_after_line(
        &public_api,
        "The crate root preserves stable module paths for downstream callers:",
    );
    let actual = lib
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|rest| rest.trim_end_matches(';').to_string())
        .collect::<Vec<_>>();

    assert_eq!(documented, actual, "PUBLIC_API.md must match lib.rs public modules");
}

#[test]
fn public_api_doc_matches_curated_reexports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api = read_to_string(&root.join("docs/PUBLIC_API.md"));
    let surface = read_to_string(&root.join("src/public_api/mod.rs"));

    for export in bullet_items_after_line(
        &public_api,
        "`src/public_api/mod.rs` curates ergonomic re-exports for the stable surface:",
    ) {
        assert!(
            surface.contains(&export),
            "documented public API export `{export}` must exist in src/public_api/mod.rs"
        );
    }
}

fn bullet_items_after_line(content: &str, marker: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_list = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == marker {
            in_list = true;
            continue;
        }
        if in_list && trimmed.starts_with("## ") {
            break;
        }
        if in_list {
            if let Some(item) = trimmed.strip_prefix("- `").and_then(|rest| rest.strip_suffix('`'))
            {
                items.push(item.to_string());
                continue;
            }
            if !items.is_empty() && !trimmed.starts_with("- ") {
                break;
            }
        }
    }
    items
}

fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
