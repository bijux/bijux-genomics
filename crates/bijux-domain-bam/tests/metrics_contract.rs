#[test]
fn bam_metrics_schema_is_registered() {
    for stage in bijux_domain_bam::BamStage::all() {
        let stage_id = stage.as_str();
        assert!(
            bijux_core::metrics_registry::metrics_schema_for_stage(stage_id).is_some(),
            "missing metrics schema for {stage_id}"
        );
    }
}
