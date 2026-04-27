use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn public_api_docs_match_root_public_modules() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let lib = fs::read_to_string(root.join("src").join("lib.rs")).expect("read src/lib.rs");
    let docs = fs::read_to_string(root.join("docs").join("PUBLIC_API.md"))
        .expect("read docs/PUBLIC_API.md");

    assert_eq!(
        documented_public_modules(&docs),
        root_public_modules(&lib),
        "PUBLIC_API.md must list the exact public modules exported by src/lib.rs"
    );
}

fn root_public_modules(lib: &str) -> BTreeSet<String> {
    lib.lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("pub mod ")
                .and_then(|module| module.strip_suffix(';'))
                .map(str::to_string)
        })
        .collect()
}

fn documented_public_modules(docs: &str) -> BTreeSet<String> {
    let mut modules = BTreeSet::new();
    let mut in_public_modules = false;

    for line in docs.lines() {
        match line {
            "## Public Modules" => {
                in_public_modules = true;
            }
            line if line.starts_with("## ") => {
                in_public_modules = false;
            }
            line if in_public_modules => {
                if let Some(module) =
                    line.trim().strip_prefix("- `").and_then(|module| module.strip_suffix('`'))
                {
                    modules.insert(module.to_string());
                }
            }
            _ => {}
        }
    }

    modules
}
