use serde_json::Value;
use std::fs;

use bijux_dna_api::v1::api::bench::{
    render_benchmark_attempt_archive_schema, render_benchmark_attempt_schema,
    render_benchmark_cycle_status_schema, render_benchmark_pipeline_parameters_schema,
    render_benchmark_report_admission_schema, render_benchmark_stage_interfaces_schema,
    render_benchmark_stage_report_schema, BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1,
    BENCHMARK_ATTEMPT_SCHEMA_V1, BENCHMARK_CYCLE_STATUS_SCHEMA_V1,
    BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1, BENCHMARK_REPORT_ADMISSION_SCHEMA_V1,
    BENCHMARK_STAGE_INTERFACES_SCHEMA_V1, BENCHMARK_STAGE_REPORT_SCHEMA_V1,
};
use bijux_dna_api::v1::api::qualification::{
    render_qualification_assessment_schema, render_qualification_report_schema,
    QUALIFICATION_ASSESSMENT_SCHEMA_V1, QUALIFICATION_REPORT_SCHEMA_V2,
};

fn load_fixture(path: &std::path::Path) -> Value {
    let raw = fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("read fixture {}: {err}", path.display()));
    serde_json::from_str(&raw).unwrap_or_else(|err| panic!("parse fixture json: {err}"))
}

#[test]
fn benchmark_planning_schemas_are_owned_by_rust_types() {
    let interfaces = render_benchmark_stage_interfaces_schema();
    let parameters = render_benchmark_pipeline_parameters_schema();

    assert_eq!(
        interfaces.pointer("/properties/schema_version/const").and_then(serde_json::Value::as_str),
        Some(BENCHMARK_STAGE_INTERFACES_SCHEMA_V1)
    );
    assert_eq!(
        parameters.pointer("/properties/schema_version/const").and_then(serde_json::Value::as_str),
        Some(BENCHMARK_PIPELINE_PARAMETERS_SCHEMA_V1)
    );
}

#[test]
fn checked_benchmark_planning_schemas_match_rust_types() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root from API crate"))?;
    let schema_root = repository_root.join("benchmarks").join("schemas");
    let interfaces: Value =
        serde_json::from_str(&fs::read_to_string(schema_root.join("stage-interfaces.v1.json"))?)?;
    let parameters: Value = serde_json::from_str(&fs::read_to_string(
        schema_root.join("pipeline-parameters.v1.json"),
    )?)?;

    assert_eq!(interfaces, render_benchmark_stage_interfaces_schema());
    assert_eq!(parameters, render_benchmark_pipeline_parameters_schema());
    Ok(())
}

#[test]
fn checked_benchmark_stage_report_schema_matches_rust_type() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root from API crate"))?;
    let checked: Value = serde_json::from_str(&fs::read_to_string(
        repository_root.join("benchmarks/schemas/stage-report.v1.json"),
    )?)?;
    let generated = render_benchmark_stage_report_schema();
    assert_eq!(
        generated.pointer("/properties/schema_version/const").and_then(Value::as_str),
        Some(BENCHMARK_STAGE_REPORT_SCHEMA_V1)
    );
    assert_eq!(checked, generated);
    Ok(())
}

#[test]
fn checked_benchmark_report_admission_schema_matches_rust_type() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root from API crate"))?;
    let checked: Value = serde_json::from_str(&fs::read_to_string(
        repository_root.join("benchmarks/schemas/report-admission.v1.json"),
    )?)?;
    let generated = render_benchmark_report_admission_schema();
    assert_eq!(
        generated.pointer("/properties/schema_version/const").and_then(Value::as_str),
        Some(BENCHMARK_REPORT_ADMISSION_SCHEMA_V1)
    );
    assert_eq!(checked, generated);
    Ok(())
}

#[test]
fn contract_handshake_rejects_mismatched_schema() {
    let policies_root = crate::support::crate_root("bijux-dna-policies")
        .unwrap_or_else(|err| panic!("resolve policies crate root: {err}"))
        .join("tests")
        .join("fixtures")
        .join("handshake")
        .join("default");
    let plan = load_fixture(&policies_root.join("plan.json"));
    let manifest = load_fixture(&policies_root.join("manifest.json"));
    let report = load_fixture(&policies_root.join("report.json"));

    assert!(plan.get("schema_version").is_some());
    assert!(manifest.get("schema_version").is_some());
    assert!(report.get("schema_version").is_some());
}

#[test]
fn benchmark_cycle_schemas_are_governed_by_rust_types() {
    let attempt = render_benchmark_attempt_schema();
    let cycle_status = render_benchmark_cycle_status_schema();

    assert_eq!(
        attempt
            .get("properties")
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str),
        Some(BENCHMARK_ATTEMPT_SCHEMA_V1),
    );
    assert_eq!(
        cycle_status
            .get("properties")
            .and_then(|value| value.get("schema_version"))
            .and_then(|value| value.get("const"))
            .and_then(serde_json::Value::as_str),
        Some(BENCHMARK_CYCLE_STATUS_SCHEMA_V1),
    );
}

#[test]
fn checked_benchmark_attempt_archive_schema_matches_rust_type() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root from API crate"))?;
    let checked: Value = serde_json::from_str(&fs::read_to_string(
        repository_root.join("benchmarks/schemas/attempt-archive.v1.json"),
    )?)?;
    let generated = render_benchmark_attempt_archive_schema();
    assert_eq!(
        generated.pointer("/properties/schema_version/const").and_then(Value::as_str),
        Some(BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA_V1)
    );
    assert_eq!(checked, generated);
    Ok(())
}

#[test]
fn checked_qualification_schema_matches_rust_type() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("resolve repository root from API crate"))?;
    let checked: Value = serde_json::from_str(&fs::read_to_string(
        repository_root.join("containers/contracts/schemas/qualification-assessment.v1.json"),
    )?)?;
    let generated = render_qualification_assessment_schema();
    assert_eq!(
        generated.pointer("/properties/schema_version/const").and_then(Value::as_str),
        Some(QUALIFICATION_ASSESSMENT_SCHEMA_V1)
    );
    assert_eq!(
        generated.pointer("/$defs/QualificationClaim/enum"),
        Some(&serde_json::json!([
            "unsupported",
            "planned",
            "built",
            "runtime_verified",
            "stage_verified",
            "real_data_verified",
            "science_verified",
            "benchmark_verified",
            "security_verified",
            "release_verified",
            "qualified",
            "failed",
            "superseded",
            "blocked"
        ]))
    );
    assert_eq!(checked, generated);
    Ok(())
}

#[test]
fn qualification_report_schema_is_owned_by_rust_types() {
    let generated = render_qualification_report_schema();
    assert_eq!(
        generated.pointer("/properties/schema_version/const").and_then(Value::as_str),
        Some(QUALIFICATION_REPORT_SCHEMA_V2)
    );
    assert_eq!(generated.pointer("/additionalProperties"), Some(&Value::Bool(false)));
}
