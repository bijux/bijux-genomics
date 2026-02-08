use std::path::Path;

use anyhow::Result;

fn read_allowed_pub_modules() -> Vec<String> {
    let readme = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");
    let content = std::fs::read_to_string(&readme)
        .unwrap_or_else(|err| panic!("read README.md at {}: {err}", readme.display()));
    let mut modules = Vec::new();
    let mut in_section = false;
    for line in content.lines() {
        if line.trim() == "## Allowed `pub` modules" {
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
fn core_scope_only_allows_contracts_and_foundation() -> Result<()> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src = root.join("src");
    let allow_dirs = ["contract", "foundation", "metrics"];
    let allow_files = ["id_catalog.rs", "ids.rs", "lib.rs", "prelude.rs"];

    for entry in std::fs::read_dir(&src)? {
        let entry = entry?;
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            assert!(
                allow_dirs.contains(&name.as_str()),
                "core scope violation: unexpected dir {name}"
            );
            continue;
        }
        if path.is_file() {
            assert!(
                allow_files.contains(&name.as_str()),
                "core scope violation: unexpected file {name}"
            );
        }
    }

    let lib_path = src.join("lib.rs");
    let content = std::fs::read_to_string(&lib_path)?;
    let mut pub_mods = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("pub mod ") {
            if let Some(name) = rest.split([';', ' ']).next() {
                pub_mods.push(name.to_string());
            }
        }
    }
    pub_mods.sort();
    let allowed_pub_mods = [
        "contract",
        "foundation",
        "id_catalog",
        "ids",
        "metrics",
        "prelude",
    ];
    let allowed_from_readme = read_allowed_pub_modules();
    let allowed_from_readme: Vec<&str> = allowed_from_readme.iter().map(String::as_str).collect();
    assert!(
        allowed_from_readme == allowed_pub_mods,
        "README allowed pub modules must match core policy.\n\
Update README or policy list to align.\n\
README: {allowed_from_readme:?}\nPolicy: {allowed_pub_mods:?}"
    );
    for module in pub_mods {
        assert!(
            allowed_pub_mods.contains(&module.as_str()),
            "core public module not allowed: {module}"
        );
    }
    Ok(())
}
