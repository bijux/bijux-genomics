#![allow(non_snake_case)]
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__profiles_runtime_policy__profiles_are_runtime_knobs_only() {
    let root = repo_root();
    let configs_dir = root.join("configs");
    let mut offenders = Vec::new();
    for entry in std::fs::read_dir(&configs_dir).expect("read configs") {
        let path = entry.expect("entry").path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("toml") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if !name.starts_with("profile.") {
            continue;
        }
        let content = std::fs::read_to_string(&path).expect("read profile");
        let forbidden = [
            "tool_id",
            "tool_ids",
            "stage_id",
            "stage_ids",
            "tools =",
            "stages =",
            "fastq.",
            "bam.",
            "vcf.",
        ];
        if forbidden.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "profiles must contain runtime knobs only (no tool/stage identity):\n{}",
        offenders.join("\n")
    );
}
