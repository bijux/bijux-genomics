use bijux_dna_bench_model::{
    benchmark_essential_pipeline_readiness_schema, validate_benchmark_essential_pipeline_readiness,
    BenchmarkEssentialPipelineReadinessGoalCheck, BenchmarkEssentialPipelineReadinessReport,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH,
    BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1,
};

fn report_with_failed_goals(failed_goals: &[u32]) -> BenchmarkEssentialPipelineReadinessReport {
    let checks = BENCHMARK_ESSENTIAL_PIPELINE_READINESS_GOALS
        .iter()
        .map(|goal| {
            let pipeline_goal = goal.goal_id <= 270;
            BenchmarkEssentialPipelineReadinessGoalCheck {
                goal_id: goal.goal_id,
                surface: if pipeline_goal {
                    format!("pipeline {}", goal.surface)
                } else {
                    goal.surface.to_string()
                },
                output_path: Some(goal.output_path.to_string()),
                ok: !failed_goals.contains(&goal.goal_id),
                detail: if failed_goals.contains(&goal.goal_id) {
                    "preserved readiness failure".to_string()
                } else {
                    "validated governed pipeline surface".to_string()
                },
            }
        })
        .collect::<Vec<_>>();
    let report_map_failed = failed_goals.contains(&276);
    BenchmarkEssentialPipelineReadinessReport {
        schema_version: BENCHMARK_ESSENTIAL_PIPELINE_READINESS_SCHEMA_V1.to_string(),
        output_path: BENCHMARK_ESSENTIAL_PIPELINE_READINESS_OUTPUT_PATH.to_string(),
        checked_goal_count: checks.len(),
        passed_goal_count: checks.len() - failed_goals.len(),
        failed_goal_count: failed_goals.len(),
        failing_goal_ids: failed_goals.to_vec(),
        pipeline_count: 10,
        dag_node_count: 93,
        corpus_asset_row_count: 93,
        rendered_command_row_count: 93,
        fake_run_node_count: 93,
        fake_run_output_count: 267,
        report_map_row_count: if report_map_failed { 0 } else { 267 },
        ok: failed_goals.is_empty(),
        checks,
    }
}

#[test]
fn essential_pipeline_readiness_accepts_complete_pass_and_failure_records() -> anyhow::Result<()> {
    validate_benchmark_essential_pipeline_readiness(&report_with_failed_goals(&[]))?;
    let failed = report_with_failed_goals(&[276]);
    validate_benchmark_essential_pipeline_readiness(&failed)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkEssentialPipelineReadinessReport>(&serde_json::to_vec(
            &failed
        )?)?,
        failed
    );
    Ok(())
}

#[test]
fn essential_pipeline_readiness_rejects_identity_and_aggregate_drift() {
    let mut report = report_with_failed_goals(&[276]);
    report.failing_goal_ids.clear();
    assert!(validate_benchmark_essential_pipeline_readiness(&report).is_err());

    let mut report = report_with_failed_goals(&[]);
    report.checks[0].surface = "pipeline renamed".to_string();
    assert!(validate_benchmark_essential_pipeline_readiness(&report).is_err());

    let mut report = report_with_failed_goals(&[]);
    report.report_map_row_count = 0;
    assert!(validate_benchmark_essential_pipeline_readiness(&report).is_err());
}

#[test]
fn essential_pipeline_readiness_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/essential-pipeline-readiness.schema.json");
    let generated = serde_json::to_value(benchmark_essential_pipeline_readiness_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "essential-pipeline readiness schema drifted");
    Ok(())
}

#[test]
fn essential_pipeline_readiness_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(report_with_failed_goals(&[]))?;
    value["checks"][0]["legacy_goal"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkEssentialPipelineReadinessReport>(value).is_err());
    Ok(())
}
