#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

const MAX_TEST_DATA_BYTES: u64 = 128 * 1024;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

#[test]
fn policy__contracts__test_data_policies__large_binary_test_data_is_forbidden() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let targets = [root.join("tests"), root.join("crates")];

    for target in targets {
        for entry in WalkDir::new(target).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let path_str = path.to_string_lossy();
            if path_str.contains("/target/") || path_str.contains("/.git/") {
                continue;
            }
            let is_root_tests = path_str.contains("/tests/");
            let is_crate_fixture = path_str.contains("/tests/fixtures/");
            if !is_root_tests && !is_crate_fixture {
                continue;
            }
            let metadata = match std::fs::metadata(path) {
                Ok(metadata) => metadata,
                Err(_) => continue,
            };
            if metadata.len() > MAX_TEST_DATA_BYTES {
                offenders.push(format!("{} ({} bytes)", path.display(), metadata.len()));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "large binary test data is forbidden (>{} bytes):\n{}",
        MAX_TEST_DATA_BYTES,
        offenders.join("\n")
    );
}
