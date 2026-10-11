use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    scientific_acceptance_thresholds_report_schema, scientific_acceptance_thresholds_schema,
    validate_scientific_acceptance_thresholds_config,
    validate_scientific_acceptance_thresholds_report, ScientificAcceptanceDirection,
    ScientificAcceptanceInsufficiencyBehavior, ScientificAcceptancePassRule,
    ScientificAcceptanceThresholdRow, ScientificAcceptanceThresholdsConfig,
    ScientificAcceptanceThresholdsReport, ScientificAcceptanceToleranceKind,
    SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1, SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1,
};

#[test]
fn scientific_acceptance_schemas_match_checked_in_contracts() -> anyhow::Result<()> {
    assert_schema(
        "scientific-acceptance-thresholds.schema.json",
        &serde_json::to_value(scientific_acceptance_thresholds_schema())?,
    )?;
    assert_schema(
        "scientific-acceptance-thresholds-report.schema.json",
        &serde_json::to_value(scientific_acceptance_thresholds_report_schema())?,
    )
}

#[test]
fn scientific_acceptance_contracts_admit_exact_semantic_closure() -> anyhow::Result<()> {
    let config = valid_config();
    validate_scientific_acceptance_thresholds_config(&config)?;
    validate_scientific_acceptance_thresholds_report(&valid_report(config.rows))?;
    Ok(())
}

#[test]
fn scientific_acceptance_config_rejects_duplicate_metric_identity() {
    let mut config = valid_config();
    config.rows.push(config.rows[0].clone());
    assert!(validate_scientific_acceptance_thresholds_config(&config).is_err());
}

#[test]
fn scientific_acceptance_config_rejects_empty_or_invalid_thresholds() {
    let mut config = valid_config();
    config.rows.clear();
    assert!(validate_scientific_acceptance_thresholds_config(&config).is_err());

    let mut config = valid_config();
    config.rows[0].tolerance_value = f64::NAN;
    assert!(validate_scientific_acceptance_thresholds_config(&config).is_err());
}

#[test]
fn scientific_acceptance_report_rejects_aggregate_or_path_drift() {
    let mut report = valid_report(valid_config().rows);
    report.row_count += 1;
    assert!(validate_scientific_acceptance_thresholds_report(&report).is_err());

    let mut report = valid_report(valid_config().rows);
    report.config_path = "../outside.toml".to_string();
    assert!(validate_scientific_acceptance_thresholds_report(&report).is_err());
}

fn valid_config() -> ScientificAcceptanceThresholdsConfig {
    ScientificAcceptanceThresholdsConfig {
        schema_version: SCIENTIFIC_ACCEPTANCE_THRESHOLDS_SCHEMA_V1.to_string(),
        rows: vec![ScientificAcceptanceThresholdRow {
            domain: "vcf".to_string(),
            stage_id: "vcf.imputation_metrics".to_string(),
            metric_id: "dosage_r2".to_string(),
            metric_name: "Dosage R squared".to_string(),
            unit: Some("score".to_string()),
            direction: ScientificAcceptanceDirection::HigherIsBetter,
            tolerance_kind: ScientificAcceptanceToleranceKind::AbsoluteDelta,
            tolerance_value: 0.02,
            pass_rule: ScientificAcceptancePassRule::MeetOrExceedReference,
            insufficiency_behavior: ScientificAcceptanceInsufficiencyBehavior::DropMetricFromStage,
            required: false,
            declaration_origin: "vcf_comparable_metric_policy".to_string(),
        }],
    }
}

fn valid_report(
    rows: Vec<ScientificAcceptanceThresholdRow>,
) -> ScientificAcceptanceThresholdsReport {
    ScientificAcceptanceThresholdsReport {
        schema_version: SCIENTIFIC_ACCEPTANCE_THRESHOLDS_REPORT_SCHEMA_V1.to_string(),
        config_path: "configs/bench/local/scientific-acceptance-thresholds.toml".to_string(),
        comparable_metric_count: rows.len(),
        row_count: rows.len(),
        domain_counts: BTreeMap::from([("vcf".to_string(), rows.len())]),
        direction_counts: BTreeMap::from([("higher_is_better".to_string(), rows.len())]),
        insufficiency_behavior_counts: BTreeMap::from([(
            "drop_metric_from_stage".to_string(),
            rows.len(),
        )]),
        rows,
    }
}

fn assert_schema(file_name: &str, generated: &serde_json::Value) -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path = root.join("benchmarks/contracts/schemas").join(file_name);
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(&serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}
