#![allow(non_snake_case)]
#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::fs;
use std::path::PathBuf;

#[test]
fn policy__boundaries__core_purity__core_has_no_runtime_or_system_deps() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf();
    let cargo_toml = root.join("crates").join("bijux-dna-core").join("Cargo.toml");
    let content = fs::read_to_string(&cargo_toml).expect("read bijux-dna-core/Cargo.toml");
    let forbidden =
        ["tracing-subscriber", "rusqlite", "bollard", "docker", "opendal", "tokio-postgres"];
    let offenders: Vec<&str> =
        forbidden.iter().copied().filter(|needle| content.contains(needle)).collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-core must remain pure; forbidden deps found: {:?}",
        offenders
    );
}

#[test]
fn policy__boundaries__core_purity__core_has_no_stage_modules() {
    let root =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().parent().unwrap().to_path_buf();
    let src_root = root.join("crates").join("bijux-dna-core").join("src");
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(&src_root).into_iter().filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy();
        if name.starts_with("stage_") {
            offenders.push(entry.path().display().to_string());
        }
    }
    for entry in walkdir::WalkDir::new(&src_root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(entry.path()) else {
            continue;
        };
        if content
            .lines()
            .any(|line| line.contains("mod stage_") || line.contains("pub mod stage_"))
        {
            offenders.push(entry.path().display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-core must not define stage_* modules:\n{}",
        offenders.join("\n")
    );
}
