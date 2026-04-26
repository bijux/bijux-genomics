use std::collections::BTreeSet;

use bijux_dna_core::contract::PipelineSpec;
use bijux_dna_core::prelude::StageId;
use bijux_dna_domain_fastq::{
    canonical_amplicon_stage_order, canonical_stage_order, default_amplicon_preprocess_stage_order,
    default_shotgun_preprocess_stage_order, preprocess_pipeline_graph_for_stage_order,
    FastqPipelineMode, STAGE_CORRECT_ERRORS, STAGE_MERGE_PAIRS, STAGE_REPORT_QC,
    STAGE_SCREEN_TAXONOMY,
};

pub(crate) fn pipeline_spec_from_stage_catalog(
    stages: Vec<String>,
    mode: FastqPipelineMode,
) -> PipelineSpec {
    preprocess_pipeline_graph_for_stage_order(
        &sort_stages_by_domain_order(stages, mode)
            .into_iter()
            .map(StageId::new)
            .collect::<Vec<_>>(),
    )
}

fn sort_stages_by_domain_order(stages: Vec<String>, mode: FastqPipelineMode) -> Vec<String> {
    let order = match mode {
        FastqPipelineMode::Shotgun => canonical_stage_order(),
        FastqPipelineMode::Amplicon => canonical_amplicon_stage_order(),
    };
    let mut selected = stages.into_iter().collect::<BTreeSet<_>>();
    let mut ordered = order
        .into_iter()
        .map(|stage| stage.as_str().to_string())
        .filter(|stage| selected.remove(stage))
        .collect::<Vec<_>>();
    ordered.extend(selected);
    ordered
}

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct DefaultPipelineOptions {
    pub paired: bool,
    pub enable_merge: bool,
    pub enable_correct: bool,
    pub enable_qc_post: bool,
    pub enable_screen: bool,
    pub mode: FastqPipelineMode,
}

impl Default for DefaultPipelineOptions {
    fn default() -> Self {
        Self {
            paired: false,
            enable_merge: true,
            enable_correct: false,
            enable_qc_post: true,
            enable_screen: false,
            mode: FastqPipelineMode::Shotgun,
        }
    }
}

#[must_use]
pub fn default_pipeline_spec(options: DefaultPipelineOptions) -> PipelineSpec {
    let mut selected_stages = if options.mode == FastqPipelineMode::Amplicon {
        default_amplicon_preprocess_stage_order()
            .into_iter()
            .map(|stage| stage.as_str().to_string())
            .collect::<BTreeSet<_>>()
    } else {
        default_shotgun_preprocess_stage_order()
            .into_iter()
            .map(|stage| stage.as_str().to_string())
            .collect::<BTreeSet<_>>()
    };
    if options.enable_correct {
        selected_stages.insert(STAGE_CORRECT_ERRORS.as_str().to_string());
    }
    if options.mode == FastqPipelineMode::Shotgun && options.paired && options.enable_merge {
        selected_stages.insert(STAGE_MERGE_PAIRS.as_str().to_string());
    }
    if options.enable_screen {
        selected_stages.insert(STAGE_SCREEN_TAXONOMY.as_str().to_string());
    } else {
        selected_stages.remove(STAGE_SCREEN_TAXONOMY.as_str());
    }
    if options.enable_qc_post {
        selected_stages.insert(STAGE_REPORT_QC.as_str().to_string());
    } else {
        selected_stages.remove(STAGE_REPORT_QC.as_str());
    }
    preprocess_pipeline_graph_for_stage_order(
        &sort_stages_by_domain_order(selected_stages.into_iter().collect(), options.mode)
            .into_iter()
            .map(StageId::new)
            .collect::<Vec<_>>(),
    )
}
