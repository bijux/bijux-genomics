use std::collections::BTreeMap;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;

pub(super) fn fastq_preprocess_tools() -> BTreeMap<StageId, ToolId> {
    BTreeMap::from([
        (
            StageId::from_static("fastq.validate_reads"),
            ToolId::from_static(id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL),
        ),
        (
            StageId::from_static("fastq.correct_errors"),
            ToolId::from_static(id_catalog::TOOL_RCORRECTOR),
        ),
        (
            StageId::from_static("fastq.extract_umis"),
            ToolId::from_static(id_catalog::TOOL_UMI_TOOLS),
        ),
        (
            StageId::from_static("fastq.detect_adapters"),
            ToolId::from_static(id_catalog::TOOL_FASTQC),
        ),
        (
            StageId::from_static("fastq.trim_reads"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.trim_polyg_tails"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.trim_terminal_damage"),
            ToolId::from_static(id_catalog::TOOL_CUTADAPT),
        ),
        (
            StageId::from_static("fastq.filter_reads"),
            ToolId::from_static(id_catalog::TOOL_FASTP),
        ),
        (
            StageId::from_static("fastq.merge_pairs"),
            ToolId::from_static(id_catalog::TOOL_PEAR),
        ),
    ])
}
