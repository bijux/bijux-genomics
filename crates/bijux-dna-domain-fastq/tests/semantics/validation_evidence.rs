use anyhow::Result;
use bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS;
use bijux_dna_domain_fastq::{ValidatedReadsManifestV1, ValidationReportV1};
use serde_json::{json, Value};
fn report() -> Value {
    json!({"schema_version":"bijux.fastq.validate.report.v1","stage":FASTQ_VALIDATE_READS,"stage_id":FASTQ_VALIDATE_READS,"tool_id":"bijux","validation_mode":"strict","pair_sync_policy":"require_header_sync","input_r1":"reads R1.fastq","input_r2":"reads R2.fastq","validation_log_r1":"R1.log","validation_log_r2":"R2.log","validated_inputs":2,"validated_reads_r1":2,"validated_reads_r2":2,"validated_pairs":2,"status_r1":0,"status_r2":0,"pair_sync_checked":true,"pair_sync_pass":true,"pair_count_match":true,"failure_class":"none","strict_pass":true,"exit_code":0})
}
fn lineage() -> Value {
    json!({"schema_version":"bijux.fastq.validate.lineage.v1","stage_id":FASTQ_VALIDATE_READS,"tool_id":"bijux","validation_mode":"strict","pair_sync_policy":"require_header_sync","input_r1":"reads R1.fastq","input_r2":"reads R2.fastq","validation_report":"validation.json","paired_mode":"paired_end","validated_stream_ids":["reads_r1","reads_r2"],"pair_sync_checked":true,"pair_sync_pass":true,"validated_pairs":2})
}
fn admit(value: Value) -> Result<()> {
    serde_json::from_value::<ValidationReportV1>(value)?.validate_consistency()
}
#[test]
fn valid_paired_evidence_preserves_wire_round_trip() -> Result<()> {
    let value = report();
    let report: ValidationReportV1 = serde_json::from_value(value.clone())?;
    report.validate_consistency()?;
    assert_eq!(serde_json::to_value(&report)?, value);
    let value = lineage();
    let lineage: ValidatedReadsManifestV1 = serde_json::from_value(value.clone())?;
    lineage.validate_against(&report)?;
    assert_eq!(serde_json::to_value(&lineage)?, value);
    Ok(())
}
#[test]
fn report_rejects_contradictory_identity_counts_pairing_and_outcomes() {
    for (field, value) in [
        ("schema_version", json!("future")),
        ("stage", json!("foreign")),
        ("stage_id", json!("foreign")),
        ("tool_id", json!(" padded ")),
        ("input_r1", json!("")),
        ("input_r2", json!("")),
        ("validation_log_r2", Value::Null),
        ("validated_inputs", json!(1)),
        ("validated_reads_r2", Value::Null),
        ("validated_pairs", json!(3)),
        ("pair_count_match", json!(false)),
        ("pair_sync_checked", json!(false)),
        ("pair_sync_policy", json!("not_applicable")),
        ("pair_sync_pass", Value::Null),
        ("status_r1", json!(1)),
        ("strict_pass", json!(false)),
        ("failure_class", json!("empty_input")),
        ("exit_code", json!(96)),
    ] {
        let mut evidence = report();
        evidence[field] = value;
        assert!(admit(evidence).is_err(), "accepted contradictory {field}");
    }
}
#[test]
fn lineage_rejects_foreign_identity_parameters_and_stream_order() -> Result<()> {
    let report: ValidationReportV1 = serde_json::from_value(report())?;
    for (field, value) in [
        ("schema_version", json!("future")),
        ("stage_id", json!("foreign")),
        ("tool_id", json!("other")),
        ("input_r1", json!("other.fastq")),
        ("input_r2", Value::Null),
        ("validation_report", json!("")),
        ("validation_mode", json!("report_only")),
        ("pair_sync_policy", json!("skip_header_sync")),
        ("paired_mode", json!("single_end")),
        ("validated_stream_ids", json!(["reads_r2", "reads_r1"])),
        ("pair_sync_checked", json!(false)),
        ("pair_sync_pass", Value::Null),
        ("validated_pairs", json!(3)),
    ] {
        let mut evidence = lineage();
        evidence[field] = value;
        let lineage: ValidatedReadsManifestV1 = serde_json::from_value(evidence)?;
        assert!(lineage.validate_against(&report).is_err(), "accepted contradictory {field}");
    }
    Ok(())
}
#[test]
fn report_only_accepts_scientific_failure_without_claiming_strict_success() -> Result<()> {
    let mut value = report();
    value["failure_class"] = json!("malformed_record");
    value["status_r1"] = json!(1);
    value["strict_pass"] = json!(false);
    value["pair_sync_pass"] = Value::Null;
    value["validation_mode"] = json!("report_only");
    admit(value.clone())?;
    value["validation_mode"] = json!("strict");
    assert!(admit(value.clone()).is_err());
    value["exit_code"] = json!(96);
    admit(value.clone())?;
    value["failure_class"] = json!("none");
    assert!(admit(value).is_err());
    Ok(())
}
#[test]
fn pair_failure_taxonomy_requires_matching_observations() -> Result<()> {
    let mut value = report();
    value["failure_class"] = json!("pair_count_mismatch");
    value["strict_pass"] = json!(false);
    value["exit_code"] = json!(96);
    assert!(admit(value.clone()).is_err());
    value["validated_reads_r2"] = json!(1);
    value["validated_pairs"] = json!(1);
    value["pair_count_match"] = json!(false);
    value["pair_sync_pass"] = Value::Null;
    admit(value)?;
    let mut value = report();
    value["failure_class"] = json!("header_sync_mismatch");
    value["strict_pass"] = json!(false);
    value["exit_code"] = json!(96);
    assert!(admit(value.clone()).is_err());
    value["pair_sync_pass"] = json!(false);
    admit(value)?;
    Ok(())
}
#[test]
fn single_end_and_skipped_sync_do_not_invent_pair_evidence() -> Result<()> {
    let mut paired = report();
    paired["pair_sync_policy"] = json!("skip_header_sync");
    paired["pair_sync_checked"] = json!(false);
    paired["pair_sync_pass"] = Value::Null;
    admit(paired.clone())?;
    paired["pair_sync_pass"] = json!(true);
    assert!(admit(paired).is_err());
    let mut single = report();
    for field in [
        "input_r2",
        "validation_log_r2",
        "validated_reads_r2",
        "validated_pairs",
        "pair_sync_pass",
        "pair_count_match",
    ] {
        single[field] = Value::Null;
    }
    single["validated_inputs"] = json!(1);
    single["pair_sync_policy"] = json!("not_applicable");
    single["pair_sync_checked"] = json!(false);
    admit(single.clone())?;
    single["status_r2"] = json!(1);
    assert!(admit(single).is_err());
    Ok(())
}
#[test]
fn overflowing_read_totals_and_false_empty_classification_are_rejected() {
    let mut value = report();
    for field in ["validated_reads_r1", "validated_reads_r2", "validated_pairs"] {
        value[field] = json!(u64::MAX);
    }
    assert!(admit(value).is_err());
    let mut value = report();
    value["failure_class"] = json!("empty_input");
    value["strict_pass"] = json!(false);
    value["exit_code"] = json!(96);
    assert!(admit(value).is_err());
}
