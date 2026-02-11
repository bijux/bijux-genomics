#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
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

#[test]
fn policy__contracts__boundary_docs_policy__every_crate_has_boundary_and_public_api_docs() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for dir in crate_dirs(&root) {
        let boundary = dir.join("BOUNDARY.md");
        let public_api = dir.join("PUBLIC_API.md");
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
fn policy__contracts__boundary_docs_policy__public_modules_must_be_listed_in_public_api_doc() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for dir in crate_dirs(&root) {
        let lib = dir.join("src/lib.rs");
        if !lib.exists() {
            continue;
        }
        let public_api = dir.join("PUBLIC_API.md");
        let public_api_raw = std::fs::read_to_string(&public_api)
            .unwrap_or_else(|_| panic!("read {}", public_api.display()));
        let lib_raw =
            std::fs::read_to_string(&lib).unwrap_or_else(|_| panic!("read {}", lib.display()));
        for line in lib_raw.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("pub mod ") {
                let module = rest
                    .split([';', ' '])
                    .next()
                    .unwrap_or_default()
                    .trim()
                    .to_string();
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
