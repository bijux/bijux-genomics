use bijux_dna_analyze::metric_semantics;

#[test]
fn metrics_used_for_ranking_have_semantics() {
    let metric_ids = [
        "runtime_s",
        "memory_mb",
        "read_retention",
        "base_retention",
        "merge_rate",
        "error_reduction_proxy",
    ];
    for metric_id in metric_ids {
        assert!(metric_semantics(metric_id).is_some(), "missing semantics for {metric_id}");
    }
}

fn semantic_count_record<T: bijux_dna_analyze::StageMetricSchema + serde::Serialize>(
    metrics: T,
) -> bijux_dna_analyze::BenchmarkRecord<T> {
    bijux_dna_analyze::BenchmarkRecord {
        context: bijux_dna_analyze::BenchmarkContext {
            tool: "count-unit-fixture".to_string(),
            tool_version: "1".to_string(),
            image_digest: "sha256:fixture".to_string(),
            runner: "local".to_string(),
            platform: "fixture".to_string(),
            input_hash: "sha256:input".to_string(),
            parameters: bijux_dna_analyze::model::JsonBlob::new(serde_json::json!({})),
        },
        execution: bijux_dna_core::prelude::measure::ExecutionMetrics {
            runtime_s: 1.0,
            memory_mb: 1.0,
            exit_code: 0,
        },
        metrics: bijux_dna_analyze::metric_set(metrics),
    }
}

#[test]
fn semantic_reports_keep_base_counts_distinct_from_read_counts() -> anyhow::Result<()> {
    use bijux_dna_analyze::{FastqFilterMetrics, FastqValidateMetrics, StageMetricSchema};
    let root = bijux_dna_infra::temp_dir("semantic-count-units")?;
    let filter: FastqFilterMetrics = serde_json::from_value(serde_json::json!({
        "reads_in": 100, "reads_out": 80, "reads_dropped": 20,
        "reads_removed_by_n": 20, "bases_in": 1000, "bases_out": 720,
        "mean_q_before": 30.0, "mean_q_after": 30.0,
        "delta_metrics": {"read_retention": 0.8, "base_retention": 0.72,
            "mean_q_delta": 0.0, "gc_delta": 0.0}
    }))?;
    let validate: FastqValidateMetrics = serde_json::from_value(serde_json::json!({
        "reads_in": 100, "reads_out": 100, "bases_in": 1000, "bases_out": 1000,
        "reads_total": 100, "reads_valid": 95, "reads_invalid": 5, "mean_q": 32.0
    }))?;
    filter.validate()?;
    validate.validate()?;
    let filter_dir = root.path().join("filter");
    let validate_dir = root.path().join("validate");
    bijux_dna_infra::ensure_dir(&filter_dir)?;
    bijux_dna_infra::ensure_dir(&validate_dir)?;
    bijux_dna_analyze::write_filter_report(
        &filter_dir,
        &[semantic_count_record(filter)],
        &[],
        false,
    )?;
    bijux_dna_analyze::write_validate_report(
        &validate_dir,
        &[semantic_count_record(validate)],
        &[],
        None,
        false,
    )?;
    let mut observations = Vec::new();
    for (directory, reads_out, bases_out) in [(&filter_dir, 80, 720), (&validate_dir, 95, 1000)] {
        let report: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("report.json"))?)?;
        let integrity = &report["semantic_metrics"][0]["integrity"];
        assert_eq!(integrity["reads_in"]["value"]["u64"], 100);
        assert_eq!(integrity["reads_out"]["value"]["u64"], reads_out);
        observations.push((
            integrity["bases_in"]["value"]["u64"] == 1000,
            integrity["bases_out"]["value"]["u64"] == bases_out,
        ));
        eprintln!("semantic_count_units {}: {integrity}", directory.display());
    }
    assert!(
        observations.iter().all(|&(input, output)| input && output),
        "semantic base counts must project the admitted typed base fields"
    );
    Ok(())
}
