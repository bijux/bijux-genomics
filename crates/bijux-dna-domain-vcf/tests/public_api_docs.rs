use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn public_api_docs_match_public_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = std::fs::read_to_string(root.join("src/lib.rs"))
        .unwrap_or_else(|err| panic!("read src/lib.rs: {err}"));
    let public_api = std::fs::read_to_string(root.join("docs/PUBLIC_API.md"))
        .unwrap_or_else(|err| panic!("read docs/PUBLIC_API.md: {err}"));

    let modules = public_modules(&source);
    let documented = documented_modules(&public_api);
    assert_eq!(documented, modules, "docs/PUBLIC_API.md must list public modules exactly");

    for section in ["Major Export Groups", "Stability Rules"] {
        assert!(
            public_api.contains(&format!("## {section}")),
            "docs/PUBLIC_API.md missing `{section}` section"
        );
    }
}

fn public_modules(source: &str) -> BTreeSet<String> {
    source
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .filter_map(|rest| rest.split([';', ' ']).next())
        .map(str::to_string)
        .collect()
}

fn documented_modules(public_api: &str) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    let mut in_modules = false;

    for line in public_api.lines() {
        if line.starts_with("## ") {
            in_modules = line == "## Public Modules";
            continue;
        }
        if in_modules {
            if let Some(module) =
                line.trim().strip_prefix("- `").and_then(|value| value.strip_suffix('`'))
            {
                modules.insert(module.to_string());
            }
        }
    }

    modules
}
