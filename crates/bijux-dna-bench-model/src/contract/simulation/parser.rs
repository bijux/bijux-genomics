//! Owner: pure admission and projection of illustrative simulation parser evidence.
use crate::{
    validate_all_domain_simulation_result, validate_benchmark_stage_result_manifest,
    AllDomainFakeRunMetrics, AllDomainFakeRunResultReport, AllDomainParserCollectorRow,
    AllDomainParserCollectorSourceKind, BenchmarkStageResultManifest, BenchmarkStageResultStatus,
    ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION,
};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Bind parsed simulation evidence to the declared result before projecting a parser row.
///
/// # Errors
/// Rejects unsupported schemas, identity/count drift and incompatible stage observations.
pub fn project_all_domain_simulation_parser_row(
    result: &AllDomainFakeRunResultReport,
    metrics: &AllDomainFakeRunMetrics,
    manifest: &BenchmarkStageResultManifest,
) -> Result<AllDomainParserCollectorRow> {
    validate_all_domain_simulation_result(result)?;
    admit_metrics(result, metrics)?;
    admit_manifest(result, metrics, manifest)?;
    let value = serde_json::to_value(metrics)?;
    let mut keys = value
        .as_object()
        .context("simulation metrics must serialize as an object")?
        .keys()
        .cloned()
        .collect::<Vec<_>>();
    keys.sort();
    Ok(AllDomainParserCollectorRow {
        record_id: format!("fake-run:{}", result.result_id),
        source_kind: AllDomainParserCollectorSourceKind::FakeRun,
        document_kind: "all_domain_fake_run_metrics".to_owned(),
        domain: result.domain.clone(),
        stage_id: result.stage_id.clone(),
        tool_id: result.tool_id.clone(),
        corpus_id: result.corpus_id.clone(),
        asset_profile_id: result.asset_profile_id.clone(),
        result_id: Some(result.result_id.clone()),
        parsed_path: result.metrics_path.clone(),
        parsed_schema_version: metrics.schema_version.clone(),
        parsed_top_level_key_count: keys.len(),
        parsed_top_level_keys: keys,
        manifest_path: Some(result.stage_result_path.clone()),
        manifest_status: Some("succeeded".to_owned()),
        manifest_exit_code: Some(manifest.runtime.exit_code),
        declared_output_count: result.declared_output_count,
        normalized_snapshot: BTreeMap::from([
            ("command_step_count".to_owned(), Value::from(metrics.command_step_count)),
            ("declared_output_count".to_owned(), Value::from(metrics.declared_output_count)),
            ("expected_metric_count".to_owned(), Value::from(metrics.expected_metric_count)),
            (
                "simulated_elapsed_seconds".to_owned(),
                Value::from(metrics.simulated_elapsed_seconds),
            ),
        ]),
    })
}

fn admit_metrics(
    result: &AllDomainFakeRunResultReport,
    metrics: &AllDomainFakeRunMetrics,
) -> Result<()> {
    ensure!(
        metrics.schema_version == ALL_DOMAIN_FAKE_RUN_METRICS_SCHEMA_VERSION,
        "unsupported simulation metrics schema"
    );
    for (name, expected, observed) in [
        ("result", &result.result_id, &metrics.result_id),
        ("domain", &result.domain, &metrics.domain),
        ("stage", &result.stage_id, &metrics.stage_id),
        ("tool", &result.tool_id, &metrics.tool_id),
        ("corpus", &result.corpus_id, &metrics.corpus_id),
        ("asset profile", &result.asset_profile_id, &metrics.asset_profile_id),
    ] {
        ensure!(
            !expected.is_empty() && expected.trim() == expected && expected == observed,
            "simulation metrics {name} identity mismatch"
        );
    }
    ensure!(
        metrics.command_step_count == result.command_step_count
            && metrics.declared_output_count == result.declared_output_count
            && metrics.expected_metric_count == result.expected_metric_count
            && metrics.expected_metric_count == metrics.expected_metrics.len(),
        "simulation metrics counts disagree with result"
    );
    let mut names = BTreeSet::new();
    for name in &metrics.expected_metrics {
        ensure!(
            !name.is_empty() && name.trim() == name && names.insert(name),
            "simulation metric names must be unique nonempty identities"
        );
    }
    ensure!(
        metrics.simulated_elapsed_seconds.is_finite() && metrics.simulated_elapsed_seconds >= 0.0,
        "simulation duration must be finite and nonnegative"
    );
    Ok(())
}

fn admit_manifest(
    result: &AllDomainFakeRunResultReport,
    metrics: &AllDomainFakeRunMetrics,
    manifest: &BenchmarkStageResultManifest,
) -> Result<()> {
    validate_benchmark_stage_result_manifest(manifest)?;
    ensure!(
        manifest.stage_id == result.stage_id && manifest.tool.id == result.tool_id,
        "simulation manifest stage/tool identity mismatch"
    );
    ensure!(
        manifest.runtime.mode == "benchmark_fake_run"
            && manifest.runtime.status == BenchmarkStageResultStatus::Succeeded
            && manifest.runtime.exit_code == 0
            && manifest.runtime.elapsed_seconds.to_bits()
                == metrics.simulated_elapsed_seconds.to_bits(),
        "simulation manifest runtime disagrees with metrics"
    );
    let expected = result
        .outputs
        .iter()
        .map(|o| (&o.artifact_id, &o.declared_output, &o.fake_run_path, &o.role, o.exists))
        .collect::<BTreeSet<_>>();
    let observed = manifest
        .outputs
        .iter()
        .map(|o| (&o.artifact_id, &o.declared_path, &o.realized_path, &o.role, o.exists))
        .collect::<BTreeSet<_>>();
    ensure!(
        expected.len() == result.outputs.len()
            && observed.len() == manifest.outputs.len()
            && expected == observed
            && manifest.outputs.iter().all(|o| !o.optional),
        "simulation manifest outputs disagree with result"
    );
    Ok(())
}
