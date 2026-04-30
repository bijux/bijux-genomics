use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn public_api_doc_matches_exported_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = read(root.join("docs/PUBLIC_API.md"));
    let lib = read(root.join("src/lib.rs"));
    let stable_surface = read(root.join("src/public_api/stable_surface.rs"));

    assert_eq!(
        section_items(&docs, "Public Modules"),
        entries(["backend", "command_runner", "public_api", "step_runner"]),
        "PUBLIC_API.md must list every public module from src/lib.rs"
    );

    for module in section_items(&docs, "Public Modules") {
        assert!(
            lib.contains(&format!("pub mod {module};")),
            "PUBLIC_API.md lists {module}, but src/lib.rs does not export it"
        );
    }

    assert_eq!(
        section_items(&docs, "Root Exports"),
        entries(["DockerRunner", "LocalRunner", "api"]),
        "PUBLIC_API.md must list root re-exports"
    );
    assert!(lib.contains("pub use public_api::api;"));
    assert!(lib.contains("pub use runner_driver::DockerRunner;"));
    assert!(lib.contains("pub use runner_driver::LocalRunner;"));

    for export in section_items(&docs, "Facade Exports") {
        assert!(
            stable_surface.contains(&export),
            "PUBLIC_API.md lists facade export {export}, but stable_surface.rs does not export it"
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn section_items(docs: &str, heading: &str) -> BTreeSet<String> {
    let mut in_section = false;
    let mut items = BTreeSet::new();

    for line in docs.lines() {
        if line == format!("## {heading}") {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            if let Some(item) =
                line.trim().strip_prefix("- `").and_then(|item| item.strip_suffix('`'))
            {
                items.insert(item.to_string());
            }
        }
    }

    items
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
