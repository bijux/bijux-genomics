use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    active_scope_fast_validation_report_schema, validate_active_scope_fast_validation_report,
    ActiveScopeFastValidationCheck, ActiveScopeFastValidationReport,
    ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1,
};

#[test]
fn active_scope_report_validates_exact_summary_closure() {
    assert!(validate_active_scope_fast_validation_report(&report()).is_ok());

    let mut drifted = report();
    drifted.passed_surface_count = 1;
    assert!(validate_active_scope_fast_validation_report(&drifted)
        .is_err_and(|error| error.to_string().contains("summary drifted")));

    let mut duplicated = report();
    duplicated.checks.push(duplicated.checks[0].clone());
    duplicated.checked_surface_count += 1;
    duplicated.passed_surface_count += 1;
    *duplicated.category_counts.entry("schemas".to_string()).or_default() += 1;
    assert!(validate_active_scope_fast_validation_report(&duplicated)
        .is_err_and(|error| error.to_string().contains("duplicate surface")));

    let mut incomplete = report();
    incomplete.checks[0].detail.clear();
    assert!(validate_active_scope_fast_validation_report(&incomplete)
        .is_err_and(|error| error.to_string().contains("check is incomplete")));

    let mut empty = report();
    empty.checks.clear();
    empty.checked_surface_count = 0;
    empty.passed_surface_count = 0;
    empty.category_counts.clear();
    assert!(validate_active_scope_fast_validation_report(&empty)
        .is_err_and(|error| error.to_string().contains("identity is invalid")));

    let mut wrong_surface = report();
    wrong_surface.checks[0].surface_id = "unexpected_surface".to_string();
    assert!(validate_active_scope_fast_validation_report(&wrong_surface)
        .is_err_and(|error| error.to_string().contains("surface closure drifted")));
}

#[test]
fn active_scope_schema_matches_model_and_version_fails_closed() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/active-scope-fast-validation.schema.json");
    let generated = serde_json::to_value(active_scope_fast_validation_report_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "active-scope schema drifted");

    let mut value = serde_json::to_value(report())?;
    value["schema_version"] = serde_json::json!("bijux.bench.active_scope_validate_fast.v2");
    assert!(serde_json::from_value::<ActiveScopeFastValidationReport>(value).is_err());

    let mut value = serde_json::to_value(report())?;
    value["mode"] = serde_json::json!("full");
    assert!(serde_json::from_value::<ActiveScopeFastValidationReport>(value).is_err());
    Ok(())
}

fn report() -> ActiveScopeFastValidationReport {
    ActiveScopeFastValidationReport {
        schema_version: ACTIVE_SCOPE_FAST_VALIDATION_SCHEMA_V1.to_string(),
        output_path: "artifacts/bench-active-scope/validate-fast.json".to_string(),
        mode: "fast".to_string(),
        benchmark_root: "benchmarks".to_string(),
        schema_root: "benchmarks/schemas".to_string(),
        fixture_root: "benchmarks/fixtures".to_string(),
        checked_surface_count: 10,
        passed_surface_count: 10,
        failed_surface_count: 0,
        category_counts: BTreeMap::from([
            ("adapters".to_string(), 1),
            ("commands".to_string(), 2),
            ("configs".to_string(), 1),
            ("expected_results".to_string(), 1),
            ("fixtures".to_string(), 1),
            ("outputs".to_string(), 1),
            ("parsers".to_string(), 1),
            ("reports".to_string(), 1),
            ("schemas".to_string(), 1),
        ]),
        failed_category_counts: BTreeMap::new(),
        ok: true,
        checks: vec![
            ActiveScopeFastValidationCheck {
                category: "configs".to_string(),
                surface_id: "all_domain_active_stage_tool_matrix".to_string(),
                output_path: "artifacts/bench-readiness/active-stage-tool-matrix.tsv".to_string(),
                ok: true,
                detail: "row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "schemas".to_string(),
                surface_id: "all_domain_schema_validation".to_string(),
                output_path: "artifacts/bench-schemas/validation.json".to_string(),
                ok: true,
                detail: "domain_count=3".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "fixtures".to_string(),
                surface_id: "benchmark_fixture_root_validation".to_string(),
                output_path: "artifacts/bench-fixtures/validation.json".to_string(),
                ok: true,
                detail: "checked_fixture_count=10".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "adapters".to_string(),
                surface_id: "all_domain_adapter_coverage".to_string(),
                output_path: "artifacts/bench-readiness/adapter-coverage.tsv".to_string(),
                ok: true,
                detail: "covered_row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "commands".to_string(),
                surface_id: "all_domain_local_job_coverage".to_string(),
                output_path: "artifacts/bench-readiness/local-job-coverage.tsv".to_string(),
                ok: true,
                detail: "covered_row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "commands".to_string(),
                surface_id: "all_domain_no_placeholder_command_check".to_string(),
                output_path: "artifacts/bench-readiness/no-placeholder-commands.tsv".to_string(),
                ok: true,
                detail: "valid_row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "parsers".to_string(),
                surface_id: "all_domain_parser_fixture_coverage".to_string(),
                output_path: "artifacts/bench-readiness/parser-fixture-coverage.tsv".to_string(),
                ok: true,
                detail: "covered_row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "outputs".to_string(),
                surface_id: "all_domain_output_contract_coverage".to_string(),
                output_path: "artifacts/bench-readiness/output-contract-coverage.tsv".to_string(),
                ok: true,
                detail: "covered_row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "expected_results".to_string(),
                surface_id: "all_domain_expected_result_coverage".to_string(),
                output_path: "artifacts/bench-readiness/expected-result-coverage.tsv".to_string(),
                ok: true,
                detail: "covered_row_count=146".to_string(),
            },
            ActiveScopeFastValidationCheck {
                category: "reports".to_string(),
                surface_id: "all_domain_report_map_coverage".to_string(),
                output_path: "artifacts/bench-readiness/report-map-coverage.tsv".to_string(),
                ok: true,
                detail: "covered_row_count=146".to_string(),
            },
        ],
    }
}
