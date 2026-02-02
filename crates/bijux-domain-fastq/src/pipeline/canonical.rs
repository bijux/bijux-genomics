use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct CanonicalPipeline {
    pub required: Vec<String>,
    pub optional: Vec<String>,
}

#[must_use]
pub fn canonical_pipeline() -> CanonicalPipeline {
    CanonicalPipeline {
        required: vec![
            "fastq.validate_pre".to_string(),
            "fastq.detect_adapters".to_string(),
            "fastq.trim".to_string(),
            "fastq.filter".to_string(),
            "fastq.stats_neutral".to_string(),
            "fastq.qc_post".to_string(),
        ],
        optional: vec![
            "fastq.merge".to_string(),
            "fastq.correct".to_string(),
            "fastq.umi".to_string(),
            "fastq.screen".to_string(),
        ],
    }
}

#[must_use]
pub fn canonical_tool_defaults() -> BTreeMap<&'static str, &'static str> {
    BTreeMap::from([
        ("fastq.validate_pre", "fastqvalidator_official"),
        ("fastq.detect_adapters", "fastqc"),
        ("fastq.trim", "fastp"),
        ("fastq.filter", "fastp"),
        ("fastq.stats_neutral", "seqkit_stats"),
        ("fastq.qc_post", "multiqc"),
        ("fastq.merge", "vsearch"),
        ("fastq.correct", "rcorrector"),
        ("fastq.umi", "umi_tools"),
        ("fastq.screen", "kraken2"),
    ])
}
