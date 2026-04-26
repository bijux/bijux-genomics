#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

fn crate_dirs(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let crates_dir = root.join("crates");
    let mut dirs = std::fs::read_dir(&crates_dir)
        .unwrap_or_else(|_| panic!("read {}", crates_dir.display()))
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|path| path.is_dir() && path.join("Cargo.toml").exists())
        .collect::<Vec<_>>();
    dirs.sort();
    dirs
}

fn crate_name(dir: &std::path::Path) -> String {
    dir.file_name().and_then(|name| name.to_str()).unwrap_or("<unknown>").to_string()
}

fn crate_doc_path(dir: &std::path::Path, name: &str) -> std::path::PathBuf {
    let root_doc = dir.join(name);
    if root_doc.exists() {
        return root_doc;
    }
    dir.join("docs").join(name)
}

#[test]
fn policy__contracts__boundary_docs_policy__every_crate_has_boundary_and_public_api_docs() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for dir in crate_dirs(&root) {
        let boundary = crate_doc_path(&dir, "BOUNDARY.md");
        let public_api = crate_doc_path(&dir, "PUBLIC_API.md");
        if !boundary.exists() {
            offenders.push(format!("missing {}", boundary.display()));
        }
        if !public_api.exists() {
            offenders.push(format!("missing {}", public_api.display()));
        }
    }
    assert!(
        offenders.is_empty(),
        "boundary/public-api docs policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__boundary_docs_policy__boundary_docs_declare_enforceable_contract_fields() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for dir in crate_dirs(&root) {
        let boundary = crate_doc_path(&dir, "BOUNDARY.md");
        let raw = std::fs::read_to_string(&boundary)
            .unwrap_or_else(|_| panic!("read {}", boundary.display()));
        if !raw.lines().any(|line| line.starts_with("# ")) {
            offenders.push(format!("{} missing H1", boundary.display()));
        }
        if crate_name(&dir) == "bijux-dna-policies" {
            for field in [
                "Owner:",
                "Scope:",
                "Allowed inputs:",
                "Forbidden dependencies:",
                "Forbidden effects:",
                "Validation command:",
            ] {
                if !raw.lines().any(|line| line.starts_with(field)) {
                    offenders.push(format!("{} missing `{field}`", boundary.display()));
                }
            }
            let expected_validation = format!("cargo test -p {} ", crate_name(&dir));
            if !raw.contains(&expected_validation) {
                offenders.push(format!(
                    "{} validation command must target its crate package",
                    boundary.display()
                ));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "boundary contract field policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__boundary_docs_policy__public_modules_must_be_listed_in_public_api_doc() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for dir in crate_dirs(&root) {
        let lib = dir.join("src/lib.rs");
        if !lib.exists() {
            continue;
        }
        let public_api = crate_doc_path(&dir, "PUBLIC_API.md");
        let public_api_raw = std::fs::read_to_string(&public_api)
            .unwrap_or_else(|_| panic!("read {}", public_api.display()));
        if !public_api_raw.contains("## Module Inventory") {
            continue;
        }
        let lib_raw =
            std::fs::read_to_string(&lib).unwrap_or_else(|_| panic!("read {}", lib.display()));
        for line in lib_raw.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("pub mod ") {
                let module = rest.split([';', ' ']).next().unwrap_or_default().trim().to_string();
                if module.is_empty() {
                    continue;
                }
                let token = format!("- {module}");
                if !public_api_raw.contains(&token) {
                    offenders.push(format!(
                        "{} missing module listing `{}` in {}",
                        dir.display(),
                        module,
                        public_api.display()
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "public API minimalism policy violations:\n{}",
        offenders.join("\n")
    );
}
