use std::collections::BTreeMap;

use bijux_dna_core::ids::id_catalog;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ToolAdapterEntry {
    pub tool_id: &'static str,
    pub adapter_id: &'static str,
}

#[must_use]
#[allow(dead_code)]
pub fn tool_registry() -> BTreeMap<&'static str, ToolAdapterEntry> {
    let mut map = BTreeMap::new();
    // Data-only registry for tool id -> adapter identifier.
    for (tool_id, adapter_id) in [
        ("adapterremoval", id_catalog::FASTQ_TRIM),
        ("atropos", id_catalog::FASTQ_TRIM),
        ("bbduk", id_catalog::FASTQ_TRIM),
        ("cutadapt", id_catalog::FASTQ_TRIM),
        ("fastp", id_catalog::FASTQ_TRIM),
        ("prinseq", id_catalog::FASTQ_FILTER),
        ("seqkit", id_catalog::FASTQ_FILTER),
        ("trimmomatic", id_catalog::FASTQ_TRIM),
        ("trim_galore", id_catalog::FASTQ_TRIM),
        ("seqpurge", id_catalog::FASTQ_TRIM),
        ("seqtk", id_catalog::FASTQ_VALIDATE_PRE),
        ("fastqc", id_catalog::FASTQ_VALIDATE_PRE),
        ("fastqvalidator", id_catalog::FASTQ_VALIDATE_PRE),
        ("fastqvalidator_official", id_catalog::FASTQ_VALIDATE_PRE),
        ("fqtools", id_catalog::FASTQ_VALIDATE_PRE),
        ("pear", id_catalog::FASTQ_MERGE),
        ("vsearch", id_catalog::FASTQ_MERGE),
        ("bbmerge", id_catalog::FASTQ_MERGE),
        ("flash2", id_catalog::FASTQ_MERGE),
        ("rcorrector", id_catalog::FASTQ_CORRECT),
        ("spades", id_catalog::FASTQ_CORRECT),
        ("bayeshammer", id_catalog::FASTQ_CORRECT),
        ("lighter", id_catalog::FASTQ_CORRECT),
        ("musket", id_catalog::FASTQ_CORRECT),
        ("umi_tools", id_catalog::FASTQ_UMI),
        ("seqkit_stats", id_catalog::FASTQ_STATS_NEUTRAL),
        ("multiqc", id_catalog::FASTQ_QC_POST),
        ("kraken2", id_catalog::FASTQ_SCREEN),
        ("centrifuge", id_catalog::FASTQ_SCREEN),
        ("metaphlan", id_catalog::FASTQ_SCREEN),
        ("kaiju", id_catalog::FASTQ_SCREEN),
        ("fastq_screen", id_catalog::FASTQ_SCREEN),
        ("samtools", id_catalog::CORE_PREPARE_REFERENCE),
        ("planner", id_catalog::FASTQ_PREPROCESS),
    ] {
        map.insert(
            tool_id,
            ToolAdapterEntry {
                tool_id,
                adapter_id,
            },
        );
    }
    map
}
