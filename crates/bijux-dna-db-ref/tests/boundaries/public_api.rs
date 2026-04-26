use std::path::Path;

#[test]
fn public_api_doc_matches_root_public_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api = read_to_string(&root.join("docs/PUBLIC_API.md"));
    let lib = read_to_string(&root.join("src/lib.rs"));

    let documented = bullet_items_after_heading(&public_api, "## Public Root Modules");
    let actual = lib
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|rest| rest.trim_end_matches(';').to_string())
        .collect::<Vec<_>>();

    assert_eq!(documented, actual, "PUBLIC_API.md must match lib.rs public modules");
}

#[test]
fn public_api_doc_mentions_curated_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api = read_to_string(&root.join("docs/PUBLIC_API.md"));
    let surface = read_to_string(&root.join("src/public_api/mod.rs"));

    for required in [
        "resolve_species_context",
        "resolve_reference_bundle",
        "resolve_panel",
        "resolve_map",
        "resolve_species_alias",
        "normalize_contig_name",
        "enforce_declared_build_and_contigs",
        "validate_imputation_tool_compatibility",
        "RuntimeRefService",
        "ref_service",
    ] {
        assert!(public_api.contains(required), "PUBLIC_API.md must mention `{required}`");
        assert!(surface.contains(required), "src/public_api/mod.rs must export `{required}`");
    }
}

fn bullet_items_after_heading(content: &str, heading: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed == heading {
            in_section = true;
            continue;
        }
        if in_section && trimmed.starts_with("## ") {
            break;
        }
        if in_section {
            if let Some(item) = trimmed.strip_prefix("- `").and_then(|rest| rest.strip_suffix('`'))
            {
                items.push(item.to_string());
            }
        }
    }
    items
}

fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
