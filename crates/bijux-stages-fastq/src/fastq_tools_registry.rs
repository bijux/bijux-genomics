use bijux_pipelines::fastq::canonical_tool_defaults;

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &str) -> Vec<String> {
    let tools: &[&str] = match stage_id {
        "fastq.preprocess" => &["planner"],
        "fastq.validate_pre" => &[
            "seqtk",
            "fastqc",
            "fastqvalidator",
            "fastqvalidator_official",
            "fqtools",
        ],
        "fastq.detect_adapters" => &["fastqc"],
        "fastq.trim" => &[
            "fastp",
            "cutadapt",
            "atropos",
            "bbduk",
            "adapterremoval",
            "trimmomatic",
            "trim_galore",
            "seqpurge",
            "prinseq",
            "seqkit",
        ],
        "fastq.filter" => &["prinseq", "fastp", "seqkit", "bbduk"],
        "fastq.stats_neutral" => &["seqkit_stats"],
        "fastq.qc_post" => &["fastqc", "multiqc"],
        "fastq.merge" => &["pear", "vsearch", "bbmerge", "flash2"],
        "fastq.correct" => &["rcorrector", "spades", "bayeshammer", "lighter", "musket"],
        "fastq.umi" => &["umi_tools"],
        "fastq.screen" => &[
            "kraken2",
            "centrifuge",
            "metaphlan",
            "kaiju",
            "fastq_screen",
        ],
        _ => &[],
    };
    tools.iter().map(|tool| (*tool).to_string()).collect()
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &str) -> Option<String> {
    if stage_id == "fastq.preprocess" {
        return Some("planner".to_string());
    }
    canonical_tool_defaults()
        .get(stage_id)
        .map(|tool| (*tool).to_string())
}
