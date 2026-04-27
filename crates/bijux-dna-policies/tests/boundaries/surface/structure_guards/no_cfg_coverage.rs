#![allow(non_snake_case)]

use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__boundaries__no_cfg_coverage__production_src_forbids_cfg_coverage() {
    let root = workspace_root();
    let crates_dir = root.join("crates");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&crates_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if !path.to_string_lossy().contains("/src/") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/")
            || path.to_string_lossy().contains("/benches/")
            || path.to_string_lossy().contains("/dev-tools/")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if content.contains("cfg(coverage)") || content.contains("#[cfg(coverage)]") {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "cfg(coverage) is not allowed in production src/ (tests/benches/dev-tools only):\n{}",
        offenders.join("\n")
    );
}
