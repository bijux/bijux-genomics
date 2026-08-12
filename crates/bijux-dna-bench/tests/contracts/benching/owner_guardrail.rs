use std::fs;

#[test]
fn source_modules_have_owner() {
    let src_dir = crate::support::crate_root("bijux-dna-bench")
        .unwrap_or_else(|err| panic!("resolve benchmark crate root: {err}"))
        .join("src");
    let mut modules = Vec::new();
    collect_rs_files(&src_dir, &mut modules);

    let mut offenders = Vec::new();
    for module in modules {
        let Ok(contents) = fs::read_to_string(&module) else {
            continue;
        };
        let mut has_owner = false;
        for line in contents.lines().take(8) {
            if line.trim().starts_with("//!") && line.contains("Owner:") {
                has_owner = true;
                break;
            }
            if !line.trim().is_empty() && !line.trim().starts_with("//!") {
                break;
            }
        }
        if !has_owner {
            offenders.push(module.display().to_string());
        }
    }

    assert!(offenders.is_empty(), "missing module owner doc comments: {offenders:?}");
}

#[test]
fn thin_cli_delegates_stage_result_admission_to_benchmark_owners() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_result_manifest.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(
        contents.contains("validate_benchmark_stage_result_path"),
        "thin CLI must delegate file-backed admission to bijux-dna-bench"
    );
    assert!(
        contents.contains("validate_benchmark_stage_result_manifest"),
        "thin CLI must delegate manifest validation to bijux-dna-bench-model through the API"
    );
    assert!(
        !contents.contains("struct BenchStageResultValidationReport"),
        "thin CLI must not own the persisted validation report"
    );
    assert!(
        !contents.contains("bijux.bench.stage_result_validation.v1"),
        "thin CLI must not own the validation schema identity"
    );
}

#[test]
fn thin_cli_uses_model_owned_local_runtime_metrics_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_stage_runtime_metrics.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_runtime_metrics"));
    assert!(!contents.contains("struct BenchLocalStageRuntimeMetricEntry"));
    assert!(!contents.contains("struct BenchLocalStageRuntimeMetricsReport"));
    assert!(!contents.contains("bijux.bench.local_runtime_metrics.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_benchmark_summary_contract() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    let cli_source =
        repo_root.join("crates/bijux-dna/src/commands/benchmark/local_benchmark_summary.rs");
    let contents = fs::read_to_string(&cli_source)
        .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

    assert!(contents.contains("validate_benchmark_local_summary"));
    assert!(!contents.contains("struct BenchLocalBenchmarkSummaryStage"));
    assert!(!contents.contains("struct BenchLocalBenchmarkSummaryReport"));
    assert!(!contents.contains("bijux.bench.local_benchmark_summary.v1"));
}

#[test]
fn thin_cli_uses_model_owned_local_stage_completion_contracts() {
    let repo_root = crate::support::repo_root().expect("resolve repository root");
    for (file_name, validator, forbidden) in [
        (
            "local_stage_manifest_completion.rs",
            "validate_benchmark_local_stage_manifest_completion",
            "struct BenchLocalStageManifestCompletionReport",
        ),
        (
            "local_stage_output_completion.rs",
            "validate_benchmark_local_stage_output_completion",
            "struct BenchLocalStageOutputCompletionReport",
        ),
    ] {
        let cli_source = repo_root.join("crates/bijux-dna/src/commands/benchmark").join(file_name);
        let contents = fs::read_to_string(&cli_source)
            .unwrap_or_else(|err| panic!("read {}: {err}", cli_source.display()));

        assert!(contents.contains(validator));
        assert!(!contents.contains(forbidden));
        assert!(!contents.contains("bijux.bench.local_stage_manifest_completion.v1"));
        assert!(!contents.contains("bijux.bench.local_stage_output_completion.v1"));
    }
}

fn collect_rs_files(root: &std::path::Path, files: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files(&path, files);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            files.push(path);
        }
    }
}
