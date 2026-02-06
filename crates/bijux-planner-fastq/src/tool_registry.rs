use std::collections::BTreeMap;

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
        ("adapterremoval", "fastq.trim"),
        ("atropos", "fastq.trim"),
        ("bbduk", "fastq.trim"),
        ("cutadapt", "fastq.trim"),
        ("fastp", "fastq.trim"),
        ("prinseq", "fastq.filter"),
        ("seqkit", "fastq.filter"),
        ("trimmomatic", "fastq.trim"),
        ("trim_galore", "fastq.trim"),
        ("seqpurge", "fastq.trim"),
        ("seqtk", "fastq.validate_pre"),
        ("fastqc", "fastq.validate_pre"),
        ("fastqvalidator", "fastq.validate_pre"),
        ("fastqvalidator_official", "fastq.validate_pre"),
        ("fqtools", "fastq.validate_pre"),
        ("pear", "fastq.merge"),
        ("vsearch", "fastq.merge"),
        ("bbmerge", "fastq.merge"),
        ("flash2", "fastq.merge"),
        ("rcorrector", "fastq.correct"),
        ("spades", "fastq.correct"),
        ("bayeshammer", "fastq.correct"),
        ("lighter", "fastq.correct"),
        ("musket", "fastq.correct"),
        ("umi_tools", "fastq.umi"),
        ("seqkit_stats", "fastq.stats_neutral"),
        ("multiqc", "fastq.qc_post"),
        ("kraken2", "fastq.screen"),
        ("centrifuge", "fastq.screen"),
        ("metaphlan", "fastq.screen"),
        ("kaiju", "fastq.screen"),
        ("fastq_screen", "fastq.screen"),
        ("samtools", "core.prepare_reference"),
        ("planner", "fastq.preprocess"),
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
