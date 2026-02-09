use std::collections::BTreeMap;

use bijux_dna_core::ids::{ToolId, id_catalog};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolAdapterEntry {
    pub tool_id: ToolId,
    pub adapter_id: &'static str,
}

#[must_use]
#[allow(dead_code)]
pub fn tool_registry() -> BTreeMap<ToolId, ToolAdapterEntry> {
    let mut map = BTreeMap::new();
    // Data-only registry for tool id -> adapter identifier.
    for (tool_id, adapter_id) in [
        (ToolId::from_static("adapterremoval"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("atropos"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("bbduk"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("cutadapt"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("fastp"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("prinseq"), id_catalog::FASTQ_FILTER),
        (ToolId::from_static("seqkit"), id_catalog::FASTQ_FILTER),
        (ToolId::from_static("trimmomatic"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("trim_galore"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("seqpurge"), id_catalog::FASTQ_TRIM),
        (ToolId::from_static("seqtk"), id_catalog::FASTQ_VALIDATE_PRE),
        (ToolId::from_static("fastqc"), id_catalog::FASTQ_VALIDATE_PRE),
        (
            ToolId::from_static("fastqvalidator"),
            id_catalog::FASTQ_VALIDATE_PRE,
        ),
        (
            ToolId::from_static("fastqvalidator_official"),
            id_catalog::FASTQ_VALIDATE_PRE,
        ),
        (ToolId::from_static("fqtools"), id_catalog::FASTQ_VALIDATE_PRE),
        (ToolId::from_static("pear"), id_catalog::FASTQ_MERGE),
        (ToolId::from_static("vsearch"), id_catalog::FASTQ_MERGE),
        (ToolId::from_static("bbmerge"), id_catalog::FASTQ_MERGE),
        (ToolId::from_static("flash2"), id_catalog::FASTQ_MERGE),
        (ToolId::from_static("rcorrector"), id_catalog::FASTQ_CORRECT),
        (ToolId::from_static("spades"), id_catalog::FASTQ_CORRECT),
        (ToolId::from_static("bayeshammer"), id_catalog::FASTQ_CORRECT),
        (ToolId::from_static("lighter"), id_catalog::FASTQ_CORRECT),
        (ToolId::from_static("musket"), id_catalog::FASTQ_CORRECT),
        (ToolId::from_static("umi_tools"), id_catalog::FASTQ_UMI),
        (
            ToolId::from_static("seqkit_stats"),
            id_catalog::FASTQ_STATS_NEUTRAL,
        ),
        (ToolId::from_static("multiqc"), id_catalog::FASTQ_QC_POST),
        (ToolId::from_static("kraken2"), id_catalog::FASTQ_SCREEN),
        (ToolId::from_static("centrifuge"), id_catalog::FASTQ_SCREEN),
        (ToolId::from_static("metaphlan"), id_catalog::FASTQ_SCREEN),
        (ToolId::from_static("kaiju"), id_catalog::FASTQ_SCREEN),
        (ToolId::from_static("fastq_screen"), id_catalog::FASTQ_SCREEN),
        (ToolId::from_static("samtools"), id_catalog::CORE_PREPARE_REFERENCE),
        (ToolId::from_static("planner"), id_catalog::FASTQ_PREPROCESS),
    ] {
        let key = tool_id.clone();
        map.insert(
            key,
            ToolAdapterEntry {
                tool_id,
                adapter_id,
            },
        );
    }
    map
}
