#[test]
fn bam_metrics_schema_is_registered() {
    for stage in bijux_dna_domain_bam::BamStage::all() {
        let stage_id = stage.as_str();
        assert!(
            bijux_dna_core::metrics::metrics_schema_for_stage(stage_id).is_some(),
            "missing metrics schema for {stage_id}"
        );
    }
}
