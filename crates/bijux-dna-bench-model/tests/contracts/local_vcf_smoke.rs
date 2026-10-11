use bijux_dna_bench_model::{
    benchmark_local_vcf_smoke_suite_schema, validate_benchmark_local_vcf_smoke_suite,
    BenchmarkLocalVcfSmokeGoalCheck, BenchmarkLocalVcfSmokeSuiteReport,
    BENCHMARK_LOCAL_VCF_SMOKE_GOALS, BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH,
    BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1,
};

fn report_with_failed_goals(failed_goals: &[u32]) -> BenchmarkLocalVcfSmokeSuiteReport {
    let checks = BENCHMARK_LOCAL_VCF_SMOKE_GOALS
        .iter()
        .map(|goal| BenchmarkLocalVcfSmokeGoalCheck {
            goal_id: goal.goal_id,
            surface: goal.surface.to_string(),
            output_path: Some(goal.output_path.to_string()),
            ok: !failed_goals.contains(&goal.goal_id),
            detail: if failed_goals.contains(&goal.goal_id) {
                "preserved execution failure".to_string()
            } else {
                "validated governed smoke output".to_string()
            },
        })
        .collect::<Vec<_>>();
    BenchmarkLocalVcfSmokeSuiteReport {
        schema_version: BENCHMARK_LOCAL_VCF_SMOKE_SUITE_SCHEMA_V1.to_string(),
        output_path: BENCHMARK_LOCAL_VCF_SMOKE_OUTPUT_PATH.to_string(),
        checked_goal_count: checks.len(),
        passed_goal_count: checks.len() - failed_goals.len(),
        failed_goal_count: failed_goals.len(),
        failing_goal_ids: failed_goals.to_vec(),
        ok: failed_goals.is_empty(),
        checks,
    }
}

#[test]
fn local_vcf_smoke_suite_accepts_complete_pass_and_failure_records() -> anyhow::Result<()> {
    validate_benchmark_local_vcf_smoke_suite(&report_with_failed_goals(&[]))?;
    let failed = report_with_failed_goals(&[211, 220, 223]);
    validate_benchmark_local_vcf_smoke_suite(&failed)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalVcfSmokeSuiteReport>(&serde_json::to_vec(&failed)?)?,
        failed
    );
    Ok(())
}

#[test]
fn local_vcf_smoke_suite_rejects_goal_and_aggregate_drift() {
    let mut report = report_with_failed_goals(&[220]);
    report.checks.remove(0);
    assert!(validate_benchmark_local_vcf_smoke_suite(&report).is_err());

    let mut report = report_with_failed_goals(&[220]);
    report.failing_goal_ids.clear();
    assert!(validate_benchmark_local_vcf_smoke_suite(&report).is_err());

    let mut report = report_with_failed_goals(&[]);
    report.checks[0].output_path = None;
    assert!(validate_benchmark_local_vcf_smoke_suite(&report).is_err());

    let mut report = report_with_failed_goals(&[]);
    report.checks[0].surface = "vcf.call renamed smoke".to_string();
    assert!(validate_benchmark_local_vcf_smoke_suite(&report).is_err());

    let mut report = report_with_failed_goals(&[]);
    report.output_path = "runs/bench/local-smoke/renamed.json".to_string();
    assert!(validate_benchmark_local_vcf_smoke_suite(&report).is_err());
}

#[test]
fn local_vcf_smoke_suite_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-vcf-smoke-suite-ready.schema.json");
    let generated = serde_json::to_value(benchmark_local_vcf_smoke_suite_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local VCF smoke-suite schema drifted from model authority");
    Ok(())
}

#[test]
fn local_vcf_smoke_suite_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(report_with_failed_goals(&[]))?;
    value["checks"][0]["legacy_goal"] = serde_json::json!(true);
    assert!(serde_json::from_value::<BenchmarkLocalVcfSmokeSuiteReport>(value).is_err());
    Ok(())
}
