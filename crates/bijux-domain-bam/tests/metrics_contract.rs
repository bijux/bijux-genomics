#[test]
fn bam_metrics_schema_is_registered() {
    for stage in &bijux_domain_bam::BAM_CANONICAL_STAGE_ORDER {
        let stage_id = stage.as_str();
        assert!(
            bijux_core::metrics_schema_for_stage(stage_id).is_some(),
            "missing metrics schema for {stage_id}"
        );
    }
}
