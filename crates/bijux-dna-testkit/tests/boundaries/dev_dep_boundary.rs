use std::fs;
use std::path::Path;

#[test]
fn testkit_is_only_dev_dependency() {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root");
    let crates_dir = workspace_root.join("crates");
    for entry in fs::read_dir(&crates_dir).expect("read crates dir") {
        let path = entry.expect("dir entry").path();
        if !path.is_dir() {
            continue;
        }
        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            continue;
        }
        let crate_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("<unknown>");
        if crate_name == "bijux-dna-testkit" {
            continue;
        }
        let content = fs::read_to_string(&cargo_toml).expect("read Cargo.toml");
        let mut in_dependencies = false;
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') {
                in_dependencies = trimmed == "[dependencies]";
                continue;
            }
            if in_dependencies && trimmed.starts_with("bijux-dna-testkit") {
                panic!("{crate_name} depends on bijux-dna-testkit as a production dependency");
            }
        }
    }
}
