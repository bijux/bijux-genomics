#![allow(non_snake_case)]
#![allow(non_snake_case)]
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
fn policy__contracts__defaults_policy__params_defaults_live_in_pipelines_only() {
    let root = workspace_root();
    let targets = [
        root.join("crates/bijux-api/src"),
        root.join("crates/bijux-cli/src"),
        root.join("crates/bijux-stages-fastq/src"),
        root.join("crates/bijux-stages-bam/src"),
    ];
    let regex_default = regex::Regex::new(r"\b[A-Za-z0-9_]*Params::default\b").unwrap();
    let regex_default_call = regex::Regex::new(r"Default::default\(\)").unwrap();
    let mut offenders = Vec::new();

    for target in targets {
        for file in collect_rs_files(&target) {
            if file.to_string_lossy().contains("/tests/") {
                continue;
            }
            let content = std::fs::read_to_string(&file).expect("read source");
            if regex_default.is_match(&content)
                || (regex_default_call.is_match(&content) && content.contains("Params"))
            {
                offenders.push(file.display().to_string());
            }
        }
    }

    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "param defaults must be defined in bijux-pipelines only:\n{}",
        offenders.join("\n")
    );
}
