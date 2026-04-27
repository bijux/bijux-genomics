#![allow(non_snake_case)]
use std::path::PathBuf;

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__contracts__bench_layout_policy__root_bench_directory_is_forbidden() {
    let root = workspace_root();
    let bench_root = root.join("bench");
    bijux_dna_policies::policy_assert!(
        !bench_root.exists(),
        "root bench/ directory is forbidden; suites must live under crates/bijux-dna-bench/bench/"
    );
}

#[test]
fn slow__policy__contracts__bench_layout_policy__bench_suites_live_only_under_bench_crate() {
    let root = workspace_root();
    let canonical = root.join("crates/bijux-dna-bench/bench/suites");
    let mut offenders = Vec::new();
    for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|v| v.to_str()) != Some("toml") {
            continue;
        }
        let raw = std::fs::read_to_string(path).unwrap_or_default();
        if !raw.contains("bijux.bench-suite.fastq.v1") {
            continue;
        }
        if !path.starts_with(&canonical) {
            offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bench suite files must live only under crates/bijux-dna-bench/bench/suites:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__bench_layout_policy__cli_and_bench_use_shared_bench_path_helper() {
    let root = workspace_root();
    let cli_path_candidates = [
        "crates/bijux-dna/src/commands/benchmark/suite/suite_sections/analysis_and_status.rs",
        "crates/bijux-dna/src/commands/bench_suite/suite_sections/analysis_and_status.rs",
    ];
    let cli = cli_path_candidates
        .iter()
        .map(|rel| root.join(rel))
        .find(|path| path.exists())
        .map(|path| std::fs::read_to_string(path).expect("read cli bench suite module"))
        .expect("resolve cli bench suite module path");
    let bench_workspace_paths =
        std::fs::read_to_string(root.join("crates/bijux-dna-bench/src/repo/workspace_paths.rs"))
            .expect("read bench workspace path helper");

    let cli_uses_helper = cli.contains("bijux_dna_infra::bench_suites_dir");
    let bench_uses_helper = bench_workspace_paths.contains("bijux_dna_infra::bench_suites_dir");

    bijux_dna_policies::policy_assert!(
        cli_uses_helper && bench_uses_helper,
        "CLI bench status and bench crate must both use bijux_dna_infra::bench_suites_dir helper"
    );
}

#[test]
fn policy__contracts__bench_layout_policy__legacy_root_bench_paths_not_hardcoded() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for dir in ["crates", "scripts", "makes", "docs"] {
        for entry in WalkDir::new(root.join(dir)).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(|v| v.to_str()).unwrap_or_default();
            if !matches!(ext, "rs" | "sh" | "mk" | "md" | "toml") {
                continue;
            }
            let raw = std::fs::read_to_string(path).unwrap_or_default();
            if raw.contains("bench/suites") && !raw.contains("crates/bijux-dna-bench/bench/suites")
            {
                offenders.push(path.strip_prefix(&root).unwrap_or(path).display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "legacy root bench/suites path must not be hardcoded:\\n{}",
        offenders.join("\n")
    );
}
