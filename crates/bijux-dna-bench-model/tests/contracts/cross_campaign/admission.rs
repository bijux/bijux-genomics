//! Owner: Cross-domain campaign admission contracts.
#![cfg(test)]
use super::*;

#[test]
fn derivation_rejects_misaligned_source_identity() {
    let matrix = valid_source_matrix();
    let appraisal = AppraisalReport {
        schema_version: APPRAISAL_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "campaign-cross".to_string(),
        domain: "cross".to_string(),
        findings: Vec::new(),
        summary: AppraisalSummary {
            total_findings: 0,
            by_appraiser: BTreeMap::new(),
            by_severity: BTreeMap::new(),
        },
    };
    let queue = HardeningQueueReport {
        schema_version: HARDENING_QUEUE_REPORT_SCHEMA_V1.to_string(),
        campaign_id: "another-campaign".to_string(),
        domain: "cross".to_string(),
        entries: Vec::new(),
    };
    assert!(derive_cross_benchmark_campaign(matrix, &appraisal, &queue, Some("G161"))
        .is_err_and(|error| error.to_string().contains("source identities")));
}

#[test]
fn cross_catalog_covers_declared_id_span() {
    assert_eq!(CROSS_GOALS_CATALOG.len(), 10);
    assert_eq!(CROSS_GOALS_CATALOG[0].goal_id, "G161");
    assert_eq!(CROSS_GOALS_CATALOG[9].goal_id, "G170");
}

#[test]
fn selected_goal_ids_reject_unknown_entries() {
    let err = selected_goal_ids(Some("G161,G999")).expect_err("expected invalid goal");
    assert!(err.to_string().contains("unknown cross goals requested"));
}
