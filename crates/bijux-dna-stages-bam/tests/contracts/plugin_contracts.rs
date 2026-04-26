use bijux_dna_stage_contract::StagePlugin;
use bijux_dna_stages_bam::BamStagePlugin;

#[test]
fn bam_stage_plugin_handles_only_registered_bam_stage_ids() {
    let plugin = BamStagePlugin;

    assert!(plugin.handles_stage("bam.align"));
    assert!(!plugin.handles_stage("bam.not_registered"));
    assert!(!plugin.handles_stage("fastq.validate_reads"));
}
