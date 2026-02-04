use bijux_core::stage_plugin::StagePlugin;

pub fn select_stage_plugin(stage_id: &str) -> Option<Box<dyn StagePlugin>> {
    let candidates: Vec<Box<dyn StagePlugin>> = vec![
        Box::new(bijux_stages_fastq::plugin::FastqStagePlugin),
        Box::new(bijux_stages_bam::plugin::BamStagePlugin),
    ];
    candidates
        .into_iter()
        .find(|plugin| plugin.handles_stage(stage_id))
}

