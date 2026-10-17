//! Owner: FASTQ campaign admission contracts.
#![cfg(test)]
use super::*;

#[test]
fn derivation_rejects_misaligned_source_identity() {
    let mut queue = valid_queue();
    queue.campaign_id = "another-campaign".to_string();
    assert!(derive_fastq_benchmark_campaign(
        valid_source_matrix(),
        &valid_appraisal(),
        &queue,
        Some("G101"),
    )
    .is_err_and(|error| error.to_string().contains("source identities")));
}

#[test]
fn fastq_catalog_covers_declared_id_span() {
    assert_eq!(FASTQ_GOALS_CATALOG.len(), 20);
    assert_eq!(FASTQ_GOALS_CATALOG[0].goal_id, "G101");
    assert_eq!(FASTQ_GOALS_CATALOG[9].goal_id, "G110");
    assert_eq!(FASTQ_GOALS_CATALOG[10].goal_id, "G111");
    assert_eq!(FASTQ_GOALS_CATALOG[19].goal_id, "G120");
}

#[test]
fn selected_goal_ids_reject_unknown_entries() {
    let err = selected_goal_ids(Some("G101,G999")).expect_err("expected invalid goal");
    assert!(err.to_string().contains("unknown fastq goals requested"));
}
