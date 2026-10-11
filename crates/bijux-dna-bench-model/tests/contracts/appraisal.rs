use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    validate_appraisal_report, validate_hardening_queue_report, AppraisalFinding, AppraisalReport,
    AppraisalSummary, HardeningQueueEntry, HardeningQueueReport, APPRAISAL_REPORT_SCHEMA_V1,
    HARDENING_QUEUE_REPORT_SCHEMA_V1,
};

#[test]
fn appraisal_report_validates_findings_order_and_summary() {
    assert!(validate_appraisal_report(&appraisal()).is_ok());

    let mut drifted = appraisal();
    drifted.summary.total_findings = 0;
    assert!(validate_appraisal_report(&drifted)
        .is_err_and(|error| error.to_string().contains("summary drifted")));

    let mut duplicated = appraisal();
    duplicated.findings.insert(1, duplicated.findings[0].clone());
    duplicated.summary.total_findings += 1;
    *duplicated.summary.by_appraiser.entry("artifact-validity".to_string()).or_insert(0) += 1;
    *duplicated.summary.by_severity.entry("critical".to_string()).or_insert(0) += 1;
    assert!(validate_appraisal_report(&duplicated)
        .is_err_and(|error| error.to_string().contains("duplicate finding")));

    let mut unordered = appraisal();
    unordered.findings.reverse();
    assert!(validate_appraisal_report(&unordered)
        .is_err_and(|error| error.to_string().contains("canonical order")));
}

#[test]
fn hardening_queue_validates_allocation_order_and_group_links() {
    assert!(validate_hardening_queue_report(&queue()).is_ok());

    let mut gap = queue();
    gap.entries[1].queue_id = "hardening-0003".to_string();
    assert!(validate_hardening_queue_report(&gap)
        .is_err_and(|error| error.to_string().contains("not contiguous")));

    let mut unsorted = queue();
    unsorted.entries[0].affected_rows = vec!["row-b".to_string(), "row-a".to_string()];
    assert!(validate_hardening_queue_report(&unsorted)
        .is_err_and(|error| error.to_string().contains("entry is invalid")));
}

#[test]
fn appraisal_schemas_match_models_and_versions_fail_closed() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    for (name, generated) in [
        (
            "appraisal-report.schema.json",
            serde_json::to_value(bijux_dna_bench_model::appraisal_report_schema())?,
        ),
        (
            "hardening-queue-report.schema.json",
            serde_json::to_value(bijux_dna_bench_model::hardening_queue_report_schema())?,
        ),
    ] {
        let path = repository_root.join("benchmarks/contracts/schemas").join(name);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{name} drifted");
    }

    let mut appraisal_value = serde_json::to_value(appraisal())?;
    appraisal_value["schema_version"] = serde_json::json!("bijux.hpc.appraisal.v2");
    assert!(serde_json::from_value::<AppraisalReport>(appraisal_value).is_err());

    let mut queue_value = serde_json::to_value(queue())?;
    queue_value["schema_version"] = serde_json::json!("bijux.hpc.hardening_queue.v2");
    assert!(serde_json::from_value::<HardeningQueueReport>(queue_value).is_err());
    Ok(())
}

fn appraisal() -> AppraisalReport {
    AppraisalReport {
        schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        findings: vec![
            finding(
                "artifact-validity",
                "row-a",
                "critical",
                "high",
                "missing-tool-binding",
                "encrypted-results",
            ),
            finding(
                "runtime-performance",
                "row-b",
                "warning",
                "medium",
                "runtime-under-sampled",
                "encrypted-results",
            ),
        ],
        summary: AppraisalSummary {
            total_findings: 2,
            by_appraiser: BTreeMap::from([
                ("artifact-validity".to_string(), 1),
                ("runtime-performance".to_string(), 1),
            ]),
            by_severity: BTreeMap::from([("critical".to_string(), 1), ("warning".to_string(), 1)]),
        },
    }
}

fn finding(
    appraiser_id: &str,
    row_id: &str,
    severity: &str,
    confidence: &str,
    failure_class: &str,
    result_scope: &str,
) -> AppraisalFinding {
    AppraisalFinding {
        appraiser_id: appraiser_id.to_string(),
        row_id: row_id.to_string(),
        severity: severity.to_string(),
        confidence: confidence.to_string(),
        failure_class: failure_class.to_string(),
        result_scope: result_scope.to_string(),
        summary: "finding summary".to_string(),
        recommendation: "finding recommendation".to_string(),
    }
}

fn queue() -> HardeningQueueReport {
    HardeningQueueReport {
        schema_version: HARDENING_QUEUE_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-fastq".to_string(),
        domain: "fastq".to_string(),
        entries: vec![
            HardeningQueueEntry {
                queue_id: "hardening-0001".to_string(),
                severity: "critical".to_string(),
                failure_class: "missing-tool-binding".to_string(),
                recommendation: "bind the tool".to_string(),
                affected_rows: vec!["row-a".to_string(), "row-b".to_string()],
                source_appraisers: vec!["artifact-validity".to_string()],
            },
            HardeningQueueEntry {
                queue_id: "hardening-0002".to_string(),
                severity: "warning".to_string(),
                failure_class: "runtime-under-sampled".to_string(),
                recommendation: "increase repetitions".to_string(),
                affected_rows: vec!["row-b".to_string()],
                source_appraisers: vec!["runtime-performance".to_string()],
            },
        ],
    }
}
