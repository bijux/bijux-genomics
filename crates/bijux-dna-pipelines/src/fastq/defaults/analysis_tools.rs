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
        (
            StageId::from_static("fastq.deplete_rrna"),
            ToolId::from_static(id_catalog::TOOL_SORTMERNA),
        ),
        (StageId::from_static("fastq.deplete_host"), ToolId::from_static(id_catalog::TOOL_BOWTIE2)),
        (
            StageId::from_static("fastq.deplete_reference_contaminants"),
            ToolId::from_static(id_catalog::TOOL_BOWTIE2),
        ),
        (
            StageId::from_static("fastq.normalize_primers"),
            ToolId::from_static(id_catalog::TOOL_CUTADAPT),
        ),
        (
            StageId::from_static("fastq.remove_chimeras"),
            ToolId::from_static(id_catalog::TOOL_VSEARCH),
        ),
        (StageId::from_static("fastq.infer_asvs"), ToolId::from_static(id_catalog::TOOL_DADA2)),
        (StageId::from_static("fastq.cluster_otus"), ToolId::from_static(id_catalog::TOOL_VSEARCH)),
        (
            StageId::from_static("fastq.normalize_abundance"),
            ToolId::from_static(id_catalog::TOOL_SEQKIT),
        ),
    ])
}
