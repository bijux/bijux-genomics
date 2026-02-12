use std::fs;
use std::path::Path;

#[test]
fn top_level_modules_have_owner() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");
    let mut modules = Vec::new();
    let Ok(entries) = fs::read_dir(&src_dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.file_name().and_then(|n| n.to_str()) == Some("lib.rs") {
            continue;
        }
        if path.is_dir() {
            let mod_rs = path.join("mod.rs");
            if mod_rs.exists() {
                modules.push(mod_rs);
            }
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            modules.push(path);
        }
    }

    let mut offenders = Vec::new();
    for module in modules {
        let Ok(contents) = fs::read_to_string(&module) else {
            continue;
        };
        let mut has_owner = false;
        for line in contents.lines().take(8) {
            if line.trim().starts_with("//!") && line.contains("Owner:") {
                has_owner = true;
                break;
            }
            if !line.trim().is_empty() && !line.trim().starts_with("//!") {
                break;
            }
        }
        if !has_owner {
            offenders.push(module.display().to_string());
        }
    }

    assert!(
        offenders.is_empty(),
        "missing module owner doc comments: {offenders:?}"
    );
}
