use std::collections::BTreeMap;

use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_core::prelude::id_catalog;

pub(super) fn fastq_analysis_tools() -> BTreeMap<StageId, ToolId> {
    BTreeMap::from([
        (
            StageId::from_static("fastq.profile_reads"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS),
        ),
        (
            StageId::from_static("fastq.profile_read_lengths"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT_STATS),
        ),
        (
            StageId::from_static("fastq.profile_overrepresented_sequences"),
            ToolId::from_static(id_catalog::TOOL_FASTQC),
        ),
        (StageId::from_static("fastq.report_qc"), ToolId::from_static(id_catalog::TOOL_MULTIQC)),
        (
            StageId::from_static("fastq.screen_taxonomy"),
            ToolId::from_static(id_catalog::TOOL_KRAKEN2),
        ),
    ])
}
