use std::collections::BTreeSet;

fn read_public_modules() -> BTreeSet<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("PUBLIC_API.md");
    let content = std::fs::read_to_string(&path).expect("read PUBLIC_API.md");
    let mut modules = BTreeSet::new();
    let mut in_section = false;
    for line in content.lines() {
        if line.trim() == "## Public Modules" {
            in_section = true;
            continue;
        }
        if in_section && line.starts_with("## ") {
            break;
        }
        if in_section {
            if let Some(rest) = line.trim().strip_prefix("- `") {
                if let Some(name) = rest.strip_suffix('`') {
                    modules.insert(name.to_string());
                }
            }
        }
    }
    modules
}

fn read_lib_pub_mods() -> BTreeSet<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/lib.rs");
    let content = std::fs::read_to_string(&path).expect("read lib.rs");
    let mut modules = BTreeSet::new();
    for line in content.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("pub mod ") {
            let name = rest.trim_end_matches(';').trim();
            modules.insert(name.to_string());
        }
    }
    modules
}

#[test]
fn public_modules_match_public_api_doc() {
    let declared = read_public_modules();
    let actual = read_lib_pub_mods();
    assert!(
        declared == actual,
        "PUBLIC_API.md public modules must match lib.rs pub mods.\n\
Update PUBLIC_API.md or make modules pub(crate) to align.\n\
Declared: {declared:?}\nActual: {actual:?}"
    );
}
