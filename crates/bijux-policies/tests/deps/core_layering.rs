use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

#[test]
fn core_layering_is_enforced() {
    let root = workspace_root();
    let primitives_dir = root.join("crates/bijux-core/src/foundation");
    let contract_dir = root.join("crates/bijux-core/src/contract");
    let mut offenders = Vec::new();

    let forbidden_in_primitives = ["crate::contract", "crate::metrics::registry"];
    for file in collect_rs_files(&primitives_dir) {
        let content = std::fs::read_to_string(&file).expect("read source");
        for needle in &forbidden_in_primitives {
            if content.contains(needle) {
                offenders.push(format!("{} imports forbidden {}", file.display(), needle));
            }
        }
    }

    let forbidden_in_contract = ["crate::foundation::invariants"];
    for file in collect_rs_files(&contract_dir) {
        let content = std::fs::read_to_string(&file).expect("read source");
        for needle in &forbidden_in_contract {
            if content.contains(needle) {
                offenders.push(format!("{} imports forbidden {}", file.display(), needle));
            }
        }
    }

    assert!(
        offenders.is_empty(),
        "core layering violations:\n{}",
        offenders.join("\n")
    );
}
