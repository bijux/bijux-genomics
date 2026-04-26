use std::collections::BTreeSet;
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

fn dir_entries(path: &Path) -> BTreeSet<String> {
    fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| {
            let entry =
                entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
            let mut name = entry.file_name().to_string_lossy().into_owned();
            if entry.path().is_dir() {
                name.push('/');
            }
            name
        })
        .collect()
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
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
    assert!(offenders.is_empty(), "stages-fastq must not execute tools directly: {offenders:?}");
    Ok(())
}

#[test]
fn stages_fastq_layout_matches_documented_architecture() {
    let root = crate_src_root();
    let expected_root = entries([
        "contracts.rs",
        "lib.rs",
        "metrics/",
        "observer/",
        "plugin/",
        "runtime/",
        "stage_specs/",
        "surface.rs",
    ]);
    assert_eq!(dir_entries(&root), expected_root, "src/ tree changed");

    let expected_metrics =
        entries(["envelope_support.rs", "fastqc.rs", "filters.rs", "mod.rs", "stage_metrics/"]);
    assert_eq!(dir_entries(&root.join("metrics")), expected_metrics, "metrics/ tree changed");

    let expected_stage_metrics = entries([
        "analysis.rs",
        "analysis_feature_tables.rs",
        "analysis_screening.rs",
        "mod.rs",
        "reporting.rs",
        "transform.rs",
        "transform_filtering.rs",
        "transform_pairing.rs",
    ]);
    assert_eq!(
        dir_entries(&root.join("metrics/stage_metrics")),
        expected_stage_metrics,
        "metrics/stage_metrics/ tree changed"
    );

    let expected_observer = entries(["artifacts.rs", "commands.rs", "mod.rs"]);
    assert_eq!(dir_entries(&root.join("observer")), expected_observer, "observer/ tree changed");

    let expected_plugin = entries([
        "mod.rs",
        "observation_context.rs",
        "output_contract.rs",
        "plugin_contracts.rs",
        "semantic/",
    ]);
    assert_eq!(dir_entries(&root.join("plugin")), expected_plugin, "plugin/ tree changed");

    let expected_semantic = entries([
        "feature_tables.rs",
        "mod.rs",
        "processing.rs",
        "processing_cleanup.rs",
        "processing_read_preparation.rs",
        "processing_trimming.rs",
        "profiling.rs",
        "quality.rs",
        "quality_qc.rs",
        "quality_read_flow.rs",
        "taxonomy.rs",
        "validation_semantics.rs",
    ]);
    assert_eq!(
        dir_entries(&root.join("plugin/semantic")),
        expected_semantic,
        "plugin/semantic/ tree changed"
    );

    let expected_runtime = entries(["interpretation.rs", "mod.rs"]);
    assert_eq!(dir_entries(&root.join("runtime")), expected_runtime, "runtime/ tree changed");

    let expected_stage_specs = entries(["artifacts.rs", "catalog.rs", "mod.rs"]);
    assert_eq!(
        dir_entries(&root.join("stage_specs")),
        expected_stage_specs,
        "stage_specs/ tree changed"
    );

    let legacy_files = [
        "runtime_interpretation.rs",
        "stage_specs.rs",
        "metrics/sections/envelope_and_stats.rs",
        "metrics/sections/stage_metrics.rs",
        "metrics/stage_metrics.rs",
        "metrics/stage_metrics_transform.rs",
        "metrics/stage_metrics_reporting.rs",
        "metrics/stage_metrics_analysis.rs",
    ];

    for relative_path in legacy_files {
        assert!(
            !root.join(relative_path).exists(),
            "legacy architecture file should be absent: {relative_path}"
        );
    }
}
