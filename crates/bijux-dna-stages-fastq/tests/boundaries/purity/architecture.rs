use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(|entry| entry.path().to_path_buf())
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .collect()
}

fn crate_src_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

#[test]
fn stages_fastq_has_no_execution_calls() -> Result<(), Box<dyn std::error::Error>> {
    let root = crate_src_root();
    let files = collect_rs_files(&root);
    let forbidden = [
        concat!("std::process::", "Command"),
        "process::Command",
        concat!("Command::", "new"),
        concat!("Docker", "Runner"),
        concat!("docker", "::"),
        "docker_runner",
        "executor",
        "run_tool_container",
        "run_validate_container",
        "run_merge_container",
        "RuntimeKind",
        "bijux_dna_engine::",
        "bijux_dna_environment::",
    ];
    let mut offenders = Vec::new();
    for path in files {
        let contents = fs::read_to_string(&path)?;
        for needle in &forbidden {
            if contents.contains(needle) {
                offenders.push(format!("{} -> {}", path.display(), needle));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "stages-fastq must not execute tools directly: {offenders:?}"
    );
    Ok(())
}

#[test]
fn stages_fastq_layout_matches_documented_architecture() {
    let root = crate_src_root();
    let expected_files = [
        "lib.rs",
        "surface.rs",
        "runtime/mod.rs",
        "runtime/interpretation.rs",
        "stage_specs/mod.rs",
        "stage_specs/catalog.rs",
        "stage_specs/artifacts.rs",
        "observer/mod.rs",
        "observer/artifacts.rs",
        "observer/commands.rs",
        "metrics/mod.rs",
        "metrics/envelope_support.rs",
        "metrics/stage_metrics.rs",
        "metrics/stage_metrics_transform.rs",
        "metrics/stage_metrics_reporting.rs",
        "metrics/stage_metrics_analysis.rs",
    ];
    let legacy_files = [
        "runtime_interpretation.rs",
        "stage_specs.rs",
        "metrics/sections/envelope_and_stats.rs",
        "metrics/sections/stage_metrics.rs",
    ];

    for relative_path in expected_files {
        assert!(
            root.join(relative_path).is_file(),
            "expected architecture file is missing: {relative_path}"
        );
    }

    for relative_path in legacy_files {
        assert!(
            !root.join(relative_path).exists(),
            "legacy architecture file should be absent: {relative_path}"
        );
    }
}
