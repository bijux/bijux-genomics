use std::fs;
use std::path::PathBuf;

fn read_public_modules() -> Vec<String> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("PUBLIC_API.md");
    let content = fs::read_to_string(path).expect("read PUBLIC_API.md");
    let mut modules = Vec::new();
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
                    modules.push(name.to_string());
                }
            }
        }
    }
    modules.sort();
    modules
}

#[test]
fn public_surface_matches_public_api() -> anyhow::Result<()> {
    let lib_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("lib.rs");
    let source = fs::read_to_string(lib_path)?;
    let mut pub_mods = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                pub_mods.push(name.to_string());
            }
        }
    }
    pub_mods.sort();
    let declared = read_public_modules();
    assert_eq!(
        pub_mods, declared,
        "Public modules must match PUBLIC_API.md.\nUpdate PUBLIC_API.md or make modules pub(crate)."
    );
    Ok(())
}
