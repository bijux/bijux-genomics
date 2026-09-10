use crate::{
    admit_all_domain_missing_result_audit, validate_all_domain_missing_result_audit,
    AllDomainMissingResultRow, AllDomainMissingResultStatus, AllDomainMissingResultTestReport,
    ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION, BAM_REMOVED_RESULT_ID,
    DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT, DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH,
    FASTQ_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID,
};
use std::collections::BTreeMap;

fn report() -> AllDomainMissingResultTestReport {
    let removed = [FASTQ_REMOVED_RESULT_ID, BAM_REMOVED_RESULT_ID, VCF_REMOVED_RESULT_ID];
    let mut rows = removed
        .iter()
        .map(|id| row(id, AllDomainMissingResultStatus::MissingResult))
        .collect::<Vec<_>>();
    rows.push(row("vcf:local:vcf.stats:sample:other", AllDomainMissingResultStatus::Present));
    AllDomainMissingResultTestReport {
        schema_version: ALL_DOMAIN_MISSING_RESULT_TEST_SCHEMA_VERSION.into(),
        output_path: DEFAULT_ALL_DOMAIN_MISSING_RESULT_TEST_PATH.into(),
        fake_result_root: DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT.into(),
        expected_row_count: 4,
        present_result_row_count: 1,
        missing_result_row_count: 3,
        passes_behavior_test: false,
        removed_result_ids: removed.iter().map(|id| (*id).to_string()).collect(),
        removed_manifest_paths: rows[..3]
            .iter()
            .map(|r| r.expected_manifest_path.clone())
            .collect(),
        domain_counts: BTreeMap::from([("fastq".into(), 1), ("bam".into(), 1), ("vcf".into(), 2)]),
        rows,
    }
}
fn row(id: &str, status: AllDomainMissingResultStatus) -> AllDomainMissingResultRow {
    let fields = id.split(':').collect::<Vec<_>>();
    let path = format!(
        "{}/{}/{}/{}/{}/{}/stage-result.json",
        DEFAULT_ALL_DOMAIN_MISSING_RESULT_FIXTURE_ROOT,
        fields[0],
        fields[1],
        fields[2],
        fields[3],
        fields[4]
    );
    AllDomainMissingResultRow {
        result_id: id.into(),
        domain: fields[0].into(),
        corpus_id: fields[1].into(),
        stage_id: fields[2].into(),
        asset_profile_id: fields[3].into(),
        tool_id: fields[4].into(),
        expected_manifest_path: path.clone(),
        audit_manifest_path: path,
        result_status: status,
        expected_output_artifact_ids: vec!["result_json".into()],
        observed_output_artifact_ids: if status == AllDomainMissingResultStatus::Present {
            vec!["result_json".into()]
        } else {
            vec![]
        },
        expected_metrics: vec!["records".into()],
        report_section: "illustrative".into(),
        reason: "manifest presence observation".into(),
    }
}

#[test]
fn missing_result_replay_preserves_missingness_and_closes_wire_fields() -> anyhow::Result<()> {
    let admitted = admit_all_domain_missing_result_audit(report())?;
    assert!(admitted.passes_behavior_test);
    assert_eq!(admitted.missing_result_row_count, 3);
    let wire = serde_json::to_value(&admitted)?;
    let replay: AllDomainMissingResultTestReport = serde_json::from_value(wire.clone())?;
    validate_all_domain_missing_result_audit(&replay)?;
    for (pointer, value) in [
        ("/schema_version", serde_json::json!("unknown")),
        ("/rows/0/result_status", serde_json::json!("unknown")),
    ] {
        let mut wrong = wire.clone();
        *wrong.pointer_mut(pointer).ok_or_else(|| anyhow::anyhow!("fixture field absent"))? = value;
        assert!(serde_json::from_value::<AllDomainMissingResultTestReport>(wrong).is_err());
    }
    let mut wrong = wire.clone();
    wrong["rows"][0]["unexpected"] = true.into();
    assert!(serde_json::from_value::<AllDomainMissingResultTestReport>(wrong).is_err());
    let mut wrong = wire;
    wrong["unexpected"] = true.into();
    assert!(serde_json::from_value::<AllDomainMissingResultTestReport>(wrong).is_err());
    Ok(())
}

#[test]
fn missing_result_verdict_cannot_hide_forged_counts_or_evidence_bindings() -> anyhow::Result<()> {
    let admitted = admit_all_domain_missing_result_audit(report())?;
    for mutate in [
        |v: &mut AllDomainMissingResultTestReport| v.expected_row_count = 51,
        |v: &mut AllDomainMissingResultTestReport| v.domain_counts.clear(),
        |v: &mut AllDomainMissingResultTestReport| v.present_result_row_count = 0,
        |v: &mut AllDomainMissingResultTestReport| v.rows[3].observed_output_artifact_ids.clear(),
        |v: &mut AllDomainMissingResultTestReport| {
            v.rows[0].observed_output_artifact_ids.push("claimed".into());
        },
        |v: &mut AllDomainMissingResultTestReport| v.removed_result_ids.swap(0, 1),
        |v: &mut AllDomainMissingResultTestReport| {
            v.rows[3].audit_manifest_path = v.rows[0].audit_manifest_path.clone();
        },
        |v: &mut AllDomainMissingResultTestReport| {
            v.rows[3].result_id = v.rows[0].result_id.clone();
        },
    ] {
        let mut wrong = admitted.clone();
        mutate(&mut wrong);
        assert!(validate_all_domain_missing_result_audit(&wrong).is_err());
        assert!(admit_all_domain_missing_result_audit(wrong).is_err());
    }
    let mut wrong = admitted;
    wrong.passes_behavior_test = false;
    assert!(validate_all_domain_missing_result_audit(&wrong).is_err());
    Ok(())
}
