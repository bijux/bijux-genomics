use super::{inputs, DESTINATION};
use crate::{fake_run_all_domain_benchmark_results, load_all_domain_simulation_parser_row};
use anyhow::Result;
use bijux_dna_bench_model::{
    project_all_domain_simulation_parser_row, AllDomainFakeRunMetrics,
    AllDomainFakeRunResultReport, BenchmarkStageResultManifest,
};
use std::{fs, path::Path};

fn simulate(root: &Path) -> Result<AllDomainFakeRunResultReport> {
    let (expected, command, outputs) = inputs();
    let report = fake_run_all_domain_benchmark_results(
        root,
        Path::new(DESTINATION),
        &[expected],
        &[command],
        &[outputs],
    )?;
    Ok(report.results[0].clone())
}

#[test]
fn simulation_parser_binds_payload_identity_and_preserves_projection() -> Result<()> {
    let root = tempfile::tempdir()?;
    let result = simulate(root.path())?;
    let path = root.path().join(&result.metrics_path);
    let bytes = fs::read(&path)?;
    let document: serde_json::Value = serde_json::from_slice(&bytes)?;
    let row = load_all_domain_simulation_parser_row(root.path(), &result)?;
    assert_eq!(row.result_id.as_deref(), Some(result.result_id.as_str()));
    assert_eq!(row.manifest_status.as_deref(), Some("succeeded"));
    assert_eq!(row.normalized_snapshot.len(), 4);
    for key in [
        "command_step_count",
        "declared_output_count",
        "expected_metric_count",
        "simulated_elapsed_seconds",
    ] {
        assert_eq!(row.normalized_snapshot[key], document[key]);
    }
    assert_eq!(
        row.parsed_top_level_keys,
        document.as_object().unwrap().keys().cloned().collect::<Vec<_>>()
    );
    for key in [
        "result_id",
        "domain",
        "stage_id",
        "tool_id",
        "corpus_id",
        "asset_profile_id",
        "schema_version",
    ] {
        let mut wrong = document.clone();
        wrong[key] = "foreign".into();
        fs::write(&path, serde_json::to_vec(&wrong)?)?;
        assert!(
            load_all_domain_simulation_parser_row(root.path(), &result).is_err(),
            "accepted foreign {key}"
        );
    }
    for key in ["command_step_count", "declared_output_count", "expected_metric_count"] {
        let mut wrong = document.clone();
        wrong[key] = 99.into();
        fs::write(&path, serde_json::to_vec(&wrong)?)?;
        assert!(
            load_all_domain_simulation_parser_row(root.path(), &result).is_err(),
            "accepted inconsistent {key}"
        );
    }
    let mut wrong = document;
    wrong["unknown"] = true.into();
    fs::write(&path, serde_json::to_vec(&wrong)?)?;
    assert!(load_all_domain_simulation_parser_row(root.path(), &result).is_err());
    fs::write(&path, &bytes)?;
    assert!(load_all_domain_simulation_parser_row(root.path(), &result).is_ok());
    Ok(())
}

#[test]
fn simulation_parser_rejects_contradictory_runtime_outputs_and_metric_names() -> Result<()> {
    let root = tempfile::tempdir()?;
    let result = simulate(root.path())?;
    let metrics: AllDomainFakeRunMetrics =
        serde_json::from_slice(&fs::read(root.path().join(&result.metrics_path))?)?;
    let manifest: BenchmarkStageResultManifest =
        serde_json::from_slice(&fs::read(root.path().join(&result.stage_result_path))?)?;
    for change in 0..7 {
        let mut wrong = manifest.clone();
        match change {
            0 => wrong.stage_id = "foreign".into(),
            1 => wrong.tool.id = "foreign".into(),
            2 => wrong.runtime.mode = "native_execution".into(),
            3 => wrong.runtime.elapsed_seconds += 1.0,
            4 => wrong.outputs[0].realized_path = "foreign".into(),
            5 => wrong.outputs[0].optional = true,
            _ => wrong.outputs.push(wrong.outputs[0].clone()),
        }
        assert!(
            project_all_domain_simulation_parser_row(&result, &metrics, &wrong).is_err(),
            "accepted manifest drift {change}"
        );
    }
    for duration in [f64::NAN, f64::INFINITY, -1.0] {
        let mut wrong = metrics.clone();
        wrong.simulated_elapsed_seconds = duration;
        assert!(project_all_domain_simulation_parser_row(&result, &wrong, &manifest).is_err());
    }
    let mut wrong = metrics.clone();
    wrong.expected_metrics[0] = " ".into();
    assert!(project_all_domain_simulation_parser_row(&result, &wrong, &manifest).is_err());
    let mut duplicate = metrics;
    duplicate.expected_metrics.push(duplicate.expected_metrics[0].clone());
    duplicate.expected_metric_count += 1;
    let mut duplicate_result = result;
    duplicate_result.expected_metric_count += 1;
    assert!(
        project_all_domain_simulation_parser_row(&duplicate_result, &duplicate, &manifest).is_err()
    );
    Ok(())
}

#[test]
fn simulation_parser_rejects_missing_truncated_and_oversized_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let result = simulate(root.path())?;
    for relative in [&result.metrics_path, &result.stage_result_path] {
        let path = root.path().join(relative);
        let bytes = fs::read(&path)?;
        fs::write(&path, b"{\"truncated\":")?;
        assert!(load_all_domain_simulation_parser_row(root.path(), &result).is_err());
        fs::File::create(&path)?.set_len(16 * 1024 * 1024 + 1)?;
        assert!(load_all_domain_simulation_parser_row(root.path(), &result).is_err());
        fs::remove_file(&path)?;
        assert!(load_all_domain_simulation_parser_row(root.path(), &result).is_err());
        fs::write(&path, bytes)?;
    }
    assert!(load_all_domain_simulation_parser_row(root.path(), &result).is_ok());
    Ok(())
}
