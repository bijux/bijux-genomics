use super::{
    contracts::{self, model::PublishedAdapterReport},
    ports::*,
    render_vcf_all_retained_tools_complete,
};
use anyhow::{bail, ensure, Result};
use std::{
    collections::BTreeSet,
    path::{Path, PathBuf},
};
fn unavailable<T>(root: &Path, _: PathBuf) -> Result<T> {
    ensure!(root.join("root-marker").is_file(), "explicit repository was lost");
    bail!("unavailable execution provider")
}
fn unavailable_phasing<T>(root: &Path, _: &str, path: PathBuf) -> Result<T> {
    unavailable(root, path)
}
fn unavailable_direct(root: &Path, _: DirectReadinessStage, path: PathBuf) -> Result<()> {
    unavailable(root, path)
}
#[allow(
    clippy::unnecessary_wraps,
    reason = "Test double implements the fallible external admission port."
)]
fn no_external(_: &Path) -> Result<BTreeSet<(String, String)>> {
    Ok(BTreeSet::new())
}
#[allow(
    clippy::unnecessary_wraps,
    reason = "Test double implements the fallible external admission port."
)]
fn licensed(_: &Path) -> Result<BTreeSet<(String, String)>> {
    Ok(BTreeSet::from([("vcf.call_gl".to_owned(), "angsd".to_owned())]))
}
fn ports() -> VcfCompletionPorts<'static> {
    VcfCompletionPorts {
        direct: DirectReadinessPorts {
            output_path: DirectReadinessStage::stage_id,
            render: &unavailable_direct,
            external_bindings: no_external,
        },
        evidence: EvidencePublicationPorts {
            matrix: unavailable,
            smoke: unavailable,
            parsers: unavailable,
            commands: unavailable,
            expectations: unavailable,
            reports: unavailable,
        },
        adapters: AdapterPublicationPorts {
            phasing: unavailable_phasing,
            imputation: unavailable,
            descent: unavailable,
        },
    }
}
#[test]
fn unavailable_providers_publish_all_checks_using_the_explicit_repository() -> Result<()> {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("root-marker"), b"available")?;
    let previous = std::env::current_dir()?;
    let report_path = root.path().join("reports/completion.json");
    assert!(render_vcf_all_retained_tools_complete(
        root.path(),
        Path::new("reports/completion.json"),
        &ports()
    )
    .is_err());
    assert_eq!(previous, std::env::current_dir()?);
    let report: serde_json::Value = serde_json::from_slice(&std::fs::read(report_path)?)?;
    assert_eq!(report["checked_goal_count"], 24);
    assert_eq!(report["failed_goal_count"], 24);
    assert_eq!(report["ok"], false);
    let checks = report["checks"].as_array().ok_or_else(|| anyhow::anyhow!("checks"))?;
    assert_eq!(
        checks.iter().filter_map(|row| row["goal_id"].as_u64()).collect::<Vec<_>>(),
        (336..360).collect::<Vec<_>>()
    );
    assert!(checks.iter().all(|row| row["detail"]
        .as_str()
        .is_some_and(|detail| detail.contains("unavailable execution provider"))));
    Ok(())
}
#[test]
fn external_execution_blockers_survive_failed_structural_rendering() -> Result<()> {
    let root = tempfile::tempdir()?;
    std::fs::write(root.path().join("root-marker"), b"available")?;
    let mut direct = ports().direct;
    direct.external_bindings = licensed;
    let result = contracts::render_and_validate_direct_vcf_ready_output(
        root.path(),
        &direct,
        DirectReadinessStage::CallGl,
        "GL readiness",
        2,
    );
    let message = result
        .err()
        .ok_or_else(|| anyhow::anyhow!("licensed evidence must remain blocked"))?
        .to_string();
    assert!(message.contains("blocked_external_access"));
    assert!(message.contains("angsd"));
    assert!(message.contains("interface_smoke=incomplete: unavailable execution provider"));
    Ok(())
}
#[test]
fn typed_phasing_admission_rejects_missing_identity_and_false_or_mistyped_evidence() -> Result<()> {
    let payload = serde_json::json!({"row_count":1,"benchmark_ready_row_count":1,"parser_output_row_count":1,"indexed_row_count":1,"missing_input_test_passed_row_count":1,"rows":[{"tool_id":"shapeit","stage_id":"vcf.phasing","benchmark_status":"benchmark_ready","argv_validation_passed":true,"missing_input_test_passed":true}]});
    let report: PublishedAdapterReport = serde_json::from_value(payload.clone())?;
    assert!(contracts::ensure_single_phasing_output(
        &report,
        "shapeit",
        "benchmark_ready",
        1,
        1,
        "phasing"
    )
    .is_ok());
    for (field, value) in [
        ("tool_id", serde_json::Value::Null),
        ("stage_id", serde_json::json!("vcf.impute")),
        ("argv_validation_passed", serde_json::json!(false)),
        ("missing_input_test_passed", serde_json::Value::Null),
    ] {
        let mut invalid = payload.clone();
        invalid["rows"][0][field] = value;
        let report = serde_json::from_value(invalid)?;
        assert!(contracts::ensure_single_phasing_output(
            &report,
            "shapeit",
            "benchmark_ready",
            1,
            1,
            "phasing"
        )
        .is_err());
    }
    let mut wrong_type = payload;
    wrong_type["indexed_row_count"] = serde_json::json!("1");
    assert!(serde_json::from_value::<PublishedAdapterReport>(wrong_type).is_err());
    Ok(())
}
#[test]
fn typed_family_counts_and_requested_bindings_remain_required() -> Result<()> {
    let payload = serde_json::json!({"row_count":5,"tool_count":5,"benchmark_ready_row_count":5,"parser_output_row_count":5,"normalized_output_row_count":5,"missing_input_test_passed_row_count":5,"rows":[{"tool_id":"plink2","stage_id":"vcf.roh","benchmark_status":"benchmark_ready","argv_validation_passed":true,"missing_input_test_passed":true}]});
    let report: PublishedAdapterReport = serde_json::from_value(payload.clone())?;
    assert!(contracts::ensure_descent_family_output_contract(&report).is_ok());
    assert!(contracts::ensure_descent_family_output(
        &report,
        "plink2",
        "vcf.roh",
        "benchmark_ready"
    )
    .is_ok());
    assert!(contracts::ensure_descent_family_output(
        &report,
        "germline",
        "vcf.ibd",
        "benchmark_ready"
    )
    .is_err());
    assert!(contracts::ensure_imputation_family_output(
        &report,
        "beagle",
        "vcf.impute",
        "benchmark_ready"
    )
    .is_err());
    for field in [
        "row_count",
        "tool_count",
        "benchmark_ready_row_count",
        "parser_output_row_count",
        "normalized_output_row_count",
        "missing_input_test_passed_row_count",
    ] {
        let mut invalid = payload.clone();
        invalid[field] = serde_json::Value::Null;
        let report = serde_json::from_value(invalid)?;
        assert!(contracts::ensure_descent_family_output_contract(&report).is_err());
    }
    Ok(())
}
