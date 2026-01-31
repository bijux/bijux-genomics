use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[test]
fn module_file_budget() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let src_dir = manifest_dir.join("src");

    let allowlist: HashMap<&'static str, usize> = HashMap::new();
    // Temporary exceptions go here (cap overrides only; include expiry in comment).
    // allowlist.insert("report", 14); // expires: 2026-06-01

    let soft_cap = 12usize;
    let hard_cap = 20usize;
    let mut offenders = Vec::new();
    let mut warnings = Vec::new();

    let Ok(entries) = fs::read_dir(&src_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let Some(module_name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        let mut count = 0usize;
        let Ok(files) = fs::read_dir(&path) else {
            continue;
        };
        for file in files.flatten() {
            let file_path = file.path();
            if file_path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                continue;
            }
            if file_path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
                continue;
            }
            count += 1;
        }

        let cap = allowlist.get(module_name).copied().unwrap_or(hard_cap);
        if count > soft_cap {
            warnings.push(format!(
                "{module_name} has {count} files; soft cap {soft_cap} -> consolidate {module_name}/*.rs",
            ));
        }
        if count > cap {
            let hint = format!(
                "{module_name} has {count} files, max {cap} -> consolidate {module_name}/*.rs",
            );
            offenders.push(hint);
        }
    }

    if !warnings.is_empty() {
        eprintln!("module file budget warnings:\n{}", warnings.join("\n"));
    }
    assert!(
        offenders.is_empty(),
        "module file budget exceeded:\n{}",
        offenders.join("\n")
    );
}
