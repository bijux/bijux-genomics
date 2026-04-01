use std::collections::HashMap;

use bijux_dna_analyze::ImageQaOutcome;
use bijux_dna_core::contract::ToolRegistry;

use crate::api::{PlatformSpec, ToolImageSpec};
use crate::image_qa::support::ResolvedImage;
use crate::image_qa::{QaDataset, QaStage};

use super::postprocess::{qa_qc_post_tool, qa_screen_tool, qa_stats_tool, qa_umi_tool};
use super::preprocess::{
    qa_correct_tool, qa_filter_tool, qa_merge_tool, qa_trim_tool, qa_validate_tool,
};

pub(crate) fn run_behavioral_qa(
    stage: QaStage,
    tool: &str,
    platform: &PlatformSpec,
    catalog: &HashMap<String, ToolImageSpec>,
    registry: &ToolRegistry,
    dataset: &QaDataset,
    seqkit_image: &ResolvedImage,
) -> ImageQaOutcome {
    let outcome = match stage {
        QaStage::Trim => qa_trim_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Validate => qa_validate_tool(tool, platform, catalog, registry, dataset),
        QaStage::Filter => qa_filter_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Merge => qa_merge_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Correct => {
            qa_correct_tool(tool, platform, catalog, registry, dataset, seqkit_image)
        }
        QaStage::ReportQc => qa_qc_post_tool(tool, platform, catalog, registry, dataset),
        QaStage::Umi => qa_umi_tool(tool, platform, catalog, registry, dataset, seqkit_image),
        QaStage::Stats => qa_stats_tool(tool, platform, catalog, registry, dataset),
        QaStage::Screen => qa_screen_tool(tool, platform, catalog, registry, dataset),
    };
    match outcome {
        Ok(()) => ImageQaOutcome::Pass,
        Err(err) => ImageQaOutcome::Fail(err.to_string()),
    }
}
