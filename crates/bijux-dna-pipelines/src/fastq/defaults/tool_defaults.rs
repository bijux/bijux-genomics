use std::collections::BTreeMap;

use bijux_dna_core::ids::{StageId, ToolId};

use super::analysis_tools::fastq_analysis_tools;
use super::preprocess_tools::fastq_preprocess_tools;

pub(super) fn fastq_default_tools() -> BTreeMap<StageId, ToolId> {
    let mut tools = fastq_preprocess_tools();
    tools.extend(fastq_analysis_tools());
    tools
}
