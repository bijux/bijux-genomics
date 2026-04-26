use std::path::Path;

#[test]
fn commands_doc_lists_managed_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands = read_to_string(&root.join("docs/COMMANDS.md"));
    let operations = table_keys_after_heading(&commands, "## Managed Library Operations");
    let expected = [
        "resolve-species-alias",
        "resolve-species-authority",
        "resolve-species-context",
        "resolve-contig-map",
        "enforce-build-contigs",
        "resolve-reference-bank",
        "resolve-reference-bundle",
        "normalize-contig-name",
        "reference-provenance",
        "resolve-genetic-map-bank",
        "resolve-organellar-policy",
        "resolve-default-reference-set",
        "resolve-sex-chromosome-rule",
        "resolve-coverage-profile",
        "resolve-panel",
        "resolve-panel-lock",
        "resolve-map",
        "resolve-map-lock",
        "validate-imputation-tool-compatibility",
        "ref-service",
    ];

    assert_eq!(operations, expected, "COMMANDS.md must list resolver operations in stable order");
}

fn table_keys_after_heading(content: &str, heading: &str) -> Vec<String> {
    let mut keys = Vec::new();
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
        if !in_section {
            continue;
        }
        let cells = markdown_table_cells(trimmed);
        if cells.len() < 3 {
            continue;
        }
        let key = cells[0].trim_matches('`');
        if key.is_empty() || key == "Operation" || key.chars().all(|ch| ch == '-') {
            continue;
        }
        keys.push(key.to_string());
    }
    keys
}

fn markdown_table_cells(line: &str) -> Vec<&str> {
    if !line.starts_with('|') || !line.ends_with('|') {
        return Vec::new();
    }
    line.trim_matches('|').split('|').map(str::trim).collect()
}

fn read_to_string(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
