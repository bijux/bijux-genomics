use std::path::Path;

#[test]
fn commands_doc_lists_managed_cli_commands() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands = read_to_string(&root.join("docs/COMMANDS.md"));
    let cli_args = read_to_string(&root.join("src/cli/args.rs"));

    let cli_commands = table_keys_after_heading(&commands, "## Managed CLI Commands");
    assert_eq!(cli_commands, ["query", "download"], "COMMANDS.md must list CLI commands");

    for variant in ["Query", "Download"] {
        assert!(cli_args.contains(variant), "cli args must expose `{variant}` command variant");
    }
}

#[test]
fn commands_doc_lists_managed_library_operations() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let commands = read_to_string(&root.join("docs/COMMANDS.md"));
    let operations = table_keys_after_heading(&commands, "## Managed Library Operations");
    let expected = [
        "build-filereport-url",
        "parse-filereport-tsv",
        "fetch-records",
        "normalize-query",
        "validate-query",
        "select-record-urls",
        "build-download-tasks",
        "download-tasks",
        "write-manifest",
    ];

    assert_eq!(operations, expected, "COMMANDS.md must list library operations in stable order");
}

fn table_keys_after_heading<const N: usize>(content: &str, heading: &str) -> [String; N] {
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
        if key.is_empty()
            || key == "Command"
            || key == "Operation"
            || key.chars().all(|ch| ch == '-')
        {
            continue;
        }
        keys.push(key.to_string());
    }
    keys.try_into().unwrap_or_else(|keys: Vec<String>| {
        panic!("expected {N} keys under {heading}, got {}", keys.len())
    })
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
