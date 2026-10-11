//! Semantic admission for persisted local essential-pipeline execution simulations.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalEssentialPipelineFakeRunNodeMetrics,
    BenchmarkLocalEssentialPipelineFakeRunNodeReport,
    BenchmarkLocalEssentialPipelineFakeRunPipelineReport,
    BenchmarkLocalEssentialPipelineFakeRunsReport,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1,
};

/// Validate one persisted node metrics record.
///
/// # Errors
/// Returns an error when identities, binding lineage, counters, or timing semantics drift.
pub fn validate_benchmark_local_essential_pipeline_fake_run_node_metrics(
    metrics: &BenchmarkLocalEssentialPipelineFakeRunNodeMetrics,
) -> Result<(), BenchError> {
    if metrics.schema_version != BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1
    {
        return invalid("unsupported local essential-pipeline node metrics schema");
    }
    for (value, field) in [
        (&metrics.pipeline_id, "pipeline_id"),
        (&metrics.node_id, "node_id"),
        (&metrics.stage_id, "stage_id"),
        (&metrics.tool_id, "tool_id"),
        (&metrics.corpus_id, "corpus_id"),
        (&metrics.asset_profile_id, "asset_profile_id"),
    ] {
        require_text(value, field)?;
    }
    if metrics.command_step_count == 0
        || metrics.output_count == 0
        || !metrics.simulated_elapsed_seconds.is_finite()
        || metrics.simulated_elapsed_seconds <= 0.0
    {
        return invalid("local essential-pipeline node metrics require work, outputs, and time");
    }
    if metrics.external_input_count != metrics.external_bindings.len()
        || metrics.upstream_input_count != metrics.upstream_bindings.len()
    {
        return invalid("local essential-pipeline node metrics binding counts drifted");
    }

    let mut input_ids = BTreeSet::new();
    let mut producer_ids = BTreeSet::new();
    for binding in &metrics.external_bindings {
        require_text(&binding.input_id, "external_bindings[].input_id")?;
        if binding.source != "external"
            || binding.producer_node_id.is_some()
            || binding.fake_run_path.is_some()
        {
            return invalid("external input bindings cannot claim an upstream producer or path");
        }
        if !input_ids.insert(binding.input_id.as_str()) {
            return invalid("local essential-pipeline node metrics repeat an input identity");
        }
    }
    for binding in &metrics.upstream_bindings {
        require_text(&binding.input_id, "upstream_bindings[].input_id")?;
        if binding.source != "upstream" {
            return invalid("upstream input bindings require the upstream source identity");
        }
        let producer = binding.producer_node_id.as_deref().ok_or_else(|| {
            BenchError::InvalidPolicy(
                "upstream input bindings require a producer node identity".to_string(),
            )
        })?;
        require_text(producer, "upstream_bindings[].producer_node_id")?;
        producer_ids.insert(producer);
        let path = binding.fake_run_path.as_deref().ok_or_else(|| {
            BenchError::InvalidPolicy(
                "upstream input bindings require a realized fake-run path".to_string(),
            )
        })?;
        require_path(path, "upstream_bindings[].fake_run_path")?;
        if !input_ids.insert(binding.input_id.as_str()) {
            return invalid("local essential-pipeline node metrics repeat an input identity");
        }
    }
    if metrics.dependency_count != producer_ids.len() {
        return invalid("local essential-pipeline dependency count drifted from producer lineage");
    }
    Ok(())
}

/// Validate one persisted essential-pipeline node report below a pipeline root.
///
/// # Errors
/// Returns an error when identity, materialization, counters, or path closure drift.
pub fn validate_benchmark_local_essential_pipeline_fake_run_node_report(
    pipeline_root: &Path,
    report: &BenchmarkLocalEssentialPipelineFakeRunNodeReport,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&report.pipeline_id, "pipeline_id"),
        (&report.node_id, "node_id"),
        (&report.stage_id, "stage_id"),
        (&report.tool_id, "tool_id"),
        (&report.domain, "domain"),
        (&report.readiness_kind, "readiness_kind"),
        (&report.command_source, "command_source"),
    ] {
        require_text(value, field)?;
    }
    if report.stage_id.split_once('.').map(|(domain, _)| domain) != Some(report.domain.as_str()) {
        return invalid("local essential-pipeline node stage and domain identities disagree");
    }
    if report.command_step_count == 0
        || report.outputs.is_empty()
        || report.declared_output_count != report.outputs.len()
        || report.created_output_count
            != report.outputs.iter().filter(|output| output.exists).count()
        || report.created_output_count != report.declared_output_count
    {
        return invalid("local essential-pipeline node output or command counts are inconsistent");
    }

    require_path(&report.stage_result_path, "stage_result_path")?;
    let Some(node_root) = Path::new(&report.stage_result_path).parent() else {
        return invalid("local essential-pipeline stage_result_path has no parent");
    };
    if node_root.as_os_str().is_empty() || node_root != pipeline_root.join(&report.node_id) {
        return invalid("local essential-pipeline node root escapes its pipeline root");
    }
    for (path, field) in [
        (&report.command_script_path, "command_script_path"),
        (&report.stdout_path, "stdout_path"),
        (&report.stderr_path, "stderr_path"),
        (&report.metrics_path, "metrics_path"),
        (&report.stage_result_path, "stage_result_path"),
    ] {
        require_descendant_path(node_root, path, field)?;
    }
    for (path, file_name) in [
        (&report.command_script_path, "command.sh"),
        (&report.stdout_path, "stdout.txt"),
        (&report.stderr_path, "stderr.txt"),
        (&report.metrics_path, "metrics.json"),
        (&report.stage_result_path, "stage-result.json"),
    ] {
        if Path::new(path) != node_root.join(file_name) {
            return invalid("local essential-pipeline node artifact path drifted");
        }
    }

    let mut output_ids = BTreeSet::new();
    for output in &report.outputs {
        require_text(&output.artifact_id, "outputs[].artifact_id")?;
        require_text(&output.declared_output, "outputs[].declared_output")?;
        require_text(&output.role, "outputs[].role")?;
        require_descendant_path(node_root, &output.fake_run_path, "outputs[].fake_run_path")?;
        if !output.exists {
            return invalid("successful local essential-pipeline output must exist");
        }
        if !output_ids.insert((output.artifact_id.as_str(), output.declared_output.as_str())) {
            return invalid("local essential-pipeline node repeats an output identity");
        }
    }
    Ok(())
}

/// Validate one persisted essential-pipeline report.
///
/// # Errors
/// Returns an error when schema, topology, node closure, counters, or paths drift.
pub fn validate_benchmark_local_essential_pipeline_fake_run_pipeline_report(
    root: &Path,
    report: &BenchmarkLocalEssentialPipelineFakeRunPipelineReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1 {
        return invalid("unsupported local essential-pipeline report schema");
    }
    require_text(&report.pipeline_id, "pipeline_id")?;
    require_path(&report.pipeline_manifest_path, "pipeline_manifest_path")?;
    let Some(pipeline_root) = Path::new(&report.pipeline_manifest_path).parent() else {
        return invalid("local essential-pipeline manifest path has no parent");
    };
    if pipeline_root.as_os_str().is_empty() || pipeline_root != root.join(&report.pipeline_id) {
        return invalid("local essential-pipeline manifest escapes the fake-run root");
    }
    if Path::new(&report.pipeline_manifest_path) != pipeline_root.join("manifest.json") {
        return invalid("local essential-pipeline manifest path drifted");
    }
    if report.nodes.is_empty()
        || report.node_count != report.nodes.len()
        || report.topological_order.len() != report.nodes.len()
        || report.created_output_count
            != report.nodes.iter().map(|node| node.created_output_count).sum::<usize>()
    {
        return invalid("local essential-pipeline topology or output counters are inconsistent");
    }
    let ordered_ids = report.topological_order.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let node_ids = report.nodes.iter().map(|node| node.node_id.as_str()).collect::<BTreeSet<_>>();
    if ordered_ids.len() != report.topological_order.len()
        || node_ids.len() != report.nodes.len()
        || ordered_ids != node_ids
    {
        return invalid("local essential-pipeline topological order must close the node inventory");
    }
    for node in &report.nodes {
        if node.pipeline_id != report.pipeline_id {
            return invalid("local essential-pipeline node has a mismatched pipeline identity");
        }
        validate_benchmark_local_essential_pipeline_fake_run_node_report(pipeline_root, node)?;
    }
    Ok(())
}

/// Validate the persisted essential-pipeline portfolio report.
///
/// # Errors
/// Returns an error when schema, portfolio identity, aggregate counters, or paths drift.
pub fn validate_benchmark_local_essential_pipeline_fake_runs_report(
    report: &BenchmarkLocalEssentialPipelineFakeRunsReport,
    expected_pipeline_ids: &[&str],
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1 {
        return invalid("unsupported local essential-pipeline fake-runs schema");
    }
    require_path(&report.fake_run_root, "fake_run_root")?;
    require_descendant(&report.fake_run_root, &report.root_manifest_path, "root_manifest_path")?;
    if Path::new(&report.root_manifest_path)
        != Path::new(&report.fake_run_root).join("manifest.json")
    {
        return invalid("local essential-pipeline root manifest path drifted");
    }
    if expected_pipeline_ids.is_empty()
        || report.pipelines.is_empty()
        || report.pipeline_count != expected_pipeline_ids.len()
        || report.pipeline_count != report.pipelines.len()
        || report.node_count != report.pipelines.iter().map(|row| row.node_count).sum::<usize>()
        || report.created_output_count
            != report.pipelines.iter().map(|row| row.created_output_count).sum::<usize>()
    {
        return invalid("local essential-pipeline portfolio counters are inconsistent");
    }
    let root = Path::new(&report.fake_run_root);
    let mut pipeline_ids = BTreeSet::new();
    for pipeline in &report.pipelines {
        if !pipeline_ids.insert(pipeline.pipeline_id.as_str()) {
            return invalid("local essential-pipeline portfolio repeats a pipeline identity");
        }
        validate_benchmark_local_essential_pipeline_fake_run_pipeline_report(root, pipeline)?;
    }
    let expected = expected_pipeline_ids.iter().copied().collect::<BTreeSet<_>>();
    if expected.len() != expected_pipeline_ids.len() || pipeline_ids != expected {
        return invalid(
            "local essential-pipeline portfolio drifted from its governed pipeline set",
        );
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local essential-pipeline evidence requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local essential-pipeline evidence requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn require_descendant(root: &str, value: &str, field: &str) -> Result<(), BenchError> {
    require_descendant_path(Path::new(root), value, field)
}

fn require_descendant_path(root: &Path, value: &str, field: &str) -> Result<(), BenchError> {
    require_path(value, field)?;
    if !Path::new(value).starts_with(root) {
        return invalid(format!(
            "local essential-pipeline evidence requires {field} below its governed root"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
