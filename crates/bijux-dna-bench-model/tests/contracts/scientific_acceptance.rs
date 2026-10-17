use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    derive_fastq_scientific_acceptance_threshold, derive_vcf_scientific_acceptance_threshold,
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

#[test]
fn scientific_acceptance_fastq_policy_preserves_governed_metric_rules() -> anyhow::Result<()> {
    use ScientificAcceptanceDirection::{ExactMatchPreferred, HigherIsBetter};
    use ScientificAcceptanceInsufficiencyBehavior::{RefuseStageComparison, WarnAndExcludeStage};
    use ScientificAcceptancePassRule::{MatchReference, MeetOrExceedReference};
    use ScientificAcceptanceToleranceKind::{AbsoluteDelta, ExactMatch};

    for (stage, metric, unit, direction, kind, tolerance, pass, insufficient) in [
        (
            "fastq.index_reference",
            "index_build_exit_code",
            "exit_code",
            ExactMatchPreferred,
            ExactMatch,
            0.0,
            MatchReference,
            RefuseStageComparison,
        ),
        (
            "fastq.profile_overrepresented_sequences",
            "sequence_count",
            "reads",
            ExactMatchPreferred,
            ExactMatch,
            0.0,
            MatchReference,
            WarnAndExcludeStage,
        ),
        (
            "fastq.profile_overrepresented_sequences",
            "flagged_sequences",
            "sequences",
            ExactMatchPreferred,
            AbsoluteDelta,
            1.0,
            MatchReference,
            WarnAndExcludeStage,
        ),
        (
            "fastq.profile_overrepresented_sequences",
            "top_fraction",
            "fraction",
            ExactMatchPreferred,
            AbsoluteDelta,
            0.05,
            MatchReference,
            WarnAndExcludeStage,
        ),
        (
            "fastq.validate_reads",
            "format_validation_pass_rate",
            "fraction",
            HigherIsBetter,
            AbsoluteDelta,
            0.01,
            MeetOrExceedReference,
            RefuseStageComparison,
        ),
    ] {
        let row = derive_fastq_scientific_acceptance_threshold(stage, metric)?;
        assert_eq!(row.domain, "fastq");
        assert_eq!(row.stage_id, stage);
        assert_eq!(row.metric_id, metric);
        assert_eq!(row.metric_name, metric);
        assert_eq!(row.unit.as_deref(), Some(unit));
        assert_eq!(row.direction, direction);
        assert_eq!(row.tolerance_kind, kind);
        assert_eq!(row.tolerance_value.to_bits(), f64::to_bits(tolerance));
        assert_eq!(row.pass_rule, pass);
        assert_eq!(row.insufficiency_behavior, insufficient);
        assert!(row.required);
        assert_eq!(row.declaration_origin, "fastq_comparable_metric_policy");
    }
    Ok(())
}

#[test]
fn scientific_acceptance_fastq_policy_refuses_unknown_pairs() -> anyhow::Result<()> {
    for (stage, metric) in [
        ("fastq.unknown", "top_fraction"),
        ("fastq.profile_overrepresented_sequences", "unknown_metric"),
        ("fastq.validate_reads", "top_fraction"),
    ] {
        let error = derive_fastq_scientific_acceptance_threshold(stage, metric)
            .err()
            .ok_or_else(|| anyhow::anyhow!("unknown FASTQ policy must refuse"))?;
        assert_eq!(
            error.to_string(),
            format!(
            "FASTQ comparable metric `{stage}` / `{metric}` is missing scientific acceptance policy"
        )
        );
    }
    Ok(())
}

#[test]
fn scientific_acceptance_vcf_exact_match_precedes_unit_admission() -> anyhow::Result<()> {
    for required in [false, true] {
        let row = derive_vcf_scientific_acceptance_threshold(
            "vcf.call",
            "identity",
            "Retained identity",
            "unknown-unit",
            ScientificAcceptanceDirection::ExactMatchPreferred,
            required,
        )?;
        assert_eq!(row.domain, "vcf");
        assert_eq!(row.stage_id, "vcf.call");
        assert_eq!(row.metric_id, "identity");
        assert_eq!(row.metric_name, "Retained identity");
        assert_eq!(row.unit.as_deref(), Some("unknown-unit"));
        assert_eq!(row.direction, ScientificAcceptanceDirection::ExactMatchPreferred);
        assert_eq!(row.tolerance_kind, ScientificAcceptanceToleranceKind::ExactMatch);
        assert_eq!(row.tolerance_value.to_bits(), f64::to_bits(0.0));
        assert_eq!(row.pass_rule, ScientificAcceptancePassRule::MatchReference);
        assert_eq!(row.required, required);
        assert_eq!(
            row.insufficiency_behavior,
            if required {
                ScientificAcceptanceInsufficiencyBehavior::RefuseStageComparison
            } else {
                ScientificAcceptanceInsufficiencyBehavior::DropMetricFromStage
            }
        );
        assert_eq!(row.declaration_origin, "vcf_comparable_metric_policy");
    }
    Ok(())
}

#[test]
fn scientific_acceptance_vcf_policy_preserves_metric_and_stage_exceptions() -> anyhow::Result<()> {
    for (direction, pass_rule) in [
        (
            ScientificAcceptanceDirection::HigherIsBetter,
            ScientificAcceptancePassRule::MeetOrExceedReference,
        ),
        (
            ScientificAcceptanceDirection::LowerIsBetter,
            ScientificAcceptancePassRule::NotExceedReference,
        ),
    ] {
        for unit in ["fraction", "score"] {
            for (metric, tolerance) in [
                ("concordance", 0.02),
                ("switch_error_proxy", 0.02),
                ("missingness_post", 0.02),
                ("dosage_r2", 0.05),
            ] {
                let row = derive_vcf_scientific_acceptance_threshold(
                    "vcf.imputation_metrics",
                    metric,
                    "Observed metric",
                    unit,
                    direction,
                    false,
                )?;
                assert_eq!(row.tolerance_kind, ScientificAcceptanceToleranceKind::AbsoluteDelta);
                assert_eq!(row.tolerance_value.to_bits(), f64::to_bits(tolerance));
                assert_eq!(row.pass_rule, pass_rule);
            }
        }
        for unit in [
            "sites",
            "samples",
            "variants",
            "pairs",
            "clusters",
            "populations",
            "genotypes",
            "bases",
        ] {
            for (stage, metric, tolerance) in [
                ("vcf.phasing", "phase_block_n50", 0.15),
                ("vcf.phasing", "other", 0.05),
                ("vcf.call", "phase_block_n50", 0.05),
            ] {
                let row = derive_vcf_scientific_acceptance_threshold(
                    stage,
                    metric,
                    "Observed metric",
                    unit,
                    direction,
                    true,
                )?;
                assert_eq!(row.tolerance_kind, ScientificAcceptanceToleranceKind::RelativeFraction);
                assert_eq!(row.tolerance_value.to_bits(), f64::to_bits(tolerance));
                assert_eq!(row.pass_rule, pass_rule);
            }
        }
    }
    Ok(())
}

#[test]
fn scientific_acceptance_vcf_policy_preserves_refusal_priority() -> anyhow::Result<()> {
    for direction in [
        ScientificAcceptanceDirection::HigherIsBetter,
        ScientificAcceptanceDirection::LowerIsBetter,
    ] {
        let error = derive_vcf_scientific_acceptance_threshold(
            "vcf.call",
            "metric",
            "Metric",
            "unknown-unit",
            direction,
            true,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("unknown VCF unit must refuse"))?;
        assert_eq!(
            error.to_string(),
            "VCF comparable metric unit `unknown-unit` is missing a tolerance kind policy"
        );
    }
    for direction in [
        ScientificAcceptanceDirection::Minimum,
        ScientificAcceptanceDirection::Maximum,
        ScientificAcceptanceDirection::Range,
        ScientificAcceptanceDirection::StructuredMatch,
    ] {
        let error = derive_vcf_scientific_acceptance_threshold(
            "vcf.call",
            "metric",
            "Metric",
            "unknown-unit",
            direction,
            true,
        )
        .err()
        .ok_or_else(|| anyhow::anyhow!("unsupported VCF direction must refuse"))?;
        assert_eq!(
            error.to_string(),
            format!(
                "VCF comparable metric `vcf.call` / `metric` uses unsupported direction `{}`",
                direction.as_str()
            )
        );
    }
    Ok(())
}
