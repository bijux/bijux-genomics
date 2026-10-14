//! Owner: BAM campaign admission behavior.
use super::*;

#[test]
fn derivation_rejects_misaligned_source_identity() {
    let mut queue = valid_queue();
    queue.campaign_id = "another-campaign".to_string();
    assert!(derive_bam_benchmark_campaign(
        valid_source_matrix(),
        &valid_appraisal(),
        &queue,
        Some("G121"),
    )
    .is_err_and(|error| error.to_string().contains("source identities")));
}

#[test]
fn bam_catalog_covers_declared_id_span() {
    assert_eq!(BAM_GOALS_CATALOG.len(), 20);
    assert_eq!(BAM_GOALS_CATALOG[0].goal_id, "G121");
    assert_eq!(BAM_GOALS_CATALOG[9].goal_id, "G130");
    assert_eq!(BAM_GOALS_CATALOG[10].goal_id, "G131");
    assert_eq!(BAM_GOALS_CATALOG[19].goal_id, "G140");
}

#[test]
fn selected_goal_ids_reject_unknown_entries() {
    let err = selected_goal_ids(Some("G121,G999")).expect_err("expected invalid goal");
    assert!(err.to_string().contains("unknown bam goals requested"));
}
